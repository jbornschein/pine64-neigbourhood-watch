// #![feature(alloc_system)]
// extern crate alloc_system;

use dns_lookup;
use gpio_cdev;
use log::{error, info, warn};
use ping;
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::HashMap, net::IpAddr};
use stderrlog;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "neigbourhood-watch", about = "Check our neighbour for health.")]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// IP address of our direct neigbour.
    #[structopt(short = "n", long = "neighbour")]
    neighbour: String,

    /// IP addresses of all our neigbours (the universe).
    #[structopt(
        short = "u",
        long = "universe",
        default_value = "pine1,pine2,pine3,pine4,pine5"
    )]
    universe: String,

    /// Dry run: Do not take any action when encountering problems.
    #[structopt(long = "dry-run")]
    dry_run: bool,

    /// Seconds without contact after which our neighbour is considered dead and will be resetted.
    #[structopt(short = "t", long = "timeout", default_value = "600")]
    timeout: u64,
}

/// Friendly reboot ourself.
#[allow(dead_code)]
fn reset_myself() -> ! {
    println!("Rebooting the system");

    let mut _child = Command::new("/sbin/reboot")
        .spawn()
        .expect("Failed to spawn /sbin/reboot");

    // Forced reboot...
    loop {
        thread::sleep(Duration::from_secs(120));

        println!("Triggering forced reboot");
        let mut _child = Command::new("/sbin/reboot")
            .arg("-f")
            .spawn()
            .expect("Failed to spawn /sbin/reboot -f");
    }
}

/// Reset out neigbour via GPIO pin
fn reset_neighbour() -> Result<(), Box<dyn Error>> {
    // The specific gpio-chip and port are determined by the physical
    // build of our Pine64 cluster:
    //
    // When using the legacy-sysfs API it is gpiochip0, line 34.
    // On the new char-dev based API it is gpiochip1, line 34.
    //
    // In both cases the pin is ACTIVE_LOW, but I'm not sure how to configure
    // that with the cdev API here; so instead we just set the defautl to 1 (on)
    // and explicitly set it to 0 for one second before releasing the line again.
    let mut chip = gpio_cdev::Chip::new("/dev/gpiochip1")?;
    let handle = chip
        .get_line(34)?
        .request(gpio_cdev::LineRequestFlags::OUTPUT, 1, "reset")?;

    info!("Resetting our neigbour");
    handle.set_value(0)?;
    thread::sleep(Duration::from_secs(1));
    handle.set_value(1)?;
    Ok(())
}


fn resolve_hostname(host: &str) -> Result<IpAddr, Box<dyn Error>> {
    let replies =
        dns_lookup::lookup_host(host).map_err(|e| format!("Could not resolve {} ({})", host, e))?;
    Ok(replies.iter().find(|addr| addr.is_ipv4()).unwrap().clone())
}

#[derive(Debug)]
enum State {
    LostUniverse,
    Idle,
    Armed,
}



/// Program Entrypoint
fn main() -> Result<(), Box<dyn Error>> {
    let args = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbose)
        .init()?;

    // Out immediate neighbour we will be watching
    let neighbour = args.neighbour.as_str();
    let neighbour_addr: IpAddr = resolve_hostname(neighbour)?;
    let mut neigbour_last_seen: Option<Instant> = None;

    // Get the list of all neigbours in the universe
    let universe: Vec<&str> = args.universe.split(",").collect();
    let universe_addrs: Vec<IpAddr> = universe
        .iter()
        .map(|&host| resolve_hostname(host))
        .collect::<Result<_, _>>()?;
    let mut universe_last_seen: HashMap<&str, Instant> = HashMap::new();

    // Actual watchdog loop...
    let mut state = State::LostUniverse;
    loop {
        // Ping all hosts in the universe.
        for (&host, addr) in universe.iter().zip(universe_addrs.iter()) {
            let ping_result = ping::ping(
                *addr,
                Some(Duration::from_millis(500)),
                None,
                None,
                None,
                None,
            );

            match ping_result {
                Ok(_) => *universe_last_seen.entry(host).or_insert(Instant::now()) = Instant::now(),
                Err(e) => info!("Failed to ping {}: {}", host, e),
            }
        }

        // Ping our neighbour
        let ping_result = ping::ping(
            neighbour_addr,
            Some(Duration::from_millis(500)),
            None,
            None,
            None,
            None,
        );
        match ping_result {
            Ok(_) => neigbour_last_seen = Some(Instant::now()),
            Err(e) => info!("Failed to ping neighbour {}: {}", neighbour_addr, e),
        }

        //
        let universe_alive = universe_last_seen
            .iter()
            .filter(|(_, &i)| i.elapsed().as_secs() < 30)
            .count()
            >= 2;
        let neighbour_alive =
            neigbour_last_seen.map_or(false, |i| i.elapsed().as_secs() < args.timeout);

        match &state {
            State::LostUniverse => {
                if universe_alive {
                    warn!("[LostUniverse -> Idle] Found my universe.");
                    state = State::Idle;
                }
            }
            State::Idle => {
                if !universe_alive {
                    warn!("[Idle -> LostUniverse] Lost my connection to the universe.");
                    state = State::LostUniverse;
                } else if universe_alive && neighbour_alive {
                    warn!("[Idle -> Armed] Neigbour is alive, activating watchdog.");
                    state = State::Armed;
                }
            }
            State::Armed => {
                if !universe_alive {
                    warn!("[Armed -> LostUniverse] Lost my connection to the universe.");
                    state = State::LostUniverse;
                } else if universe_alive && !neighbour_alive {
                    if !args.dry_run {
                        warn!("[Armed -> Idle] Lost connection to neighbour, RESETTING!");
                        match reset_neighbour() {
                            Ok(_) => info!("Reset triggered."),
                            Err(e) => error!("Error triggering reset: {}", e),
                        }
                    } else {
                        warn!("[Armed -> Idle] Lost connection to neighbour; But DRY-RUN, not resetting neighbour.");
                    }
                    state = State::Idle;
                }
            }
        }

        std::thread::sleep(Duration::from_secs(10));
    }

}
