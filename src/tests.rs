#[cfg(test)]
mod tests {
    use crate::{error_messages, make_file_list, parse, Action};
    use args::ArgsError;
    use std::error::Error;
    use std::io;

    #[test]
    fn test_parse_help() -> Result<(), ArgsError> {
        let args = vec![String::from("--help")];
        let res = parse(&args);
        assert_eq!(res?, Action::HELP);

        return Ok(());
    }

    #[test]
    fn test_parse_help_must_fail() {
        let args = vec![String::from("--hel")];
        let res = parse(&args);
        assert!(res.is_err(), "This should throw an error");
    }

    #[test]
    fn test_parse_no_src_or_no_dest() {
        assert!(
            parse(&vec![String::from("-s abc")]).is_err(),
            "Source without destination"
        );
        assert!(
            parse(&vec![String::from("-d abc")]).is_err(),
            "Destination without destination"
        );
    }

    #[test]
    fn test_parse_convert() -> Result<(), ArgsError> {
        let res = parse(&vec![
            String::from("--src=source"),
            String::from("--dest=dst"),
        ]);
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
            String::from("--src=source"),
            String::from("--dest=dst"),
            String::from("--log_file=mylogfile"),
            String::from("--dry_run"),
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

    #[test]
    fn test_walk_tree() -> Result<(), io::Error> {
        let input_dir = String::from("./test-assets/");
        let file_list = make_file_list(&input_dir)?;

        // On the current set of test photographs we expect exactly 54
        // photos to have valid exif and date.
        assert_eq!(file_list.len(), 54);

        let _output = file_list.iter().fold(String::new(), |acc, arg| {
            acc + format!("{:?}\n", arg).as_str()
        });

        return Ok(());
    }
}
