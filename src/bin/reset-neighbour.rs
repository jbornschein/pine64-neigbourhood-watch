use log::{error, info, trace};
use stderrlog;
use structopt::StructOpt;
use gpio_cdev::{Chip, LineRequestFlags};
use std::error::Error;
use std::thread;
use std::time::Duration;


#[derive(Debug, StructOpt)]
#[structopt(name = "reset-neigbour", about = "Reset out Pine64 cluster neigbour via GPIO.")]
struct Args {
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// Dry run: Do not take any action when encountering problems.
    #[structopt(short = "f", long = "force")]
    force: bool,

    // /// Dry run: Do not take any action.
    // #[structopt(long = "dry-run")]
    // dry_run: bool,
}


fn main() -> Result<(), Box<Error>> {
    let args = Args::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbose)
        .init()
        .unwrap();


    trace!("Starting reset-neigbour.");

    let mut chip = Chip::new("/dev/gpiochip0")?;
    let handle = chip
        .get_line(34)?
        .request(LineRequestFlags::ACTIVE_LOW, 0, "read-input")?;    

    println!("Sending reset in 5s...");
    thread::sleep(Duration::from_secs(5));

    println!("Resetting!");
    handle.set_value(1)?;
    thread::sleep(Duration::from_secs(1));
    handle.set_value(0)?;

    Ok(())
}