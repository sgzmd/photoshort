use crate::config::configurator::Config;
use crate::discovery::discovery;
use crate::discovery::discovery::discover_file;
use crate::pserror::error::{PsError, PsErrorKind};
use crate::{move_photo, update_photo_new_path};
use futures::Future;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::fs::File;
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};
use zip::ZipArchive;

pub fn process_zip_file(file_path: &str, cfg: &Config) -> Result<u64, ()> {
    let file = File::open(Path::new(file_path)).unwrap();
    let mut zf = zip::ZipArchive::new(file).unwrap();

    let bar = ProgressBar::new(zf.len() as u64);

    bar.set_message("Moving/copying files ... ");
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:80.green/red} {pos:>7}/{len:7} {msg}")
            .progress_chars("█░"),
    );

    let mut num_files_copied = 0;
    let temp_dir = TempDir::new().unwrap();
    let temp_dir_path = temp_dir.path();

    for i in 0..zf.len() {
        bar.inc(1);
        let result = process_ith_file(&cfg, &mut zf, i);
        match result {
            Ok(u) => {
                info!("Processed {}-th file", i);
                num_files_copied += u as u64;
            }
            Err(e) => {
                warn!("Failed to process file {}: {:?}", i, e);
            }
        }
    }

    bar.finish();

    Ok((num_files_copied))
}

fn process_ith_file(cfg: &&Config, zf: &mut ZipArchive<File>, i: usize) -> Result<u8, PsError> {
    let mut file = zf.by_index(i).unwrap();
    if file.name().ends_with("/") || !discovery::is_supported_file(file.name()) {
        // Directory, not interesting
        return Err(PsError::new(
            PsErrorKind::FileNotSupported,
            format!("File not supported: {}", file.name()),
        ));
    }

    let mut temp_file = NamedTempFile::new();

    if temp_file.is_err() {
        return Err(PsError::new(
            PsErrorKind::IoError,
            format!("IO Error: {:?}", temp_file.err().unwrap()),
        ));
    }

    let mut temp_file = temp_file.unwrap();
    let written = std::io::copy(&mut file, &mut temp_file).unwrap();
    let temp_file_path = temp_file.path();

    info!(
        "Extracted {} -> {}, {} bytes written",
        file.name(),
        temp_file_path.to_str().unwrap(),
        written
    );

    let photo = discover_file(temp_file_path);
    let new_path = Option::from(
        Path::new(file.name())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
    );
    return match photo {
        Ok(mut photo) => {
            update_photo_new_path(&cfg.destination, &mut photo, new_path);
            let move_file_result = move_photo(
                &photo,
                // note that copy/move flag is ignored here as we
                // are creating temp file which we move later
                false,
                cfg.dry_run,
            );
            match move_file_result {
                Ok(_) => Ok(1),
                Err(e) => {
                    warn!("Failed to move file to {}", new_path.unwrap());
                    Err(e.into())
                }
            }
        }
        Err(err) => {
            info!("Couldn't discover file {}: {:?}", file.name(), err);
            Err(err)
        }
    };
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
