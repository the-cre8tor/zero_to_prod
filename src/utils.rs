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
