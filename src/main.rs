mod error;
mod options;
mod runner;

use crate::options::CliOptions;
use runner::{RunConfiguration, Runner};
use structopt::StructOpt;

fn main() {
    let cli_opt: CliOptions = CliOptions::from_args();
    let run_configuration = RunConfiguration::try_from(cli_opt);
    if let Err(e) = run_configuration {
        eprintln!("{}", e);
        return;
    }

    let run_configuration = run_configuration.unwrap();
    let runner = Runner::from(run_configuration);
    let result = runner.run();
    if let Err(e) = result {
        eprintln!("{}", e);
    }
}
