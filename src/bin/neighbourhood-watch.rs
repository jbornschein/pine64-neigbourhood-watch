// #![feature(alloc_system)]
// extern crate alloc_system;

use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;
use gpio_cdev::{Chip, LineRequestFlags};
use log::{error, info, trace};
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;
use stderrlog;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
#[structopt(name = "neigbourhood-watch", about = "Check our neighbour for health.")]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// our universe of neigbours
    #[structopt(short = "u", long = "universe")]
    universe: String,

    /// our direct neigbour
    #[structopt(short = "n", long = "neighbour")]
    neighbour: String,

    /// Dry run: Do not take any action when encountering problems.
    #[structopt(long = "dry-run")]
    dry_run: bool,
}

/// Try to re-establish our own network connectivity.
fn reset_network_config() -> Result<(), Box<dyn Error>> {
    let mut _child = Command::new("/sbin/ifup")
        .arg("-a")
        .spawn()?;

    Ok(())
}

/// Friendly reboot ourself.
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
    let mut chip = Chip::new("/dev/gpiochip1")?;
    let handle = chip
        .get_line(34)?
        .request(LineRequestFlags::OUTPUT, 1, "reset")?;    

    info!("Resetting our neigbour");
    handle.set_value(0)?;
    thread::sleep(Duration::from_secs(1));
    handle.set_value(1)?;
    println!("Done.");
    Ok(())
}

fn main() {
    let args = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbose)
        .init()
        .unwrap();

    let (pinger, results) = match Pinger::new(None, None) {
        Ok((pinger, results)) => (pinger, results),
        Err(e) => panic!("Error creating pinger: {}", e),
    };

    // Get the list of all neigbours in the universe and add them to the pinger
    let universe: Vec<&str> = args.universe.split(",").collect();
    for ip in &universe {
        pinger.add_ipaddr(ip)
    }
    pinger.add_ipaddr(&args.neighbour);

    loop {
        pinger.ping_once();

        for _ in 0..(universe.len() + 1) {
            let mut num_universe_alive: i32 = 0;

            match results.recv() {
                Ok(result) => match result {
                    Idle { addr } => {
                        error!("Idle Address {}.", addr);
                    }
                    Receive { addr, rtt } => {
                        info!("Receive from Address {} in {:?}.", addr, rtt);
                        num_universe_alive += 1;
                    }
                },
                Err(_) => panic!("Worker threads disconnected before the solution was found!"),
            }
        }
    }
}
