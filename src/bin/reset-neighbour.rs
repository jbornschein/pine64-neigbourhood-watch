// #![feature(alloc_system)]
// extern crate alloc_system;

use gpio_cdev::LineRequestFlags;
use std::error::Error;
use std::thread;
use std::time::Duration;
use stderrlog;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "reset-neigbour",
    about = "Reset our Pine64 cluster neigbour via GPIO."
)]
struct Args {
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// List GPIO chips
    #[structopt(long = "list-chips", short = "l")]
    list_chips: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbose)
        .init()?;

    if args.list_chips {
        // Lets see if we can access our gpio pins:
        println!("Checking gpio lines:");
        for chip in gpio_cdev::chips()? {
            match chip {
                Ok(chip) =>
                    println!("* gpio {} has {} lines.", chip.name(), chip.num_lines()),
                Err(e) =>
                    println!("Could not open gpio chip: {}", e),
            }
        }
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
    let mut chip = gpio_cdev::Chip::new("/dev/gpiochip1")?;
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
