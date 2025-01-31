use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

use rinja::Template;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error(transparent)]
    RenderError(#[from] rinja::Error),
}

#[tracing::instrument]
pub fn serve_404() -> Result<impl IntoResponse> {
    #[derive(Template)]
    #[template(path = "404.html")]
    struct NotFoundTemplate;

    Ok(Html(NotFoundTemplate.render()?))
}

#[derive(Template)]
#[template(path = "something_went_wrong.html")]
struct SomethingWentWrongTemplate;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            // RowNotFound is expected, anything else is a problem
            Self::DatabaseError(sqlx::Error::RowNotFound) => serve_404()
                .map(|html| html.into_response())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
            Self::DatabaseError(_) => SomethingWentWrongTemplate
                .render()
                .map(|html| html.into_response())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
            Self::RenderError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
