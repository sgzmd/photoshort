extern crate args;
extern crate getopts;

use args::{Args, ArgsError};
use chrono::NaiveDateTime;
use exif::{In, Tag};
use getopts::Occur;
use std::io;
use std::path::Path;
use std::io::{Error, ErrorKind};

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

#[derive(Debug)]
struct Photo {
    date: NaiveDateTime,
    path: String,
    new_path: Option<String>,
}

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let action = parse(&args);
    match action {
        Ok(action) => match action {
            Action::CONVERT(config) => {
                println!("Starting conversion for config {:?}", config);
                convert_files(&config);
            }
            _ => {}
        },
        Err(_) => {}
    }
}


fn convert_files(config: &Config) {
    let file_list = make_file_list(&config.source);
    if file_list.is_err() {
        println!("Error building list of files: {:?}", file_list.err());
        return;
    }

    let mut file_list = file_list.unwrap();
    println!("Produced a list of {} files", file_list.len());
    update_new_path(&config.destination, &mut file_list);
    println!("Updated a list of {} files", file_list.len());
    for photo in file_list {
        match move_photo(&photo, false) {
            Ok(_) => {
                println!("Moved photo {} -> {}", &photo.path, &photo.new_path.as_ref().unwrap());
            }
            Err(err) => {
                println!("Failed to move photo {}: {}", photo.path, err);
            }
        }
    }
}

fn make_file_list(input_dir: &String) -> Result<Vec<Photo>, io::Error> {
    use walkdir::WalkDir;

    let mut result: Vec<Photo> = Vec::new();

    let exifreader = exif::Reader::new();
    for entry in WalkDir::new(input_dir) {
        let entry = entry?;

        println!("Current entry {:?}", entry);

        let path = entry.path();
        if path.is_dir() {
            println!("Skipping directory {:?}", path.to_str());
            continue;
        }

        let file = std::fs::File::open(path)?;
        let mut bufreader = std::io::BufReader::new(&file);
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

                    let no_timezone =
                        NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S").unwrap();
                    result.push(Photo {
                        date: no_timezone,
                        path: String::from(path.to_str().unwrap()),
                        new_path: Option::None,
                    });
                } else {
                    println!("Field {} is empty for {:?}", Tag::DateTimeOriginal, path);
                }
            }
            Err(_) => {
                println!("Failed to read exif data from {}", path.to_str().unwrap());
            }
        }
    }

    return Ok(result);
}

fn update_new_path(dest_dir: &String, photos: &mut Vec<Photo>) {
    use chrono::{Datelike, NaiveDate};

    for photo in photos {
        let existing_path = Path::new(&photo.path);
        match existing_path.file_name() {
            None => {
                println!(
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

fn move_photo(photo: &Photo, move_file: bool) -> Result<(), Error> {
    return if photo.new_path.is_none() {
        Err(Error::new(ErrorKind::InvalidData, "new_path not available"))
    } else {
        let new_path = photo.new_path.as_ref().unwrap();

        let full_path = Path::new(new_path);
        let dir = match full_path.parent() {
            None => { return Err(Error::new(ErrorKind::InvalidData,
                                            format!("No parent directory for {}", new_path).as_str())); }
            Some(dir) => { dir }
        };

        if !dir.exists() {
            match std::fs::create_dir_all(dir) {
                Err(err) => { return Err(err.into()); }
                _ => {}
            }
        }

        if move_file {
            match std::fs::rename(&photo.path, &new_path) {
                Ok(_) => {}
                Err(err) => { println!("Failed to move file: {}", err); }
            }
        } else {
            match std::fs::copy(&photo.path, &new_path) {
                Ok(ok) => { println!("Copied {} bytes presumably", ok); }
                Err(err) => {
                    println!("Failed to copy {} -> {}: {}",
                             &photo.path,
                             &new_path,
                             err);
                }
            }
        }
        Ok(())
    };
}

fn parse(input: &Vec<String>) -> Result<Action, ArgsError> {
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
mod tests;
