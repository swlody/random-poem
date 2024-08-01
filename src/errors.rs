use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::DatabaseError(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, Body::empty()).into_response()
            }
            Self::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Body::empty()).into_response()
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
