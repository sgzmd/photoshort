extern crate clap;

use clap::{App, Arg, SubCommand};
use std::env;

pub mod configurator {
    use super::clap::ArgMatches;
    use crate::Config;

    pub fn create_parser(input: Option<&Vec<String>>) -> ArgMatches {
        let matches = clap::App::new("Photosort")
            .version("0.0.1")
            .author("Roman 'sgzmd' Kirillov <sigizmund@gmail.com>")
            .arg(
                clap::Arg::with_name("mode")
                    .short("m")
                    .value_name("MODE")
                    .default_value("move")
                    .help("Copy (copy) or move (move) files, e.g. -m copy")
                    .possible_values(&["copy", "move"])
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name("src")
                    .short("s")
                    .value_name("SOURCE")
                    .required(true)
                    .empty_values(false)
                    .takes_value(true)
                    .help("Source directory with files, e.g. -s /path/to/photos"),
            )
            .arg(
                clap::Arg::with_name("dst")
                    .short("d")
                    .value_name("DESTINATION")
                    .required(true)
                    .empty_values(false)
                    .takes_value(true)
                    .help("Destination directory, e.g. -d /path/to/photos"),
            )
            .arg(
                clap::Arg::with_name("log")
                    .short("l")
                    .value_name("LOG")
                    .empty_values(false)
                    .takes_value(true)
                    .help("Log file to use, e.g. -l output.log"),
            )
            .arg(
                clap::Arg::with_name("verbose")
                    .short("v")
                    .takes_value(false)
                    .help("Level of verbosity, default -v0"),
            )
            .arg(
                clap::Arg::with_name("dry_run")
                    .short("t")
                    .takes_value(false)
                    .help("Dry-run, no changes are being written"),
            );

        return if input.is_some() {
            matches.get_matches_from(input)
        } else {
            matches.get_matches()
        };
    }

    pub fn get_config(input: Option<&Vec<String>>) -> Config {
        let matches = create_parser(input);

        return Config {
            source: "".to_string(),
            destination: "".to_string(),
            logfile: None,
            dry_run: false,
            copy: false
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        use super::configurator::*;
        let mut options: Vec<String> = Vec::new();
        // options.push(String::from("--help"));
        options.push(String::from("--mode move1"));
        create_parser(Option::from(&options));
    }
}
