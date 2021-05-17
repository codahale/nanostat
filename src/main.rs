extern crate structopt;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use nanostat::{Confidence, Summary};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "nanostat",
    about = "Check for statistically valid differences between sets of measurements.",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Opt {
    #[structopt(
        name = "CONTROL",
        help = "The path to a file with per-line floating point values",
        required = true,
        parse(from_os_str)
    )]
    control: PathBuf,

    #[structopt(
        name = "EXPERIMENT",
        help = "The paths to one or more files with per-line floating point values",
        required = true,
        parse(from_os_str)
    )]
    experiments: Vec<PathBuf>,

    #[structopt(
        name = "P80|P90|P95|P98|P99|P999",
        help = "The statistical confidence required",
        short = "c",
        long = "confidence",
        default_value = "P95"
    )]
    confidence: Confidence,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let ctrl = read_file(&opt.control)?;
    for path in opt.experiments {
        let exp = read_file(&path)?;
        let diff = ctrl.compare(&exp, opt.confidence);

        println!("{}:", path.to_string_lossy());
        if diff.is_significant() {
            println!("\tDifference at {:?} confidence!", opt.confidence);
            println!("\t\t{:.2} +/- {:.2}", diff.delta, diff.error);
            println!(
                "\t\t{:.2}% +/- {:.2}%",
                diff.rel_delta * 100.0,
                diff.rel_error * 100.0
            );
            println!("\t\tStudent's t, pooled s = {}\n", diff.std_dev);
        } else {
            println!("\tNo difference at {:?} confidence.\n", opt.confidence);
        }
    }

    Ok(())
}

fn read_file(path: &Path) -> Result<Summary, Box<dyn Error>> {
    let mut values = vec![];
    for l in BufReader::new(File::open(path)?).lines() {
        values.push(l?.parse()?);
    }
    Ok(Summary::of(&values))
}
