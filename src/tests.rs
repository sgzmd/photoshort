#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use std::io;
    use std::path::Path;
    use crate::discovery::*;

    use crate::{
        move_photo, update_new_path, Photo,
    };
    use crate::pserror::error::PsError;


    #[test]
    fn test_walk_tree() -> Result<(), PsError> {
        let input_dir = String::from("./test-assets/");
        let file_list = discovery::make_file_list(&input_dir)?;

        // On the current set of test photographs we expect exactly 54
        // photos and 1 video to have valid exif and date.
        assert_eq!(file_list.len(), 55);

        let _output = file_list.iter().fold(String::new(), |acc, arg| {
            acc + format!("{:?}\n", arg).as_str()
        });

        return Ok(());
    }

    #[test]
    fn test_update_path() {
        use chrono::NaiveDate;

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
        let tempdir = tempfile::tempdir().unwrap();
        let temp_dir_path = tempdir.path().to_str().unwrap().to_string();

        println!("Created temp directory {}", temp_dir_path);
        let original_path = String::from("./test-assets/jpg/Canon_40D.jpg");
        let photo = Photo {
            date: NaiveDate::from_ymd(2008, 5, 30).and_hms(15, 56, 1),
            path: original_path,
            new_path: Option::Some(format!("{}/new_path.jpg", temp_dir_path)),
        };

        println!("Moving {:?}", photo);
        assert!(move_photo(
            &photo, false, /* copying */
            false  /* no dry run */
        )
        .is_ok());
        let mut file = std::fs::File::open(temp_dir_path + "/new_path.jpg")?;
        let original_file = std::fs::File::open(Path::new(&photo.path));
        file_diff::diff_files(&mut file, &mut original_file?);

        return Ok(());
    }
}
