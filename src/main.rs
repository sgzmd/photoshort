extern crate args;
extern crate ffmpeg_next as ffmpeg;
extern crate getopts;

use std::io::{Error, ErrorKind};
use std::path::Path;
use std::{env, io};

use args::{Args, ArgsError};
use chrono::{DateTime, Datelike, NaiveDateTime};
use exif::{In, Tag};
use ffmpeg::format::context::Input;
use getopts::Occur;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use log::LevelFilter;

mod config;

use config::configurator::Config;
use getopts::Occur::Optional;

mod error_messages {
    pub const BOTH_MUST_BE_PROVIDED: &str = "Both --src and --dest must be provided";
}

#[derive(PartialEq, Eq, Debug)]
enum Action {
    HELP,
    CONVERT(Config),
}

#[derive(Debug)]
struct Photo {
    date: NaiveDateTime,
    path: String,
    new_path: Option<String>,
}

fn main() -> Result<(), Box<std::error::Error>>{
    let config = config::configurator::get_config(Option::None);
    if config.is_err() {
        return Err(config.err().unwrap());
    }

    let config = config.unwrap();
    info!("Starting conversion for config {:?}", config);

    ffmpeg::init().unwrap();
    convert_files(&config);

    return Ok(());
}

fn convert_files(config: &Config) {
    let file_list = make_file_list(&config.source);
    if file_list.is_err() {
        info!("Error building list of files: {:?}", file_list.err());
        return;
    }

    if config.logfile.is_some() {
        let logfile = config.logfile.as_ref().unwrap();
        match simple_logging::log_to_file(logfile, LevelFilter::Info) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Couldn't enable logging: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    let mut file_list = file_list.unwrap();
    info!("Produced a list of {} files", file_list.len());
    update_new_path(&config.destination, &mut file_list);
    info!("Updated a list of {} files", file_list.len());
    let bar = ProgressBar::new(file_list.len() as u64);

    bar.set_message("Moving/copying files ... ");
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:80.green/red} {pos:>7}/{len:7} {msg}")
            .progress_chars("█░"),
    );
    for photo in file_list {
        bar.inc(1);
        match move_photo(&photo, !config.copy, config.dry_run) {
            Ok(_) => {
                info!(
                    "Moved photo {} -> {}",
                    &photo.path,
                    &photo.new_path.as_ref().unwrap()
                );
            }
            Err(err) => {
                info!("Failed to move photo {}: {}", photo.path, err);
            }
        }
    }
    bar.finish();
}

fn make_file_list(input_dir: &String) -> Result<Vec<Photo>, io::Error> {
    info!("Make file list {}", line!());
    use walkdir::WalkDir;

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

        let photo = process_file(path);
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

fn process_file(path: &Path) -> Result<Photo, Error> {
    let file_name = path.file_name();
    if file_name.is_none() {
        return Err(Error::new(ErrorKind::InvalidData, "No file name"));
    }

    let file_name = String::from(
        file_name
            .unwrap() // unwrapping is safe here and below because we know
            .to_str() // file name is not none.
            .unwrap(),
    );

    // Let's skip everything that doesn't look like jpeg/png etc since we don't
    // know how to parse them anyway, so can just as well save time reading it.
    if !(is_supported_file(&file_name)) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("File type not supported: {}", file_name),
        ));
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
                    Err(err) => {
                        return Err(Error::new(ErrorKind::InvalidData, err));
                    }
                }
            }
        }
        // Ignoring, will try ffmpeg later
        Err(_) => {}
    }

    let err = format!("Field {} is empty for {:?}", Tag::DateTimeOriginal, path);

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
        Err(Error::new(ErrorKind::InvalidData, err))
    };
}

fn extract_ndt(creation_time: &str) -> NaiveDateTime {
    let dt = DateTime::parse_from_rfc3339(creation_time);
    let dt = dt.unwrap();
    let ndt: NaiveDateTime = NaiveDateTime::from_timestamp(dt.timestamp(), 0);

    return ndt;
}

fn get_ffmpeg_date(path: &Path) -> Result<NaiveDateTime, Error> {
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

fn stream_creation_time(inp: Input) -> Result<NaiveDateTime, Error> {
    for stream in inp.streams() {
        let meta = stream.metadata();
        let stream_creation_time = meta.get("creation_time");
        if stream_creation_time.is_some() {
            return Ok(extract_ndt(stream_creation_time.unwrap()));
        }
    }

    return Err(Error::new(
        ErrorKind::InvalidData,
        "Cannot extract date/time from file",
    ));
}

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

fn update_new_path(dest_dir: &String, photos: &mut Vec<Photo>) {
    for photo in photos {
        let existing_path = Path::new(&photo.path);
        match existing_path.file_name() {
            None => {
                info!(
                    "Path doesn't appear to have a valid file name: {}",
                    photo.path
                )
            }
            Some(file_name) => {
                let path = format!(
                    "{}/{}/{:02}/{:02}/{}",
                    dest_dir,
                    photo.date.year(),
                    photo.date.month(),
                    photo.date.day(),
                    file_name.to_str().unwrap() // should be safe (why?)
                );

                photo.new_path = Option::Some(path);
            }
        }
    }
}

fn move_photo(photo: &Photo, move_file: bool, dry_run: bool) -> Result<(), Error> {
    return if photo.new_path.is_none() {
        Err(Error::new(ErrorKind::InvalidData, "new_path not available"))
    } else {
        let new_path = photo.new_path.as_ref().unwrap();

        let full_path = Path::new(new_path);
        let dir = match full_path.parent() {
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("No parent directory for {}", new_path).as_str(),
                ));
            }
            Some(dir) => dir,
        };

        if !dir.exists() {
            match std::fs::create_dir_all(dir) {
                Err(err) => {
                    return Err(err.into());
                }
                _ => {}
            }
        }

        if dry_run {
            info!("Dry-run, not really copying/moving {}", &photo.path);
            return Ok(());
        }
        if move_file {
            match std::fs::rename(&photo.path, &new_path) {
                Ok(_) => {}
                Err(err) => {
                    info!("Failed to move file: {}", err);
                }
            }
        } else {
            match std::fs::copy(&photo.path, &new_path) {
                Ok(_) => {}
                Err(err) => {
                    info!("Failed to copy {} -> {}: {}", &photo.path, &new_path, err);
                }
            }
        }
        Ok(())
    };
}

#[cfg(test)]
mod tests;
