use actix_web::{http::StatusCode, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use super::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
          SELECT
            response_status_code,
            response_headers as "response_headers: Vec<HeaderPairRecord>",
            response_body
          FROM idempotency
          WHERE user_id = $1
          AND idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(record) = saved_response {
        let status_code = StatusCode::from_u16(record.response_status_code.try_into()?)?;

        let mut response = HttpResponse::build(status_code);

        for HeaderPairRecord { name, value } in record.response_headers {
            response.append_header((name, value));
        }

        let response = response.body(record.response_body);

        Ok(Some(response))
    } else {
        Ok(None)
    }
}
