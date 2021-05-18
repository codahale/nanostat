use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use argh::FromArgs;

use nanostat::Summary;
use plotlib::page::Page;
use plotlib::repr::BoxPlot;
use plotlib::view::CategoricalView;

#[derive(Debug, FromArgs)]
#[argh(description = "check for statistically valid differences between sets of measurements")]
struct Opt {
    #[argh(
        positional,
        description = "the path to a file with per-line floating point values"
    )]
    control: String,

    #[argh(
        positional,
        description = "the paths to one or more files with per-line floating point values"
    )]
    experiments: Vec<String>,

    #[argh(
        option,
        short = 'c',
        long = "confidence",
        description = "the statistical confidence required (0,100)",
        default = "95.0"
    )]
    confidence: f64,

    #[argh(
        option,
        long = "box-plot",
        description = "write an SVG box plot to the given path"
    )]
    box_plot: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = argh::from_env();

    let mut plots = CategoricalView::new();

    let (ctrl_data, ctrl) = read_file(&opt.control)?;
    plots = plots.add(BoxPlot::from_vec(ctrl_data).label(&opt.control));

    for path in opt.experiments {
        let (exp_data, exp) = read_file(&path)?;
        plots = plots.add(BoxPlot::from_vec(exp_data).label(&path));

        let diff = ctrl.compare(&exp, opt.confidence);

        println!("{}:", path);
        if diff.is_significant() {
            let p = format!("{:.3}", diff.p_value);
            let p = p.trim_start_matches('0');
            let op = if exp.mean < ctrl.mean { "<" } else { ">" };

            println!("\tDifference at {}% confidence!", opt.confidence);
            println!(
                "\t\t{:.2} {} {:.2} ± {:.2}, p = {}",
                exp.mean, op, ctrl.mean, diff.critical_value, p,
            );
        } else {
            println!("\tNo difference at {}% confidence.\n", opt.confidence);
        }
    }

    if let Some(path) = opt.box_plot {
        Page::single(&plots).save(&path)?;
    }

    Ok(())
}

fn read_file(path: &str) -> Result<(Vec<f64>, Summary), Box<dyn Error>> {
    let mut values = vec![];
    for l in BufReader::new(File::open(path)?).lines() {
        values.push(l?.parse()?);
    }
    let summary = values.iter().collect();
    Ok((values, summary))
}
