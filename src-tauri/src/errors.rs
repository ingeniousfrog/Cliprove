use serde::Serialize;
use thiserror::Error;

use crate::models::StructuredError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{message}")]
    Structured {
        code: String,
        message: String,
        suggestion: Option<String>,
        technical_detail: Option<String>,
    },
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<StructuredError> for AppError {
    fn from(value: StructuredError) -> Self {
        Self::Structured {
            code: value.code,
            message: value.message,
            suggestion: value.suggestion,
            technical_detail: value.technical_detail,
        }
    }
}

impl AppError {
    pub fn structured(
        code: impl Into<String>,
        message: impl Into<String>,
        suggestion: Option<String>,
    ) -> Self {
        Self::Structured {
            code: code.into(),
            message: message.into(),
            suggestion,
            technical_detail: None,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
