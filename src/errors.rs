use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::html;
use thiserror::Error;

use crate::render::wrap_body;

pub fn serve_404() -> Response {
    let body = wrap_body(&html! {
        div id="body-content" {
            p {
                "There is no poem here."
            }
            a href="/poem/random" {
                "Click for a random poem"
            }
        }
    });
    (StatusCode::NOT_FOUND, body).into_response()
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::DatabaseError(sqlx::Error::RowNotFound) => serve_404(),
            Self::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Body::empty()).into_response()
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
