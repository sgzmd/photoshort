extern crate args;
extern crate getopts;

use args::{Args, ArgsError};
use getopts::Occur;

mod error_messages {
    pub const BOTH_MUST_BE_PROVIDED: &str = "Both --src and --dest must be provided";
}

#[derive(PartialEq, Eq, Debug)]
enum Action {
    HELP,
    CONVERT(Config),
}

#[derive(PartialEq, Eq, Debug)]
struct Config {
    source: String,
    destination: String,
    logfile: Option<String>,
    dry_run: bool,
}

fn convert_files(config: &Config) {}

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let action = parse(&args);
    match action {
        Ok(action) => match action {
            Action::CONVERT(config) => {
                println!("Starting conversion for config {:?}", config);
            }
            _ => {}
        },
        Err(_) => {}
    }
}

fn parse(input: &Vec<String>) -> Result<Action, ArgsError> {
    let mut args = Args::new("ProgramName", "ProgDesc");
    args.flag("h", "help", "Print the usage menu");
    args.flag("t", "dry_run", "Dry-run, do not make actual changes");
    args.option(
        "s",
        "src",
        "Source directory with photos",
        "SOURCE",
        Occur::Optional,
        None,
    );

    args.option(
        "d",
        "dest",
        "Destination directory",
        "DEST",
        Occur::Optional,
        None,
    );

    args.option(
        "l",
        "log_file",
        "The name of the log file",
        "NAME",
        Occur::Optional,
        None,
    );

    args.parse(input)?;
    let help: Result<bool, ArgsError> = args.value_of("help");

    match help {
        Ok(is_help) => {
            if is_help {
                println!("{}", args.full_usage());
                return Result::Ok(Action::HELP);
            }
        }
        Err(err) => {
            panic!("Error: {}", err);
        }
    }

    let src: Result<String, ArgsError> = args.value_of("src");
    let dst: Result<String, ArgsError> = args.value_of("dest");
    let log: Result<String, ArgsError> = args.value_of("log_file");

    if (src.is_err() || dst.is_err()) {
        println!("{}", args.full_usage());
        return Result::Err(ArgsError::new(
            "args",
            error_messages::BOTH_MUST_BE_PROVIDED,
        ));
    }

    let result = Config {
        source: src?.to_string(),
        destination: dst?.to_string(),
        logfile: match log {
            Ok(log) => Option::from(log),
            Err(_) => Option::None,
        },
        dry_run: match args.value_of("dry_run") {
            Ok(d) => d,
            Err(_) => false,
        },
    };

    // OK it's not help, let' see what we have here.

    return Result::Ok(Action::CONVERT(result));
}

#[cfg(test)]
mod tests;
