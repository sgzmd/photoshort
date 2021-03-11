extern crate ffmpeg_next as ffmpeg;
#[macro_use]
extern crate derive_builder;

use std::io::{Error, ErrorKind};
use std::path::Path;

use chrono::Datelike;

use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use log::LevelFilter;

use crate::pserror::error::*;
use config::configurator::{get_config, Config};
use discovery::discovery::make_file_list;
use photo::Photo;

mod config;
mod discovery;
mod photo;
mod pserror;

mod error_messages {
    pub const BOTH_MUST_BE_PROVIDED: &str = "Both --src and --dest must be provided";
}

#[derive(PartialEq, Eq, Debug)]
enum Action {
    HELP,
    CONVERT(Config),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config(Option::None);
    if config.is_err() {
        config::configurator::print_help();
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
                    photo.path().as_ref().unwrap(),
                    photo.new_path().as_ref().unwrap()
                );
            }
            Err(err) => {
                info!("Failed to move photo {:?}: {}", photo.path(), err);
            }
        }
    }
    bar.finish();
}

fn update_new_path(dest_dir: &String, photos: &mut Vec<Photo>) {
    for photo in photos {
        let existing_path = Path::new(photo.path().as_ref().unwrap());
        match existing_path.file_name() {
            None => {
                info!(
                    "Path doesn't appear to have a valid file name: {}",
                    photo
                        .path()
                        .as_ref()
                        .unwrap_or(&"BAD_FILE_NAME".to_string())
                )
            }
            Some(file_name) => {
                // photo must have valid date at this point.
                let date = photo.date().unwrap();
                let path = format!(
                    "{}/{}/{:02}/{:02}/{}",
                    dest_dir,
                    date.year(),
                    date.month(),
                    date.day(),
                    file_name.to_str().unwrap() // should be safe (why?)
                );

                photo.set_new_path(path);
            }
        }
    }
}

fn move_photo(photo: &Photo, move_file: bool, dry_run: bool) -> Result<(), PsError> {
    let new_path = photo.new_path().as_ref().unwrap();

    let full_path = Path::new(new_path);
    let dir = match full_path.parent() {
        None => {
            return Err(PsError::new(
                PsErrorKind::IoError,
                format!("No parent directory for {}", new_path),
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
        info!("Dry-run, not really copying/moving {:?}", photo.path());
        return Ok(());
    }

    // If photo doesn't have path() at this point, it's a fatal mistake.
    let original_path = photo.path().as_ref().unwrap();

    if move_file {
        match std::fs::rename(original_path, &new_path) {
            Ok(_) => {}
            Err(err) => {
                info!("Failed to move file: {}", err);
            }
        }
    } else {
        match std::fs::copy(original_path, &new_path) {
            Ok(_) => {}
            Err(err) => {
                info!("Failed to copy {} -> {}: {}", original_path, &new_path, err);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
