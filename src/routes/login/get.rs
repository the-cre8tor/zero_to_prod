use actix_web::{http::header::ContentType, web::Query, HttpResponse};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(query: Query<QueryParams>) -> HttpResponse {
    let error_html = match query.0.error {
        Some(message) => format!("<p><i>{message}</i></p>"),
        None => "".into(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta http-equiv="content-type" content="text/html; charset=utf-8">
                    <title>Login</title>
                </head>
                <body>
                    {error_html}
                    <form action="/login" method="post">
                        <label>Username
                            <input
                                type="text"
                                placeholder="Enter Username"
                                name="username"
                            >
                        </label>
                        <label>Password
                            <input
                                type="password"
                                placeholder="Enter Password"
                                name="password"
                            >
                        </label>
                        <button type="submit">Login</button>
                    </form>
                </body>
                </html>"#,
        ))
}
