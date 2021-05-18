extern crate structopt;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use nanostat::Summary;

#[derive(Debug, StructOpt)]
#[structopt(
name = "nanostat",
about = "Check for statistically valid differences between sets of measurements.",
version = env ! ("CARGO_PKG_VERSION"),
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
        help = "The statistical confidence required [0,100)",
        short = "c",
        long = "confidence",
        default_value = "95"
    )]
    confidence: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let ctrl = read_file(&opt.control)?;
    for path in opt.experiments {
        let exp = read_file(&path)?;
        let diff = ctrl.compare(&exp, opt.confidence);

        println!("{}:", path.to_string_lossy());
        if diff.is_significant() {
            let p = format!("{:.3}", diff.p_value);
            let p = p.trim_start_matches('0');
            let op = if exp.mean < ctrl.mean { "<" } else { ">" };

            println!("\tDifference at {} confidence!", opt.confidence);
            println!(
                "\t\t{:.2} {} {:.2} ± {:.2}, p = {}",
                exp.mean, op, ctrl.mean, diff.critical_value, p,
            );
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
    Ok(values.iter().collect())
}
