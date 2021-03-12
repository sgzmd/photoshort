use crate::config::configurator::Config;
use crate::discovery::discovery;
use crate::discovery::discovery::discover_file;
use crate::photo::Photo;
use crate::pserror::error::PsError;
use crate::{move_photo, update_photo_new_path};
use filepath::FilePath;
use log::info;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;
use tempfile::NamedTempFile;
use indicatif::{ProgressBar, ProgressStyle};

pub fn process_zip_file(file_path: &str, cfg: &Config) -> Result<u64, PsError> {
    let file = File::open(Path::new(file_path))?;
    let mut zf = zip::ZipArchive::new(file)?;

    let bar = ProgressBar::new(zf.len() as u64);

    bar.set_message("Moving/copying files ... ");
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:80.green/red} {pos:>7}/{len:7} {msg}")
            .progress_chars("█░"),
    );


    let mut num_files_copied = 0;
    for i in 0..zf.len() {
        bar.inc(1);
        let mut file = zf.by_index(i)?;
        if file.name().ends_with("/") || !discovery::is_supported_file(file.name()) {
            // Directory, not interesting
            continue;
        }

        let mut temp_file = NamedTempFile::new()?;

        let (mut temp_file, temp_path) = temp_file.into_parts();
        let temp_file_path = temp_path.to_str().unwrap(); // unwrap shouldn't fail.
        let written = std::io::copy(&mut file, &mut temp_file)?;

        info!(
            "Extracted {} -> {}, {} bytes written",
            file.name(),
            temp_file_path,
            written
        );

        let mut photo = discover_file(Path::new(Path::new(temp_file_path)));
        match photo {
            Ok(mut photo) => {
                update_photo_new_path(&cfg.destination, &mut photo, Option::from(file.name()));
                move_photo(&photo, !cfg.copy, cfg.dry_run);
                num_files_copied += 1;
            }
            Err(err) => {
                info!("Couldn't discover file {}: {:?}", file.name(), err);
            }
        }
    }

    bar.finish();

    Ok((num_files_copied))
}

#[cfg(test)]
mod tests {
    use crate::config::configurator::Config;
    use crate::zipfiles::process_zip_file;
    use log::LevelFilter;
    use walkdir::DirEntry;

    #[test]
    fn test_process_zip_file() {
        simple_logging::log_to_stderr(LevelFilter::Info);

        let temp_dir = tempfile::tempdir().unwrap();
        let source = "./test-assets/assets.zip";
        let cfg = Config {
            source: source.to_string(),
            destination: temp_dir.path().to_str().unwrap().to_string(),
            logfile: None,
            dry_run: false,
            copy: true,
        };

        let num_files_copied = process_zip_file(source, &cfg).unwrap();
        assert_eq!(num_files_copied, 55);
        let result: Vec<DirEntry> = walkdir::WalkDir::new(cfg.destination)
            .into_iter()
            .map(|e| e.unwrap())
            .filter(|e| e.path().is_file())
            .collect();

        assert_eq!(num_files_copied, result.len() as u64);
    }
}
