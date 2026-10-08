use hf_hub::HFClient;
use tokenizers::Tokenizer;

use crate::errors::{AppError, HfApiError, TokenizerError};

/// Creates a tokenizer instance based on the repository owner and name. `hf_hub` caches
/// `tokenizer.json` files, so they should only be downloaded once for each model (see
/// <https://docs.rs/hf-hub/latest/hf_hub/struct.HFClientBuilder.html#method.cache_dir>).
///
/// # Returns
/// Generated tokeniser.
pub async fn create_tokeniser(repo_owner: &str, repo_name: &str) -> Result<Tokenizer, AppError> {
    let token = std::env::var("HUGGING_FACE_ACCESS_TOKEN").ok().or_else(|| {
        tracing::warn!(
            "`HUGGING_FACE_ACCESS_TOKEN` environment variable is not defined, using Hugging \
            Face API without an access token.  Performance may vary."
        );

        None
    });
    let client = if let Some(token) = token {
        HFClient::builder()
            .token(&token)
            .build()
            .map_err(HfApiError::from)?
    } else {
        HFClient::new().map_err(HfApiError::from)?
    };
    let repo = client.model(repo_owner, repo_name);
    let tokeniser_filename = repo
        .download_file()
        .filename("tokenizer.json")
        .send()
        .await
        .map_err(HfApiError::from)?;

    Ok(Tokenizer::from_file(tokeniser_filename).map_err(TokenizerError::from)?)
}

/// Counts the number of tokens in a prompt.
///
/// # Returns
/// A `miette::Result` containing the number of tokens.
///
/// # Errors if unable to encode the prompt.
pub fn count_tokens(tokeniser: &Tokenizer, prompt: &str) -> Result<usize, AppError> {
    let add_special_tokens = true;
    let tokens = tokeniser
        .encode_fast(prompt, add_special_tokens)
        .map_err(TokenizerError::from)?
        .get_ids()
        .to_vec();

    Ok(tokens.len())
}

#[cfg(test)]
mod tests {
    use crate::token::{count_tokens, create_tokeniser};

    #[tokio::test]
    async fn create_tokeniser_returns_expected_value() {
        // arrange
        let owner = "Qwen";
        let name = "Qwen3-1.7B";

        // act
        let tokeniser = create_tokeniser(owner, name).await;

        // assert
        assert!(tokeniser.is_ok());
    }

    #[tokio::test]
    async fn count_tokens_returns_expected_value() {
        // arrange
        let owner = "Qwen";
        let name = "Qwen3-1.7B";
        let tokeniser = create_tokeniser(owner, name).await.unwrap();

        // act
        let count = count_tokens(&tokeniser, "Why is the sky blue?").unwrap();

        // assert
        assert_eq!(count, 6);
    }
}
