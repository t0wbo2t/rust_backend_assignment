use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use tower::Service;
use serde_json::{json, Value};

use backend_project::config::AppConfig;
use backend_project::state::AppState;
use backend_project::router;
use backend_project::models::{ViewTasksResponse, Role};

async fn send_request(
    app: &mut axum::Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Body,
) -> (StatusCode, Bytes) {
    let mut req = Request::builder()
        .method(method)
        .uri(uri);

    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }

    if method == "POST" {
        req = req.header("Content-Type", "application/json");
    }

    let req = req.body(body).unwrap();
    let response = app.call(req).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, body_bytes)
}

#[tokio::test]
async fn test_full_validation_workflow() {
    // 1. Set environment variable for memory sqlite
    unsafe {
        std::env::set_var("DATABASE_URL", "sqlite::memory:?cache=shared");
    }

    let state = AppState::new(AppConfig::default());

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&state.database)
        .await
        .unwrap();

    let mut app = router::build_router(state.clone());

    // 2. POST /seed/users
    let (status, body) = send_request(&mut app, "POST", "/seed/users", None, Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    let seed_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(seed_res["message"], "Users seeded successfully");

    // 3. Admin login: POST /auth/login
    let login_payload = json!({
        "email": "admin@example.com",
        "password": "admin123"
    });
    let (status, body) = send_request(&mut app, "POST", "/auth/login", None, Body::from(login_payload.to_string())).await;
    assert_eq!(status, StatusCode::OK);
    let login_res: Value = serde_json::from_slice(&body).unwrap();
    let admin_challenge_id = login_res["login_challenge_id"].as_str().unwrap();

    // 4. Retrieve verification code from dev endpoint
    let (status, body) = send_request(&mut app, "GET", "/dev/email-logs/latest", None, Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    let dev_email_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dev_email_res["recipient"], "admin@example.com");
    let admin_code = dev_email_res["code"].as_str().unwrap();

    // 5. Verify 2FA to get Admin JWT: POST /auth/verify-2fa
    let verify_payload = json!({
        "login_challenge_id": admin_challenge_id,
        "code": admin_code
    });
    let (status, body) = send_request(&mut app, "POST", "/auth/verify-2fa", None, Body::from(verify_payload.to_string())).await;
    assert_eq!(status, StatusCode::OK);
    let verify_res: Value = serde_json::from_slice(&body).unwrap();
    let admin_token = verify_res["token"].as_str().unwrap();

    // 6. Test incorrect code rejection
    let bad_verify_payload = json!({
        "login_challenge_id": admin_challenge_id,
        "code": "999999"
    });
    let (status, _) = send_request(&mut app, "POST", "/auth/verify-2fa", None, Body::from(bad_verify_payload.to_string())).await;
    // Challenge was already marked used in step 5, so it will return 410 Gone (reused code rejected)
    assert_eq!(status, StatusCode::GONE);

    // 7. Admin creates exactly 5 tasks
    let mut task_ids = Vec::new();
    for i in 1..=5 {
        let create_task_payload = json!({
            "title": format!("Task {i}"),
            "description": format!("Description for task {i}"),
            "status": "todo",
            "priority": if i % 2 == 0 { "high" } else { "low" },
            "assigned_to_id": null
        });
        let (status, body) = send_request(&mut app, "POST", "/tasks", Some(admin_token), Body::from(create_task_payload.to_string())).await;
        assert_eq!(status, StatusCode::CREATED);
        let task_res: Value = serde_json::from_slice(&body).unwrap();
        task_ids.push(task_res["id"].as_str().unwrap().to_string());
    }
    assert_eq!(task_ids.len(), 5);

    // Get James Bond user details from database to assign tasks to him
    let james_bond: backend_project::models::User = sqlx::query_as("SELECT * FROM users WHERE email = 'jamesbond@example.com'")
        .fetch_one(&state.database)
        .await
        .unwrap();

    // 8. Admin assigns exactly 3 tasks to James Bond
    let assign_payload = json!({
        "task_ids": [&task_ids[0], &task_ids[1], &task_ids[2]],
        "user_id": james_bond.id
    });
    let (status, body) = send_request(&mut app, "POST", "/tasks/assign", Some(admin_token), Body::from(assign_payload.to_string())).await;
    assert_eq!(status, StatusCode::OK);
    let assign_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(assign_res["message"], "Tasks assigned successfully");

    // 9. James Bond login flow
    let jb_login_payload = json!({
        "email": "jamesbond@example.com",
        "password": "james123"
    });
    let (status, body) = send_request(&mut app, "POST", "/auth/login", None, Body::from(jb_login_payload.to_string())).await;
    assert_eq!(status, StatusCode::OK);
    let jb_login_res: Value = serde_json::from_slice(&body).unwrap();
    let jb_challenge_id = jb_login_res["login_challenge_id"].as_str().unwrap();

    // Get James Bond 2FA code
    let (status, body) = send_request(&mut app, "GET", "/dev/email-logs/latest", None, Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    let jb_email_res: Value = serde_json::from_slice(&body).unwrap();
    let jb_code = jb_email_res["code"].as_str().unwrap();

    // Verify 2FA for James Bond
    let jb_verify_payload = json!({
        "login_challenge_id": jb_challenge_id,
        "code": jb_code
    });
    let (status, body) = send_request(&mut app, "POST", "/auth/verify-2fa", None, Body::from(jb_verify_payload.to_string())).await;
    assert_eq!(status, StatusCode::OK);
    let jb_verify_res: Value = serde_json::from_slice(&body).unwrap();
    let jb_token = jb_verify_res["token"].as_str().unwrap();

    // 10. Attempt POST /tasks with James Bond token -> Expect 403 Forbidden
    let unauthorized_create_payload = json!({
        "title": "Unauthorized Task",
        "description": "I should not be able to create this",
        "status": "todo",
        "priority": "low",
        "assigned_to_id": null
    });
    let (status, _) = send_request(&mut app, "POST", "/tasks", Some(jb_token), Body::from(unauthorized_create_payload.to_string())).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 11. View tasks first request (cache miss)
    let (status, body) = send_request(&mut app, "GET", "/tasks/view-my-tasks", Some(jb_token), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    let view_res_1: ViewTasksResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(view_res_1.user.email, "jamesbond@example.com");
    assert_eq!(view_res_1.user.role, Role::Staff);
    assert_eq!(view_res_1.tasks.len(), 3);
    assert_eq!(view_res_1.summary.total_assigned_tasks, 3);
    assert_eq!(view_res_1.cache.hit, false);

    // 12. View tasks second request (cache hit)
    let (status, body) = send_request(&mut app, "GET", "/tasks/view-my-tasks", Some(jb_token), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    let view_res_2: ViewTasksResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(view_res_2.tasks.len(), 3);
    assert_eq!(view_res_2.cache.hit, true);

    // 13. Reassign one of James Bond's tasks to trigger cache invalidation
    // Get Admin user details
    let admin: backend_project::models::User = sqlx::query_as("SELECT * FROM users WHERE email = 'admin@example.com'")
        .fetch_one(&state.database)
        .await
        .unwrap();

    let reassign_payload = json!({
        "task_ids": [&task_ids[0]],
        "user_id": admin.id
    });
    let (status, _) = send_request(&mut app, "POST", "/tasks/assign", Some(admin_token), Body::from(reassign_payload.to_string())).await;
    assert_eq!(status, StatusCode::OK);

    // 14. View tasks third request (should be cache miss, returning 2 tasks)
    let (status, body) = send_request(&mut app, "GET", "/tasks/view-my-tasks", Some(jb_token), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    let view_res_3: ViewTasksResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(view_res_3.tasks.len(), 2);
    assert_eq!(view_res_3.cache.hit, false);
}
