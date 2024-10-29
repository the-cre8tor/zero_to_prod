use std::fmt::{Debug, Display};

use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;

// Return an opaque 500 while preserving the error root's cause for logging.
pub fn error_500<T>(error: T) -> actix_web::Error
where
    T: Debug + Display + 'static,
{
    ErrorInternalServerError(error)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

// Return a 400 with the user-representation of the validation error as body.
// The error root cause is preserved for logging purposes.
pub fn error_400<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorBadRequest(e)
}
