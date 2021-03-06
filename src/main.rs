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

fn main() {
    let args = vec!["--help"];
    println!("{:?}", args);
    let action = parse(&args);
    match action {
        Ok(action) => match action {
            Action::HELP => {
                println!("Help!")
            }
            Action::CONVERT(_) => {
                println!("Convert")
            }
        },
        Err(_) => {}
    }
}

fn parse(input: &Vec<&str>) -> Result<Action, ArgsError> {
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
mod tests {
    use crate::{error_messages, parse, Action};
    use args::ArgsError;
    use std::error::Error;

    #[test]
    fn test_parse_help() -> Result<(), ArgsError> {
        let args = vec!["--help"];
        let res = parse(&args);
        assert_eq!(res?, Action::HELP);

        return Ok(());
    }

    #[test]
    fn test_parse_help_must_fail() {
        let args = vec!["--hel"];
        let res = parse(&args);
        assert!(res.is_err(), "This should throw an error");
    }

    #[test]
    fn test_parse_no_src_or_no_dest() {
        assert!(
            parse(&vec!["-s abc"]).is_err(),
            "Source without destination"
        );
        assert!(
            parse(&vec!["-d abc"]).is_err(),
            "Destination without destination"
        );
    }

    #[test]
    fn test_parse_convert() -> Result<(), ArgsError> {
        let res = parse(&vec!["--src=source", "--dest=dst"]);
        match res? {
            Action::HELP => {
                panic!("This is not help command")
            }
            Action::CONVERT(config) => {
                assert_eq!(config.source, "source");
                assert_eq!(config.destination, "dst");
                assert_eq!(config.dry_run, false);
                assert_eq!(config.logfile, Option::None)
            }
        };

        let res = parse(&vec![
            "--src=source",
            "--dest=dst",
            "--log_file=mylogfile",
            "--dry_run",
        ]);

        match res? {
            Action::HELP => {
                panic!("This is not help command")
            }
            Action::CONVERT(config) => {
                assert_eq!(config.dry_run, true);
                assert_eq!(config.logfile, Option::Some(String::from("mylogfile")));
            }
        }

        return Ok(());
    }
}
