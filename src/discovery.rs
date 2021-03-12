pub mod discovery {
    use crate::pserror::error::*;
    use crate::Photo;

    use chrono::{DateTime, NaiveDateTime};
    use indicatif::{ProgressBar, ProgressStyle};
    use log::{info, warn};
    use std::path::Path;
    use walkdir::{DirEntry, WalkDir};

    use crate::photo::PhotoBuilder;
    use exif::{Error, Exif, In, Tag};
    use ffmpeg::format::context::Input;
    use std::fs::File;
    use std::io::{BufReader, Read};

    pub fn is_supported_file(file_name: &str) -> bool {
        let file_name = String::from(file_name).to_lowercase();

        let result = file_name.ends_with("jpg")
            || file_name.ends_with("jpeg")
            || file_name.ends_with("png")
            || file_name.ends_with("gif")
            || file_name.ends_with("mp4")
            || file_name.ends_with("mov")
            || file_name.ends_with("mp");

        info!("File {} is supported: {}", file_name, result);
        return result;
    }

    fn is_zip_file(file_name: &str) -> bool {
        return String::from(file_name).to_lowercase().ends_with(".zip");
    }

    /// Returns all physical files in the input_dir which are supported.
    pub fn list_all_files(input_dir: &str) -> Vec<String> {
        return WalkDir::new(input_dir)
            .into_iter()
            // Can we unwrap result?
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            // Mapping path to string
            .map(|e| e.into_path().into_os_string().into_string().unwrap())
            // Filtering out unsupported files
            .filter(|e| is_supported_file(e) | is_zip_file(e))
            .collect();
    }

    pub fn process_raw_files(files: &Vec<String>) -> Vec<Photo> {
        let bar = ProgressBar::new(files.len() as u64);
        bar.set_message("Collecting information about files....");
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:80.cyan/blue} {pos:>7}/{len:7} {msg}")
                .progress_chars("█░"),
        );

        let mut photos: Vec<Photo> = Vec::new();
        for file in files {
            let path = Path::new(file);
            let result = discover_file(path);
            match result {
                Ok(photo) => {
                    info!("Adding file to collection: {:?}", photo);
                    photos.push(photo);
                }
                Err(err) => {
                    warn!("Couldn't discover file {:?} because of {:?}", path, err);
                }
            }
        }

        bar.finish();
        return photos;
    }

    fn extract_picture_exif(file: &File, path: &Path) -> Result<Photo, PsError> {
        let mut bufreader = std::io::BufReader::new(file);
        let exifreader = exif::Reader::new();
        let exif: Result<Exif, Error> = exifreader.read_from_container(&mut bufreader);

        let photo_path = path.clone().to_str();
        return match_exif(exif, photo_path);
    }

    fn match_exif(exif: Result<Exif, Error>, photo_path: Option<&str>) -> Result<Photo, PsError> {
        match exif {
            Ok(exif) => {
                // TODO(sgzmd): we have to possibly use DateTimeDigitized for, e.g. scanned photos.
                // Unclear how much value this will add, so leaving it for later.
                let field = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY);
                if field.is_some() {
                    let date = field
                        .unwrap() // This is safe because we checked that field.is_some()
                        .display_value()
                        .with_unit(&exif)
                        .to_string();

                    let parsed = NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S");
                    match parsed {
                        Ok(ndt) => {
                            let photo = Photo::from(photo_path.unwrap().to_string(), ndt);
                            return Ok(photo);
                        }
                        Err(_err) => Err(PsError::new(
                            PsErrorKind::FormatError,
                            format!("Couldn't parse date {}", date),
                        )),
                    }
                } else {
                    Err(PsError::new(
                        PsErrorKind::NoExif,
                        format!("No exif field for {:?}", photo_path),
                    ))
                }
            }
            // Ignoring, will try ffmpeg later
            Err(e) => Err(PsError::new(
                PsErrorKind::FormatError,
                format!("Error reading EXIF: {:?} {:?}", photo_path, e),
            )),
        }
    }

    pub fn discover_file(path: &Path) -> Result<Photo, PsError> {
        let file_name = path.file_name();
        if file_name.is_none() {
            return Err(PsError::new(
                PsErrorKind::IoError,
                "file_name is empty".to_string(),
            ));
        }

        let file = std::fs::File::open(path)?;
        let exif = extract_picture_exif(&file, path);

        if exif.is_ok() {
            return Ok(exif.unwrap());
        }

        info!("No regular image exif in {:?}, trying ffmpeg...", path);

        // okay let's try ffmpeg
        let ffmpeg_date = get_ffmpeg_date(&path);
        return if ffmpeg_date.is_ok() {
            let mut photo = Photo::new();
            Ok(Photo::from(
                path.to_str().unwrap().to_string(),
                ffmpeg_date.unwrap(),
            ))
        } else {
            info!("Couldn't get ffmpeg date from {:?}", path);
            return Err(PsError::new(
                PsErrorKind::NoDateField,
                format!("No EXIF of any kind in {:}", path.to_str().unwrap()),
            ));
        };
    }

    fn extract_ndt(creation_time: &str) -> NaiveDateTime {
        let dt = DateTime::parse_from_rfc3339(creation_time);
        let dt = dt.unwrap();
        let ndt: NaiveDateTime = NaiveDateTime::from_timestamp(dt.timestamp(), 0);

        return ndt;
    }

    fn get_ffmpeg_date(path: &Path) -> Result<NaiveDateTime, PsError> {
        let inp: Input = ffmpeg::format::input(&path)?;
        let meta = inp.metadata();
        let file_creation_time = meta.get("creation_time");
        let creation_time = if file_creation_time.is_some() {
            Ok(extract_ndt(file_creation_time.unwrap()))
        } else {
            stream_creation_time(inp)
        };

        return creation_time;
    }

    fn stream_creation_time(inp: Input) -> Result<NaiveDateTime, PsError> {
        for stream in inp.streams() {
            let meta = stream.metadata();
            let stream_creation_time = meta.get("creation_time");
            if stream_creation_time.is_some() {
                return Ok(extract_ndt(stream_creation_time.unwrap()));
            }
        }

        return Err(PsError::new(
            PsErrorKind::NoDateField,
            "Cannot extract date/time from file".to_string(),
        ));
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::NaiveDate;
        use log::LevelFilter;

        pub fn setup() {
            simple_logging::log_to_stderr(LevelFilter::Info);
        }

        fn test_is_supported_file() {
            setup();

            assert!(is_supported_file("filename.jpg"));
            assert!(is_supported_file("filename.png"));
            assert!(is_supported_file("filename.mp4"));
            assert!(is_supported_file("filename.mov"));
            assert!(!is_supported_file("filename.doc"));
        }

        #[test]
        fn test_extract_ndt() {
            setup();

            let dt = "2011-11-05T02:51:16.000000Z";

            assert_eq!(
                extract_ndt(dt),
                NaiveDate::from_ymd(2011, 11, 5).and_hms(2, 51, 16)
            );
        }

        #[test]
        fn test_get_ffmpeg_date() {
            setup();

            let path = Path::new("./test-assets/mpeg/05112011034.mp4");
            let dt = get_ffmpeg_date(&path);

            assert_eq!(
                dt.unwrap(),
                NaiveDate::from_ymd(2011, 11, 5).and_hms(2, 51, 16)
            );
        }

        #[test]
        fn test_list_all_files() {
            setup();

            let all_files = list_all_files("./test-assets");
            assert_eq!(all_files.len(), 92);
        }

        #[test]
        fn test_process_raw_files() {
            setup();

            let supported_files: Vec<String> = list_all_files("./test-assets")
                .into_iter()
                .filter(|e| is_supported_file(e))
                .collect();

            let photos = process_raw_files(&supported_files);
            assert_eq!(photos.len(), 55);
        }
    }
}
