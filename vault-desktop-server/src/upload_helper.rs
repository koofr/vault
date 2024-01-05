use std::{fs, path::PathBuf};

use vault_core::transfers::state::TransferUploadRelativeName;

pub fn handle_path(
    path: PathBuf,
    upload: Box<dyn Fn(PathBuf, TransferUploadRelativeName) + Send + Sync + 'static>,
    on_error: Box<dyn Fn(PathBuf, std::io::Error) + Send + Sync + 'static>,
) {
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            on_error(path, err);
            return;
        }
    };

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(err) => {
            on_error(path, err);
            return;
        }
    };

    handle_path_metadata(path, &metadata, None, &upload, &on_error);
}

fn handle_path_metadata(
    path: PathBuf,
    metadata: &fs::Metadata,
    name_prefix: Option<&TransferUploadRelativeName>,
    upload: &Box<dyn Fn(PathBuf, TransferUploadRelativeName) + Send + Sync + 'static>,
    on_error: &Box<dyn Fn(PathBuf, std::io::Error) + Send + Sync + 'static>,
) {
    let name = match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => {
            on_error(path, std::io::Error::from(std::io::ErrorKind::InvalidData));

            return;
        }
    };

    if metadata.is_dir() {
        let dir_name_prefix = TransferUploadRelativeName(match name_prefix {
            Some(prefix) => format!("{}{}/", prefix.0, name),
            None => format!("{}/", name),
        });

        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(err) => {
                on_error(path, err);
                return;
            }
        };

        for dir_entry in entries {
            match dir_entry {
                Ok(dir_entry) => {
                    let path = dir_entry.path();

                    match dir_entry.metadata() {
                        Ok(metadata) => {
                            handle_path_metadata(
                                path,
                                &metadata,
                                Some(&dir_name_prefix),
                                upload,
                                on_error,
                            );
                        }
                        Err(err) => {
                            on_error(path, err);
                        }
                    };
                }
                Err(err) => {
                    on_error(path.clone(), err);
                }
            }
        }
    } else {
        let name = TransferUploadRelativeName(match name_prefix {
            Some(prefix) => format!("{}{}", prefix.0, name),
            None => name.to_owned(),
        });

        upload(path, name);
    }
}
