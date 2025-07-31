use std::path::Path;

use miette::{bail, miette};

use crate::utility::read_file;

/// Retrieves the prompt text from a file's content or user input.
///
/// # Returns
/// A `miette::Result` containing the prompt text.
///
/// # Errors
///
/// Errors if both `file` and `prompt` are [`None`].
pub fn get_prompt<P: AsRef<Path>>(file: Option<P>, prompt: Option<&str>) -> miette::Result<String> {
    let prompt = if let Some(value) = file {
        read_file(&value).inspect_err(|err| {
            log::error!(
                "Error reading prompt file (`{}`): {err:?}",
                value.as_ref().display()
            )
        })?
    } else {
        prompt
            .ok_or_else(|| {
                miette!("Supply a file containing the prompt text or the prompt as a string")
            })?
            .to_owned()
    };
    if prompt.trim().is_empty() {
        bail!("Missing a prompt value");
    }
    if prompt.len() > 20_048_000 {
        bail!("Prompt context is too large: {}!", prompt.len());
    }

    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use assert_fs::{
        TempDir,
        prelude::{FileWriteStr, PathChild},
    };

    use crate::prompt::get_prompt;

    #[test]
    fn get_prompt_returns_prompt_for_file_input() {
        // arrange
        let content = "Why is the sky blue?";
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("prompt.txt").write_str(content);
        let temp_data_path = temp_dir.join("prompt.txt");

        // act
        let outcome = get_prompt(Some(temp_data_path), None).unwrap();

        // assert
        assert_eq!(outcome, content);

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn get_prompt_returns_prompt_for_string_input() {
        // arrange
        let input_prompt = "Why is the sky blue?";

        // act
        let outcome = get_prompt(Option::<PathBuf>::None, Some(input_prompt)).unwrap();

        // assert
        assert_eq!(outcome, input_prompt);
    }

    #[test]
    fn get_prompt_handles_empty_prompt() {
        // arrange
        let input_prompt = "";

        // act
        let outcome = get_prompt(Option::<PathBuf>::None, Some(input_prompt)).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("Missing a prompt value".to_owned())
        );
        assert!(chain.next().is_none());
    }

    #[test]
    fn get_prompt_returns_file_input_prompt_when_both_string_and_file_input_are_provided() {
        // arrange
        let content = "Why is the sky blue?";
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("prompt.txt").write_str(content);
        let temp_data_path = temp_dir.join("prompt.txt");
        let input_prompt = "Why is the sea blue?";

        // act
        let outcome = get_prompt(Some(temp_data_path), Some(input_prompt)).unwrap();

        // assert
        assert_eq!(outcome, content);

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn get_prompt_returns_error_if_neither_string_nor_file_input_are_provided() {
        // arrange
        let content = "Why is the sky blue?";
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("prompt.txt").write_str(content);

        // act
        let outcome = get_prompt(Option::<PathBuf>::None, None).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("Supply a file containing the prompt text or the prompt as a string".to_owned())
        );
        assert!(chain.next().is_none());
    }
}
