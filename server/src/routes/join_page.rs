use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::state::AppState;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[derive(Debug, Deserialize)]
pub struct JoinPageQuery {
    pub token: Option<String>,
}

fn detect_os(user_agent: &str) -> &'static str {
    let ua = user_agent.to_lowercase();
    if ua.contains("windows") {
        "windows"
    } else if ua.contains("mac os x") || ua.contains("macos") {
        "macos-arm64"
    } else if ua.contains("linux") {
        "linux"
    } else {
        "unknown"
    }
}

fn extract_host_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h).to_string())
}

fn render_error_page(title: &str, message: &str) -> Html<String> {
    Html(format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ background: #0d1117; color: #e6edf3; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; }}
.container {{ text-align: center; padding: 2rem; max-width: 480px; }}
h1 {{ font-size: 1.5rem; margin-bottom: 1rem; }}
p {{ color: #8b949e; line-height: 1.6; }}
.icon {{ font-size: 3rem; margin-bottom: 1rem; }}
</style>
</head>
<body>
<div class="container">
<div class="icon">&#9888;</div>
<h1>{title}</h1>
<p>{message}</p>
</div>
</body>
</html>"##,
        title = title,
        message = message,
    ))
}

fn render_join_page(
    ring_name: &str,
    member_count: i64,
    role: &str,
    token_type: &str,
    token: &str,
    host_ip: Option<&str>,
    detected_os: &str,
) -> Html<String> {
    let download_section = match std::env::var("RING_DOWNLOAD_URL") {
        Ok(base_url) => {
            let platforms = [
                ("windows", "Windows", "ring-server-windows-amd64.zip", detected_os == "windows"),
                ("macos-arm64", "macOS (Apple Silicon)", "ring-server-macos-arm64.tar.gz", detected_os == "macos-arm64"),
                ("macos-amd64", "macOS (Intel)", "ring-server-macos-amd64.tar.gz", detected_os == "macos-amd64"),
                ("linux", "Linux", "ring-server-linux-amd64.tar.gz", detected_os == "linux"),
            ];
            let buttons = platforms.iter().map(|(_, label, filename, detected)| {
                let bg = if *detected { "#238636" } else { "#21262d" };
                let border = if *detected { "#2ea043" } else { "#30363d" };
                let badge = if *detected { r##"<span style="font-size:0.7rem;background:#238636;padding:2px 8px;border-radius:9999px;margin-left:8px;">Recommended</span>"## } else { "" };
                format!(
                    r##"<a href="{base}/{file}" style="display:flex;align-items:center;justify-content:center;gap:8px;background:{bg};border:1px solid {border};color:#e6edf3;padding:10px 20px;border-radius:6px;text-decoration:none;font-size:0.9rem;margin-bottom:8px;width:100%;max-width:340px;">{label} {badge}</a>"##,
                    base = base_url.trim_end_matches('/'),
                    file = filename,
                    bg = bg,
                    border = border,
                    label = label,
                    badge = badge,
                )
            }).collect::<Vec<_>>().join("\n");
            format!(
                r##"<h2 class="section-title">Download Ring</h2><div class="downloads">{buttons}</div>"##
            )
        }
        Err(_) => {
            r##"<div class="info-box" style="text-align:center;"><p style="color:#8b949e;">请联系 Ring 管理员获取安装包</p></div>"##.to_string()
        }
    };

    let creator_ip_param = match host_ip {
        Some(ip) => format!("&creator_ip={}", ip),
        None => String::new(),
    };

    Html(format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Join {ring_name} - Ring</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ background: #0d1117; color: #e6edf3; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; }}
.container {{ max-width: 640px; margin: 0 auto; padding: 2rem 1rem; }}
h1 {{ font-size: 1.8rem; margin-bottom: 0.5rem; }}
.subtitle {{ color: #8b949e; margin-bottom: 2rem; }}
.info-box {{ background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 1.25rem; margin-bottom: 2rem; }}
.info-row {{ display: flex; justify-content: space-between; padding: 6px 0; }}
.info-label {{ color: #8b949e; }}
.info-value {{ color: #e6edf3; font-weight: 500; }}
.section-title {{ font-size: 1.1rem; margin-bottom: 1rem; border-bottom: 1px solid #21262d; padding-bottom: 0.5rem; }}
.downloads {{ display: flex; flex-direction: column; align-items: center; margin-bottom: 2rem; }}
.steps {{ background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 1.25rem; margin-bottom: 2rem; }}
.step {{ display: flex; gap: 12px; margin-bottom: 12px; align-items: flex-start; }}
.step:last-child {{ margin-bottom: 0; }}
.step-num {{ background: #58a6ff; color: #0d1117; width: 24px; height: 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700; font-size: 0.8rem; flex-shrink: 0; }}
.step-text {{ color: #c9d1d9; line-height: 1.5; }}
.continue-btn {{ display: block; width: 100%; max-width: 340px; margin: 0 auto; background: #58a6ff; color: #0d1117; text-align: center; padding: 12px 24px; border-radius: 6px; text-decoration: none; font-weight: 600; font-size: 1rem; }}
.continue-btn:hover {{ background: #79c0ff; }}
</style>
</head>
<body>
<div class="container">
<h1>Join <span style="color:#58a6ff;">{ring_name}</span></h1>
<p class="subtitle">You have been invited to join this Ring</p>

<div class="info-box">
<div class="info-row"><span class="info-label">Ring</span><span class="info-value">{ring_name}</span></div>
<div class="info-row"><span class="info-label">Members</span><span class="info-value">{member_count}</span></div>
<div class="info-row"><span class="info-label">Your role</span><span class="info-value">{role}</span></div>
<div class="info-row"><span class="info-label">Invite type</span><span class="info-value">{token_type}</span></div>
</div>

{download_section}

<div class="steps">
<div class="step"><div class="step-num">1</div><div class="step-text">Download and install Ring for your platform</div></div>
<div class="step"><div class="step-num">2</div><div class="step-text">Start Ring server, it will open on localhost:7420</div></div>
<div class="step"><div class="step-num">3</div><div class="step-text">Click the button below to join the Ring</div></div>
</div>

<a href="http://localhost:7420/ring/join?token={token}{creator_ip_param}" class="continue-btn">Continue to Join</a>
</div>
</body>
</html>"##,
        ring_name = html_escape(ring_name),
        member_count = member_count,
        role = html_escape(role),
        token_type = html_escape(token_type),
        download_section = download_section,
        token = html_escape(token),
        creator_ip_param = creator_ip_param,
    ))
}

pub async fn join_page_handler(
    State(state): State<AppState>,
    Query(query): Query<JoinPageQuery>,
    headers: HeaderMap,
) -> Response {
    let token = match query.token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return render_error_page(
                "Missing Invite Token",
                "No invite token was provided. Please use a valid invite link.",
            )
            .into_response();
        }
    };

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let detected_os = detect_os(user_agent);

    match crate::services::invite::verify_join_token(&state, &token).await {
        Ok(info) => {
            if info.valid {
                let host_ip = extract_host_ip(&headers);
                render_join_page(
                    info.ring_name.as_deref().unwrap_or("Unknown"),
                    info.member_count.unwrap_or(0),
                    info.role.as_deref().unwrap_or("member"),
                    info.token_type.as_deref().unwrap_or("open"),
                    &token,
                    host_ip.as_deref(),
                    detected_os,
                )
                .into_response()
            } else {
                let reason = info.reason.as_deref().unwrap_or("unknown");
                let (title, message) = if reason.contains("expired") {
                    (
                        "Invite Expired".to_string(),
                        "This invite link has expired. Please request a new one from the Ring admin.".to_string(),
                    )
                } else if reason.contains("revoked") {
                    (
                        "Invite Revoked".to_string(),
                        "This invite link has been revoked by the Ring admin.".to_string(),
                    )
                } else {
                    (
                        "Invalid Invite".to_string(),
                        format!("This invite is invalid: {}", reason),
                    )
                };
                render_error_page(&title, &message).into_response()
            }
        }
        Err(_) => render_error_page(
            "Server Error",
            "Something went wrong while verifying the invite. Please try again later.",
        )
        .into_response(),
    }
}
