use ::std::fs::read_dir;
use ::std::path::Path;
use ::std::path::PathBuf;

fn is_pdf_file(path: &Path) -> bool {
    matches!(path.extension(), Some(ext) if path.is_file() && ext.to_ascii_lowercase() == "pdf")
}

pub(super) fn filter_pdf_files(files: Vec<PathBuf>) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|path| is_pdf_file(path))
        .collect::<Vec<PathBuf>>()
}

pub(super) fn append_pdf_files(files: &mut Vec<PathBuf>, dir: &Path) {
    match read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            append_pdf_files(files, &path);
                        } else if is_pdf_file(&path) {
                            files.push(path);
                        }
                    }
                    Err(err) => {
                        // TODO Replace with log::warn!.
                        eprintln!("WARNING: Failed to read entry. Error: {:?}", err);
                    }
                }
            }
        }
        Err(err) => {
            // TODO Replace with log::warn!.
            eprintln!(
                "WARNING: Failed to read directory {}. Error: {:?}",
                dir.display(),
                err
            );
        }
    }
}
