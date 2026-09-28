use std::io;

use http::StatusCode;

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct InvalidRepoIdError {
    #[help]
    advice: String,

    detail: String,
}

impl InvalidRepoIdError {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            advice: String::from("Make sure the repo id matches the `<owner>/<name>` pattern"),
            detail: format!("{} does not match the expect repo ID format", id.into()),
        }
    }
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct HfApiError {
    #[help]
    pub advice: String,

    pub detail: String,

    pub cause: hf_hub::HFError,
}

impl From<hf_hub::HFError> for HfApiError {
    fn from(value: hf_hub::HFError) -> Self {
        match value {
            hf_hub::HFError::Request { ref source, .. } => match source.status() {
                Some(StatusCode::NOT_FOUND) => Self {
                    advice: "Check the repo listed in the `models.json` file is correct, the repo \
                        is for a model and that the repo has a `tokenizer.json` file in the root \
                        directory."
                        .to_owned(),
                    detail: format!("{value:?}"),
                    cause: value,
                },
                _ => Self {
                    advice: "Check Hugging Face configuration".to_owned(),
                    detail: format!("{value:?}"),
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

/// Input/output error
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{detail}")]
pub struct IoError {
    /// User-focused remedial suggestion
    #[help]
    advice: String,

    /// Error detail
    detail: String,
}

impl IoError {
    pub fn from_io_with_context<S: Into<String>>(error: &io::Error, cause_action: S) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => Self {
                advice: "Check the path exists".to_owned(),
                detail: format!("{error} while {}", cause_action.into()),
            },
            io::ErrorKind::PermissionDenied => Self {
                advice: "Check you have access permissions".to_owned(),
                detail: format!("{error} while {}", cause_action.into()),
            },
            _ => Self {
                advice: "Check the path exists with access permissions".to_owned(),
                detail: format!("{error} while {}", cause_action.into()),
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
    InvalidRepoId(#[from] InvalidRepoIdError),

    #[diagnostic(transparent)]
    #[diagnostic_source]
    #[error(transparent)]
    Io(#[from] IoError),

    #[diagnostic(transparent)]
    #[diagnostic_source]
    #[error(transparent)]
    Tokenizer(#[from] TokenizerError),
}
