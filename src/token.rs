use hf_hub::{Repo, RepoType, api::sync::ApiBuilder};
use miette::{Context, IntoDiagnostic, miette};
use tokenizers::Tokenizer;

/// Creates a tokenizer instance based on the repository ID. `hf_hub` caches `tokenizer.json`
/// files, so they should only be downloaded once for each model.
///
/// # Returns
/// A `miette::Result` containing the tokenizer.
pub fn create_tokeniser(repo_id: &str) -> miette::Result<Tokenizer> {
    let api = ApiBuilder::new()
        .build()
        .into_diagnostic()
        .wrap_err("building API")?;
    let repo = api.repo(Repo::with_revision(
        repo_id.to_owned(),
        RepoType::Model,
        "main".to_owned(),
    ));
    let tokeniser_filename = repo
        .get("tokenizer.json")
        .into_diagnostic()
        .wrap_err("fetching tokeniser file")?;

    Tokenizer::from_file(tokeniser_filename).map_err(|_| miette!("Initialising tokeniser"))
}

/// Counts the number of tokens in a prompt.
///
/// # Returns
/// A `miette::Result` containing the number of tokens.
///
/// # Errors if unable to encode the prompt.
pub fn count_tokens(tokeniser: &Tokenizer, prompt: &str) -> miette::Result<usize> {
    let add_special_tokens = true;
    let tokens = tokeniser
        .encode_fast(prompt, add_special_tokens)
        .map_err(|_| miette!("Encoding prompt"))?
        .get_ids()
        .to_vec();

    Ok(tokens.len())
}

#[cfg(test)]
mod tests {
    use crate::token::{count_tokens, create_tokeniser};

    #[test]
    fn create_tokeniser_returns_expected_value() {
        // arrange
        let repo_id = "Qwen/Qwen3-1.7B";

        // act
        let tokeniser = create_tokeniser(repo_id);

        // assert
        assert!(tokeniser.is_ok());
    }

    #[test]
    fn count_tokens_returns_expected_value() {
        // arrange
        let repo_id = "Qwen/Qwen3-1.7B";
        let tokeniser = create_tokeniser(repo_id).unwrap();

        // act
        let count = count_tokens(&tokeniser, "Why is the sky blue?").unwrap();

        // assert
        assert_eq!(count, 6);
    }
}
