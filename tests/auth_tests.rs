use common::TestApp;
use serde_json::json;

mod common;

#[sqlx::test]
async fn test_successful_login(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    // Create a unique user for this test
    let email = format!("login_success_{}@example.com", uuid::Uuid::new_v4());
    let password = "my_secure_password";
    app.create_user(&email, password).await;

    // Attempt login
    let response = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.get("access_token").is_some());
    assert!(body.get("refresh_token").is_some());
    assert_eq!(body["user_info"]["email"], email);
}

#[sqlx::test]
async fn test_login_invalid_email(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let response = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "some_password"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Depending on your error handling, it might be 400 or 401. Assuming 400 Bad Request or similar from WebError.
    // Let's assert it is not successful
    assert!(!response.status().is_success());
}

#[sqlx::test]
async fn test_login_invalid_password(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let email = format!("login_invalid_pwd_{}@example.com", uuid::Uuid::new_v4());
    let password = "correct_password";
    app.create_user(&email, password).await;

    let response = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({
            "email": email,
            "password": "wrong_password"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(!response.status().is_success());
}

#[sqlx::test]
async fn test_token_refresh(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let email = format!("refresh_{}@example.com", uuid::Uuid::new_v4());
    let password = "my_secure_password";
    app.create_user(&email, password).await;

    // First login
    let login_resp = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = login_resp.json().await.unwrap();
    let refresh_token = body["refresh_token"].as_str().unwrap();

    // Now refresh
    let refresh_resp = app
        .client
        .post(format!("{}/api/auth/refresh", app.base_url))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(refresh_resp.status().as_u16(), 200);
    let refresh_body: serde_json::Value = refresh_resp.json().await.unwrap();
    assert!(refresh_body.get("access_token").is_some());
    assert!(refresh_body.get("refresh_token").is_some());
}

#[sqlx::test]
async fn test_access_protected_route_without_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let response = app
        .client
        .get(format!("{}/api/auth/health", app.base_url))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 401);
}

#[sqlx::test]
async fn test_access_protected_route_with_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let email = format!("protected_{}@example.com", uuid::Uuid::new_v4());
    let password = "password";
    app.create_user(&email, password).await;

    let login_resp = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = login_resp.json().await.unwrap();
    let access_token = body["access_token"].as_str().unwrap();

    let response = app
        .client
        .get(format!("{}/api/auth/health", app.base_url))
        .bearer_auth(access_token)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
}

#[sqlx::test]
async fn test_logout(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let email = format!("logout_{}@example.com", uuid::Uuid::new_v4());
    let password = "password";
    app.create_user(&email, password).await;

    let login_resp = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = login_resp.json().await.unwrap();
    let refresh_token = body["refresh_token"].as_str().unwrap();

    // Perform logout
    let logout_resp = app
        .client
        .post(format!("{}/api/auth/logout", app.base_url))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(logout_resp.status().as_u16(), 200);

    // Refreshing should now fail
    let refresh_resp = app
        .client
        .post(format!("{}/api/auth/refresh", app.base_url))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert!(!refresh_resp.status().is_success());
}

#[sqlx::test]
async fn test_logout_all(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let email = format!("logout_all_{}@example.com", uuid::Uuid::new_v4());
    let password = "password";
    app.create_user(&email, password).await;

    let login_req = json!({"email": email, "password": password});

    // Login device 1
    let login1 = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&login_req)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    // Login device 2
    let login2 = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&login_req)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let access1 = login1["access_token"].as_str().unwrap();
    let refresh2 = login2["refresh_token"].as_str().unwrap();

    // Logout all from device 1
    let logout_all_resp = app
        .client
        .post(format!("{}/api/auth/logout-all", app.base_url))
        .bearer_auth(access1)
        .send()
        .await
        .unwrap();

    assert_eq!(logout_all_resp.status().as_u16(), 200);

    // Refreshing on device 2 should fail
    let refresh_resp = app
        .client
        .post(format!("{}/api/auth/refresh", app.base_url))
        .json(&json!({"refresh_token": refresh2}))
        .send()
        .await
        .unwrap();

    assert!(!refresh_resp.status().is_success());
}

#[sqlx::test]
async fn test_password_change(pool: sqlx::PgPool) {
    let app = TestApp::new(pool).await;

    let email = format!("pwd_change_{}@example.com", uuid::Uuid::new_v4());
    let old_password = "old_password";
    let new_password = "new_password";
    app.create_user(&email, old_password).await;

    // Login
    let login_resp = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({"email": email, "password": old_password}))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = login_resp.json().await.unwrap();
    let access_token = body["access_token"].as_str().unwrap();
    let refresh_token = body["refresh_token"].as_str().unwrap();

    // Change Password
    let change_pwd_resp = app
        .client
        .post(format!("{}/api/auth/password", app.base_url))
        .bearer_auth(access_token)
        .json(&json!({
            "old_password": old_password,
            "new_password": new_password
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(change_pwd_resp.status().as_u16(), 200);

    // Old refresh token shouldn't work anymore (because all sessions are revoked)
    let refresh_resp = app
        .client
        .post(format!("{}/api/auth/refresh", app.base_url))
        .json(&json!({"refresh_token": refresh_token}))
        .send()
        .await
        .unwrap();
    assert!(!refresh_resp.status().is_success());

    // Login with new password should work
    let new_login_resp = app
        .client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({"email": email, "password": new_password}))
        .send()
        .await
        .unwrap();

    assert_eq!(new_login_resp.status().as_u16(), 200);
}
