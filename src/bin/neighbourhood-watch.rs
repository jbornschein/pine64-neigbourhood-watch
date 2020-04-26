use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;
use log::{error, info, trace};
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

fn reset_network() {
    /// Try to reset and reestablish our own network connectivity.
    let mut _child = Command::new("/sbin/ifup")
        .arg("-a")
        .spawn()
        .expect("Failed to spawn /sbin/ifup");
}

fn reset_myself() {
    // Friendly reboot ourself system/
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

fn reset_neighbour() {
    /// Reset out neigbour
    /// 
    ///  echo "34" > /sys/class/gpio/export
    ///  echo 1     > /sys/class/gpio/gpio34/active_low
    ///  echo "out" > /sys/class/gpio/gpio34/direction
    ///  echo 1     > /sys/class/gpio/gpio34/value
    ///  sleep 1
    ///  echo 0 > /sys/class/gpio/gpio34/value
    error!("Failed to reset neigbour: not implemented yet!")
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
