use std::collections::HashSet;

use reqwest::header::HeaderValue;

use crate::helpers::TestApp;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = TestApp::spawn_app().await;

    // Act
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });

    let response = app.post_login(&login_body).await;
    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    // Assert
    TestApp::assert_is_redirect_to(&response, "/login");

    let cookies: HashSet<&HeaderValue> = response
        .headers()
        .get_all("Set-Cookie")
        .into_iter()
        .collect();

    assert!(cookies.contains(&HeaderValue::from_str("_flash=Authentication failed").unwrap()));

    let flash_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == "_flash")
        .unwrap();

    assert_eq!(flash_cookie.value(), "Authentication failed");
}
