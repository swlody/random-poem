use askama::Template;
use askama_web::WebTemplate;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error(transparent)]
    RenderError(#[from] askama::Error),
}

#[tracing::instrument]
pub fn serve_404() -> impl IntoResponse {
    #[derive(Template, WebTemplate)]
    #[template(path = "404.html")]
    struct NotFoundTemplate;

    NotFoundTemplate
}

#[derive(Template, WebTemplate)]
#[template(path = "something_went_wrong.html")]
struct SomethingWentWrongTemplate;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            // RowNotFound is expected, anything else is a problem
            Self::DatabaseError(sqlx::Error::RowNotFound) => serve_404().into_response(),
            Self::DatabaseError(_) => SomethingWentWrongTemplate.into_response(),
            Self::RenderError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
