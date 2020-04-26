// #![feature(alloc_system)]
// extern crate alloc_system;

use log::{error, info, trace};
use stderrlog;
use structopt::StructOpt;
use gpio_cdev::{chips, Chip, LineRequestFlags};
use std::error::Error;
use std::thread;
use std::time::Duration;


#[derive(Debug, StructOpt)]
#[structopt(name = "reset-neigbour", about = "Reset out Pine64 cluster neigbour via GPIO.")]
struct Args {
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    // /// Dry run: Do not take any action when encountering problems.
    // #[structopt(short = "f", long = "force")]
    // force: bool,

    // /// Dry run: Do not take any action.
    // #[structopt(long = "dry-run")]
    // dry_run: bool,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbose)
        .init()
        .unwrap();

    info!("Iterating over all gpio chips:");
    for chip in chips()? {
        let chip = chip.expect("Not a chip");
        info!("* gpio {} ({}) has {} lines.", chip.path().display(), chip.name(), chip.num_lines());
    }

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

    println!("Resetting neigbour in 5s...");
    thread::sleep(Duration::from_secs(5));

    println!("Resetting!");
    handle.set_value(0)?;
    thread::sleep(Duration::from_secs(1));
    handle.set_value(1)?;
    println!("Done.");

    Ok(())
}