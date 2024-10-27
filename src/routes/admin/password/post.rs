//! src/routes/admin/password/post.rs
use actix_web::{web, HttpResponse};
use redact::Secret;

use crate::{
    session_state::TypedSession,
    utils::{error_500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(error_500)?.is_none() {
        return Ok(see_other("/login"));
    };

    todo!()
}
