extern crate clap;

use clap::{App, Arg, SubCommand};
use std::env;

pub mod configurator {
    use super::clap::{App, ArgMatches};
    use std::error::Error;

    #[derive(PartialEq, Eq, Debug)]
    pub struct Config {
        pub source: String,
        pub destination: String,
        pub logfile: Option<String>,
        // TODO(sgzmd): these two need to be merged into a single enum
        pub dry_run: bool,
        pub copy: bool,
    }

    fn configure_matchers() -> App<'static, 'static> {
        clap::App::new("Photosort")
            .version("0.0.1")
            .author("Roman 'sgzmd' Kirillov <sigizmund@gmail.com>")
            .arg(
                clap::Arg::with_name("src")
                    .long("src")
                    .short("s")
                    .value_name("SOURCE")
                    .required(true)
                    .empty_values(false)
                    .takes_value(true)
                    .help("Source directory with files, e.g. -s /path/to/photos"),
            )
            .arg(
                clap::Arg::with_name("dst")
                    .long("dst")
                    .short("d")
                    .value_name("DESTINATION")
                    .required(true)
                    .empty_values(false)
                    .takes_value(true)
                    .help("Destination directory, e.g. -d /path/to/photos"),
            )
            .arg(
                clap::Arg::with_name("mode")
                    .long("mode")
                    .short("m")
                    .value_name("MODE")
                    .default_value("move")
                    .help("Copy or move files")
                    .possible_values(&["copy", "move"])
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name("log")
                    .long("log")
                    .short("l")
                    .value_name("LOG")
                    .empty_values(false)
                    .takes_value(true)
                    .help("Log file to use, e.g. -l output.log"),
            )
            .arg(
                clap::Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .takes_value(false)
                    .help("Level of verbosity, default -v0"),
            )
            .arg(
                clap::Arg::with_name("dry_run")
                    .long("dry_run")
                    .short("t")
                    .takes_value(false)
                    .help("Dry-run, no changes are being written"),
            )
    }

    pub fn print_help() {
        configure_matchers().print_long_help();
    }

    pub fn get_config(input: Option<&Vec<&str>>) -> Result<Config, Box<Error>> {
        let matches = configure_matchers();

        let result = if input.is_some() {
            matches.get_matches_from_safe(input.unwrap().iter())
        } else {
            matches.get_matches_safe()
        };

        let matches = result?;
        return Result::Ok(Config {
            // Unwrap is safe because next two are required parameters
            source: matches.value_of("src").unwrap().to_string(),
            destination: matches.value_of("dst").unwrap().to_string(),
            logfile: matches.value_of("log").map(|s| s.to_string()),
            dry_run: match matches.occurrences_of("dry_run") {
                0 => false,
                1 | _ => true,
            },
            copy: match matches.value_of("mode") {
                None => false,
                Some(val) => match val {
                    "copy" => true,
                    "move" | _ => false,
                },
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    #[test]
    fn test_parse_fail() {
        use super::configurator::*;
        let options = vec!["--mode something"];
        assert!(get_config(Option::from(&options)).is_err(), "Broken config");
    }

    #[test]
    fn test_parse_full_config() -> Result<(), Box<Error>> {
        use super::configurator::*;
        let options = vec![
            "CommandName",
            "-sSOURCE",
            "--dst=DEST",
            "--mode=move",
            "--log=some.file.log",
            "-t",
        ];
        let config = get_config(Option::from(&options))?;

        let expected_config = Config {
            source: "SOURCE".to_string(),
            destination: "DEST".to_string(),
            logfile: Option::from("some.file.log".to_string()),
            dry_run: true,
            copy: false,
        };

        assert_eq!(config, expected_config);

        return Ok(());
    }
}
