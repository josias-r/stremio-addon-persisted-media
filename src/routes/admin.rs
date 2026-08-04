use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use base64::prelude::*;
use axum::http::HeaderMap;
use serde::Deserialize;
use maud::html;

use crate::routes::AppState;
use crate::routes::ui::base_layout;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(admin_dashboard))
        .route("/users", post(create_user))
        .route("/users/delete", post(delete_user))
}

fn check_auth(headers: &HeaderMap, state: &AppState) -> Result<(), Response> {
    let auth_header = headers.get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !auth_header.starts_with("Basic ") {
        return Err(unauthorized_response());
    }

    let b64 = &auth_header[6..];
    let decoded = match BASE64_STANDARD.decode(b64) {
        Ok(bytes) => String::from_utf8(bytes).unwrap_or_default(),
        Err(_) => return Err(unauthorized_response()),
    };

    let parts: Vec<&str> = decoded.splitn(2, ':').collect();
    if parts.len() != 2 || parts[0] != "admin" {
        return Err(unauthorized_response());
    }

    let password = parts[1];
    let expected_password = state.db.get_admin_password().unwrap_or_default();

    if password == expected_password {
        Ok(())
    } else {
        Err(unauthorized_response())
    }
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Basic realm=\"Admin Panel\"")],
        "Unauthorized",
    )
        .into_response()
}

async fn admin_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Response> {
    check_auth(&headers, &state)?;

    let history = state.db.get_watch_history().unwrap_or_default();
    let users = state.db.get_users().unwrap_or_default();

    let content = html! {
        div class="section" {
            h2 { "Users Management" }
            div class="card" style="margin-bottom: 2rem;" {
                h3 style="margin-top: 0; margin-bottom: 1rem;" { "Create User" }
                form method="POST" action="/admin/users" style="display: flex; gap: 1rem; align-items: center;" {
                    input type="text" name="username" placeholder="Username" required class="input-field" style="margin-bottom: 0; flex-grow: 1;" {}
                    button type="submit" class="btn" { "Create New User (API Key)" }
                }
            }

            h3 { "Active Users" }
            div class="card" style="padding: 0;" {
                table {
                    tr {
                        th { "Username" }
                        th { "API Key" }
                        th { "Created At" }
                        th { "Action" }
                    }
                    @for (username, api_key, created_at) in users {
                        tr {
                            td { (username) }
                            td { code { (api_key) } }
                            td { (created_at) }
                            td {
                                form method="POST" action="/admin/users/delete" style="margin: 0;" {
                                    input type="hidden" name="api_key" value=(api_key) {}
                                    button type="submit" class="btn btn-danger" style="padding: 0.4rem 0.8rem; font-size: 0.9rem;" { "Delete" }
                                }
                            }
                        }
                    }
                }
            }
        }

        div class="section" {
            h2 { "Watch History" }
            div class="card" style="padding: 0;" {
                table {
                    tr {
                        th { "API Key" }
                        th { "Item ID" }
                        th { "Type" }
                        th { "Last Watched" }
                    }
                    @for (api_key, item_id, item_type, last_watched) in history {
                        tr {
                            td { code { (api_key) } }
                            td { (item_id) }
                            td { (item_type) }
                            td { (last_watched) }
                        }
                    }
                }
            }
        }
    };

    Ok(base_layout("Admin Panel", "Manage users and server activity", content))
}

#[derive(Deserialize)]
struct CreateUserForm {
    username: String,
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Form(form): axum::extract::Form<CreateUserForm>,
) -> Result<impl IntoResponse, Response> {
    check_auth(&headers, &state)?;

    match state.db.create_user(&form.username) {
        Ok(api_key) => {
            let content = html! {
                div class="section" {
                    div class="card" style="text-align: center;" {
                        h2 style="color: #10b981; border: none; margin-bottom: 0;" { "User Created Successfully" }
                        p style="font-size: 1.2rem; margin: 1rem 0;" {
                            "Username: " strong { (form.username) }
                        }
                        div style="background: rgba(0,0,0,0.2); padding: 1rem; border-radius: 8px; margin: 2rem 0;" {
                            p style="margin: 0 0 0.5rem 0; color: var(--text-muted);" { "API Key:" }
                            code style="font-size: 1.5rem; color: #a78bfa;" { (api_key) }
                        }
                        a href="/admin" class="btn" { "Back to Admin Panel" }
                    }
                }
            };
            Ok(base_layout("Success", "", content))
        }
        Err(e) => {
            let content = html! {
                div class="section" {
                    div class="card" style="text-align: center;" {
                        h2 style="color: #ef4444; border: none;" { "Error" }
                        p { "Failed to create user: " (e) }
                        a href="/admin" class="btn" { "Back to Admin Panel" }
                    }
                }
            };
            Ok(base_layout("Error", "", content))
        }
    }
}

#[derive(Deserialize)]
struct DeleteUserForm {
    api_key: String,
}

async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Form(form): axum::extract::Form<DeleteUserForm>,
) -> Result<axum::response::Redirect, Response> {
    check_auth(&headers, &state)?;
    let _ = state.db.delete_user(&form.api_key);
    Ok(axum::response::Redirect::to("/admin"))
}
