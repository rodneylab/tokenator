#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct HfApiError {
    #[help]
    #[allow(unused_assignments)]
    pub advice: String,

    #[allow(unused_assignments)]
    pub detail: String,

    #[allow(unused_assignments)]
    pub cause: hf_hub::api::sync::ApiError,
}

impl From<hf_hub::api::sync::ApiError> for HfApiError {
    fn from(value: hf_hub::api::sync::ApiError) -> Self {
        Self {
            advice: "Check Hugging Face configuration".to_owned(),
            detail: format!("{value:?}"),
            cause: value,
        }
    }
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct TokenizerError {
    #[help]
    #[allow(unused_assignments)]
    pub advice: String,

    #[allow(unused_assignments)]
    pub detail: String,

    #[allow(unused_assignments)]
    pub cause: tokenizers::tokenizer::Error,
}

impl From<tokenizers::tokenizer::Error> for TokenizerError {
    fn from(value: tokenizers::tokenizer::Error) -> Self {
        Self {
            advice: "Tokenizer file might be corrupted. Try re-running the last operation"
                .to_owned(),
            detail: format!("Tokenizer error: {value:?}"),
            cause: value,
        }
    }
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum AppError {
    #[diagnostic(transparent)]
    #[diagnostic_source]
    #[error(transparent)]
    HfApi(#[from] HfApiError),

    #[diagnostic(transparent)]
    #[diagnostic_source]
    #[error(transparent)]
    Tokenizer(#[from] TokenizerError),
}
