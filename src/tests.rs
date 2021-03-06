#[cfg(test)]
mod tests {
    use crate::{error_messages, make_file_list, parse, update_new_path, Action, Photo, move_photo};
    use args::ArgsError;
    use chrono::{NaiveDateTime, NaiveDate};
    use std::error::Error;
    use std::io;
    use file_diff::{diff_files};
    use std::path::Path;

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

    #[test]
    fn test_update_path() {
        use chrono::{Datelike, NaiveDate};

        let dest_dir = String::from("TEST_DIR");
        let mut photos = vec![
            Photo {
                date: NaiveDate::from_ymd(2021, 3, 6).and_hms(16, 47, 13),
                path: "my/current/path.jpg".to_string(),
                new_path: None,
            },
            Photo {
                date: NaiveDate::from_ymd(2002, 2, 6).and_hms(16, 47, 13),
                path: "my/current/another_path.jpg".to_string(),
                new_path: None,
            },
        ];

        update_new_path(&dest_dir, &mut photos);
        assert_eq!(
            photos[0].new_path.as_ref().unwrap(),
            "TEST_DIR/2021/03/06/path.jpg"
        );
        assert_eq!(
            photos[1].new_path.as_ref().unwrap(),
            "TEST_DIR/2002/02/06/another_path.jpg"
        );
    }

    #[test]
    fn copy_file_test() -> Result<(), io::Error> {
        let mut tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_string();

        println!("Created temp directory {}", temp_dir_path);
        let original_path = String::from("./test-assets/jpg/Canon_40D.jpg");
        let photo = Photo {
            date: NaiveDate::from_ymd(2008, 5, 30).and_hms(15, 56, 1),
            path: original_path,
            new_path: Option::Some(format!("{}/new_path.jpg", temp_dir_path))
        };

        println!("Moving {:?}", photo);
        assert!(move_photo(&photo, false /* copying */ ).is_ok());
        let mut file = std::fs::File::open(temp_dir_path + "/new_path.jpg")?;
        let mut original_file = std::fs::File::open(Path::new(&photo.path));
        file_diff::diff_files(&mut file, &mut original_file?);

        return Ok(());
    }

    #[test]
    fn test_copy_photos() {
        let mut tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().to_str().unwrap();
    }
}
