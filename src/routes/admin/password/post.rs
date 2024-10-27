//! src/routes/admin/password/post.rs
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use redact::Secret;
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    routes::admin::dashboard::get_username,
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
    pool: Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session.get_user_id().map_err(error_500)?;

    if user_id.is_none() {
        return Ok(see_other("/login"));
    }

    let user_id = user_id.unwrap();

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();

        return Ok(see_other("/admin/password"));
    }

    let username = get_username(user_id, &pool).await.map_err(error_500)?;

    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };

    if let Err(error) = validate_credentials(credentials, &pool).await {
        return match error {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(error_500(error).into()),
        };
    }

    todo!()
}
