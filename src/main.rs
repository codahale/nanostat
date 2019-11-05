extern crate clap;

use std::convert::TryInto;
use std::error;
use std::fs::File;
use std::io;
use std::io::BufRead;

use clap::{App, Arg};

use nanostat::Summary;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = App::new("nanostat")
        .about("Check for statistically valid differences between sets of measurements.")
        .version(VERSION)
        .arg(
            Arg::with_name("control")
                .value_name("CONTROL")
                .help("The path to a file with per-line floating point values")
                .required(true),
        )
        .arg(
            Arg::with_name("confidence")
                .short("c")
                .long("confidence")
                .value_name("PERCENTAGE")
                .takes_value(true)
                .default_value("P95"),
        )
        .arg(
            Arg::with_name("experiments")
                .value_name("EXPERIMENT")
                .multiple(true)
                .takes_value(true)
                .help("The path to one or more files with per-line floating point values")
                .required(true),
        )
        .get_matches();

    let confidence = matches.value_of("confidence").unwrap().try_into()?;

    let ctrl = read_file(matches.value_of("control").unwrap())?;
    let ctrl_sum = Summary::of(&ctrl);

    for path in matches.values_of("experiments").unwrap() {
        let exp = read_file(path)?;
        let exp_sum = Summary::of(&exp);
        let diff = ctrl_sum.compare(&exp_sum, confidence);

        println!("{}:", path);
        if diff.is_significant() {
            println!("\tDifference at {:?} confidence!", confidence);
            println!("\t\t{:.2} +/- {:.2}", diff.delta, diff.error);
            println!(
                "\t\t{:.2}% +/- {:.2}%",
                diff.rel_delta * 100.0,
                diff.rel_error * 100.0
            );
            println!("\t\tStudent's t, pooled s = {}\n", diff.std_dev);
        } else {
            println!("\tNo difference at {:?} confidence.\n", confidence);
        }
    }

    Ok(())
}

fn read_file(path: &str) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let mut values = vec![];
    for l in io::BufReader::new(File::open(path)?).lines() {
        values.push(l?.parse()?);
    }
    Ok(values)
}
