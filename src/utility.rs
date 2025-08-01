use std::{fs, io, path::Path};

use miette::{Context, IntoDiagnostic, bail};

/// Reads the content of a file into a string.
///
/// # Errors
/// Errors if the user has insufficient permission to read the file, it does not exist, or is
/// too large.
pub fn read_file<P: AsRef<Path>>(path: P) -> miette::Result<String> {
    let metadata = fs::metadata(&path)
        .inspect_err(|err| match err.kind() {
            io::ErrorKind::NotFound => {
                log::error!("File `{}` not found", path.as_ref().display());
            }
            io::ErrorKind::PermissionDenied => {
                log::error!(
                    "Insufficient permissions to read file `{}`",
                    path.as_ref().display()
                );
            }
            _ => {
                log::error!("Error reading from file `{}`", path.as_ref().display());
                log::debug!(
                    "Error reading from file `{}`: {err:?}",
                    path.as_ref().display()
                );
            }
        })
        .into_diagnostic()
        .wrap_err(format!("Error opening file `{}`", path.as_ref().display()))?;
    let filesize = metadata.len();
    if filesize > 10_485_760 {
        bail!("File is too large.")
    }
    if filesize == 0 {
        log::warn!("File `{}` is empty.", path.as_ref().display());
    }
    fs::read_to_string(&path)
        .inspect_err(|err| match err.kind() {
            io::ErrorKind::InvalidData => {
                log::error!(
                    "Unable to read file `{}`.  Check it only contains valid UTF-8 data.",
                    path.as_ref().display()
                );
            }
            io::ErrorKind::PermissionDenied => {
                log::error!(
                    "Insufficient permissions to read file `{}`",
                    path.as_ref().display()
                );
            }
            _ => {
                log::error!("Error reading from file `{}`", path.as_ref().display());
                log::debug!(
                    "Error reading from file `{}`: {err:?}",
                    path.as_ref().display()
                );
            }
        })
        .into_diagnostic()
        .wrap_err(format!("Error reading file `{}`", path.as_ref().display()))
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

    use assert_fs::prelude::{FileWriteBin, FileWriteStr as _, PathChild as _};

    use crate::utility::read_file;

    #[test]
    fn read_file_handles_valid_input() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let filename = "example.txt";
        let file_path = temp_dir.join(filename);
        let content = "This is a valid UTF-8 string.";
        temp_dir.child(filename).write_str(content).unwrap();

        // act
        let result = read_file(&file_path).unwrap();

        // assert
        assert_eq!(result, content);

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn read_file_returns_error_if_file_contains_non_utf8_bytes() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let filename = "example.txt";
        let file_path = temp_dir.join(filename);
        temp_dir
            .child(filename)
            .write_binary(&[0xF8, 0x82, 0x80])
            .unwrap();

        // act
        let outcome = read_file(&file_path).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some(format!("Error reading file `{}`", file_path.display()))
        );
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("stream did not contain valid UTF-8".to_owned())
        );
        assert!(chain.next().is_none());

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn read_file_returns_error_if_file_does_not_exist() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let filename = "does-not-exist.txt";
        let file_path = temp_dir.join(filename);

        // act
        let outcome = read_file(&file_path).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some(format!("Error opening file `{}`", file_path.display()))
        );
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("No such file or directory (os error 2)".to_owned())
        );
        assert!(chain.next().is_none());

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn read_file_returns_error_if_directory_does_not_exist() {
        // arrange
        let file_path = PathBuf::from("./does-not-exist/example.txt");

        // act
        let outcome = read_file(&file_path).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some(format!("Error opening file `{}`", file_path.display()))
        );
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("No such file or directory (os error 2)".to_owned())
        );
        assert!(chain.next().is_none());
    }

    #[test]
    fn read_file_returns_error_if_user_does_not_have_access_permissions() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let filename = "example.txt";
        let file_path = temp_dir.join(filename);
        let content = "This is a valid UTF-8 string.";
        temp_dir.child(filename).write_str(content).unwrap();
        fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o044)).unwrap();

        // act
        let outcome = read_file(&file_path).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some(format!("Error reading file `{}`", file_path.display()))
        );
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("Permission denied (os error 13)".to_owned())
        );
        assert!(chain.next().is_none());

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn read_file_handles_an_empty_file() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let filename = "example.txt";
        let file_path = temp_dir.join(filename);
        let content = "";
        temp_dir.child(filename).write_str(content).unwrap();

        // act
        let result = read_file(&file_path).unwrap();

        // assert
        assert!(result.is_empty());

        // cleanup
        temp_dir.close().unwrap();
    }
}
