use actix_web::{body::to_bytes, http::StatusCode, HttpResponse};
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

pub async fn save_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    http_response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    let (response_head, body) = http_response.into_parts();
    let body = to_bytes(body)
        .await
        .map_err(|error| anyhow::anyhow!("{}", error))?;

    let status_code: i16 = response_head.status().as_u16() as i16;

    let headers = {
        let mut header = Vec::with_capacity(response_head.headers().len());

        for (name, value) in response_head.headers().iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();

            let header_record = HeaderPairRecord { name, value };

            header.push(header_record);
        }

        header
    };

    // We need `.map_into_boxed_body` to go from
    // `HttpResponse<Bytes>` to `HttpResponse<BoxBody>`
    let http_response = response_head.set_body(body).map_into_boxed_body();

    Ok(http_response)
}
