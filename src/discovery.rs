pub mod discovery {
  use crate::pserror::error::*;
  use crate::Photo;

  use chrono::{DateTime, NaiveDateTime};
  use indicatif::{ProgressBar, ProgressStyle};
  use log::info;
  use std::path::Path;
  use walkdir::WalkDir;
  
  use exif::{In, Tag};
  use ffmpeg::format::context::Input;


  fn is_supported_file(file_name: &str) -> bool {
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

  pub fn make_file_list(input_dir: &String) -> Result<Vec<Photo>, PsError> {
    info!("Make file list {}", line!());
    let mut result: Vec<Photo> = Vec::new();

    info!("Starting to walk the file tree..");
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(120);
    pb.set_style(
      ProgressStyle::default_spinner()
        .tick_strings(&["/", "-", "\\", "|", "✅"])
        .template("{spinner:.blue} {msg}"),
    );
    pb.set_message("Reading list of files, might take a while ...");
    let all_entries: Vec<Result<walkdir::DirEntry, walkdir::Error>> =
      WalkDir::new(input_dir).into_iter().collect();
    pb.finish_with_message("Done");

    let bar = ProgressBar::new(all_entries.len() as u64);
    bar.set_message("Collecting information about files....");
    bar.set_style(
      ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:80.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("█░"),
    );
    for entry in all_entries {
      bar.inc(1);
      match entry {
        Err(_) => {
          continue;
        }
        _ => {}
      };
      let entry = entry.unwrap();
      let path = entry.path();
      if path.is_dir() {
        info!("Skipping directory {:?}", path.to_str());
        continue;
      }

      if !is_supported_file(entry.file_name().to_str().unwrap_or("")) {
        continue;
      }

      let photo = discover_file(path);
      match photo {
        Ok(photo) => {
          result.push(photo);
        }
        Err(err) => {
          info!(
            "Error processing file {}: {:?}",
            path.to_str().unwrap(),
            err
          )
        }
      }
    }
    bar.finish();

    return Ok(result);
  }

  fn discover_file(path: &Path) -> Result<Photo, PsError> {
    let file_name = path.file_name();
    if file_name.is_none() {
      return Err(PsError::new(PsErrorKind::IoError, "file_name is empty".to_string()));
    }

    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    let exif = exifreader.read_from_container(&mut bufreader);
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
              return Ok(Photo {
                date: ndt,
                path: String::from(path.to_str().unwrap()),
                new_path: Option::None,
              });
            }
            Err(_err) => {
              return Err(PsError::new(PsErrorKind::FormatError,
                                      format!("Couldn't parse date {}", date)));
            }
          }
        }
      }
      // Ignoring, will try ffmpeg later
      Err(_) => {}
    }

    let _err = format!("Field {} is empty for {:?}", Tag::DateTimeOriginal, path);

    // okay let's try ffmpeg
    let ffmpeg_date = get_ffmpeg_date(&path);
    return if ffmpeg_date.is_ok() {
      Ok(Photo {
        date: ffmpeg_date.unwrap(),
        path: String::from(path.to_str().unwrap()),
        new_path: Option::None,
      })
    } else {
      info!("Couldn't get ffmpeg date from {:?}", path);
      return Err(PsError::new(PsErrorKind::NoDateField,
                       format!("Couldn't get ffmpeg date from {:?}", path)));
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

    fn test_is_supported_file() {
      assert!(is_supported_file("filename.jpg"));
      assert!(is_supported_file("filename.png"));
      assert!(is_supported_file("filename.mp4"));
      assert!(is_supported_file("filename.mov"));
      assert!(!is_supported_file("filename.doc"));
    }

    #[test]
    fn test_extract_ndt() {
      let dt = "2011-11-05T02:51:16.000000Z";

      assert_eq!(
        extract_ndt(dt),
        NaiveDate::from_ymd(2011, 11, 5).and_hms(2, 51, 16)
      );
    }

    #[test]
    fn test_get_ffmpeg_date() {
      let path = Path::new("./test-assets/mpeg/05112011034.mp4");
      let dt = get_ffmpeg_date(&path);

      assert_eq!(
        dt.unwrap(),
        NaiveDate::from_ymd(2011, 11, 5).and_hms(2, 51, 16)
      );
    }
  }
}
