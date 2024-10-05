//! tests/health_check.rs

use crate::helpers::TestApp;
use reqwest::Client;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = TestApp::spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .get(format!("{}/health-check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
