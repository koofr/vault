use std::path::PathBuf;

use tokio::fs;

use vault_core::utils::name_utils;

pub async fn create_unused_file(path: PathBuf) -> std::io::Result<(fs::File, PathBuf, String)> {
    let parent_path = path
        .parent()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;

    let (base_name, ext) = name_utils::split_name_ext(&name);

    let mut i = 0;

    loop {
        let new_name = if i == 0 {
            name.clone()
        } else {
            name_utils::join_name_ext(&format!("{} ({})", base_name, i), ext)
        };

        i += 1;

        let path = parent_path.join(&new_name);

        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(file) => return Ok((file, path, new_name)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }
}

pub fn cleanup_name(name: &str) -> String {
    let name = name.replace(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'], "");

    match name.as_str() {
        "" => "invalid name".into(),
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use super::cleanup_name;

    #[test]
    pub fn test_cleanup_name() {
        assert_eq!(cleanup_name("file.txt"), "file.txt");
        assert_eq!(cleanup_name("file <1>.txt"), "file 1.txt");
        assert_eq!(cleanup_name("foo:bar.txt"), "foobar.txt");
        assert_eq!(cleanup_name("\"/\\|?*"), "invalid name");
    }
}
