#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct HfApiError {
    #[help]
    pub advice: String,

    pub detail: String,

    pub cause: hf_hub::api::sync::ApiError,
}

impl From<hf_hub::api::sync::ApiError> for HfApiError {
    fn from(value: hf_hub::api::sync::ApiError) -> Self {
        match value {
            hf_hub::api::sync::ApiError::RequestError(ref err) => match &**err {
                ureq::Error::Transport(transport) => {
                    let url = transport.url();
                    Self {
                        advice: "Check your Internet connection.".to_owned(),
                        detail: if let Some(url) = url {
                            format!("Error connecting to API URL `{url}`")
                        } else {
                            "Api request error".to_owned()
                        },
                        cause: value,
                    }
                }
                ureq::Error::Status(code, response) => Self {
                    advice: "Check your configuration.".to_owned(),
                    detail: format!(
                        "Api request error {}, status code: {code}",
                        response.status_text()
                    ),
                    cause: value,
                },
            },
            _ => Self {
                advice: "Check Hugging Face configuration".to_owned(),
                detail: format!("{value:?}"),
                cause: value,
            },
        }
    }
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct TokenizerError {
    #[help]
    pub advice: String,

    pub detail: String,

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
