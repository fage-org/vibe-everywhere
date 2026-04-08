use std::{
    fs,
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use keyring::{Entry, Error as KeyringError};
use rand::{RngCore, rngs::OsRng};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tauri_plugin_notification::NotificationExt;
use tiny_http::{Header, Method, Request, Response, Server};

const KEYRING_SERVICE: &str = "engineering.vibe.app.next";
const KEYRING_USER: &str = "desktop-credentials";
const LOOPBACK_TIMEOUT: Duration = Duration::from_secs(300);
const LOOPBACK_BIND: &str = "127.0.0.1:0";

#[derive(Default)]
struct DesktopAuthState {
    attempt: Mutex<Option<AccountLinkAttempt>>,
}

struct AccountLinkAttempt {
    attempt_id: String,
    status: Arc<Mutex<AccountLinkAttemptState>>,
    shutdown: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

#[derive(Clone)]
enum AccountLinkAttemptState {
    Pending,
    Completed,
    Failed(String),
    Canceled,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BeginAccountLinkCallbackResponse {
    attempt_id: String,
    browser_url: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum AccountLinkCallbackStatusResponse {
    Pending,
    Completed,
    Failed { error: String },
    Canceled,
    NotFound,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CallbackBody {
    state: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserPageConfig<'a> {
    server_url: &'a str,
    public_key: &'a str,
    deep_link: &'a str,
    state: &'a str,
}

#[tauri::command]
fn secure_store_get_credentials() -> Result<Option<String>, String> {
    let entry = credentials_entry()?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
fn secure_store_set_credentials(value: String) -> Result<(), String> {
    let entry = credentials_entry()?;
    entry.set_password(&value).map_err(|error| error.to_string())
}

#[tauri::command]
fn secure_store_clear_credentials() -> Result<(), String> {
    let entry = credentials_entry()?;
    match entry.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    webbrowser::open(&url)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_text_file_dialog(title: String) -> Result<Option<String>, String> {
    let path = FileDialog::new()
        .set_title(title)
        .add_filter("Text", &["txt", "md", "key", "json"])
        .pick_file();

    let Some(path) = path else {
        return Ok(None);
    };

    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    Ok(Some(contents))
}

#[tauri::command]
fn save_text_file_dialog(
    title: String,
    suggested_name: String,
    contents: String,
) -> Result<Option<String>, String> {
    let path = FileDialog::new()
        .set_title(title)
        .set_file_name(suggested_name)
        .add_filter("Text", &["txt", "md", "key", "json"])
        .save_file();

    let Some(path) = path else {
        return Ok(None);
    };

    fs::write(&path, contents)
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))?;
    Ok(Some(path.display().to_string()))
}

#[tauri::command]
fn show_desktop_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn begin_account_link_callback(
    state: tauri::State<'_, DesktopAuthState>,
    server_url: String,
    public_key: String,
    deep_link: String,
) -> Result<BeginAccountLinkCallbackResponse, String> {
    start_account_link_callback(&state, server_url, public_key, deep_link, LOOPBACK_TIMEOUT)
}

#[tauri::command]
fn get_account_link_callback_status(
    state: tauri::State<'_, DesktopAuthState>,
    attempt_id: String,
) -> Result<AccountLinkCallbackStatusResponse, String> {
    get_account_link_callback_status_for_attempt(&state, &attempt_id)
}

#[tauri::command]
fn cancel_account_link_callback(
    state: tauri::State<'_, DesktopAuthState>,
    attempt_id: String,
) -> Result<(), String> {
    cancel_account_link_callback_for_attempt(&state, &attempt_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DesktopAuthState::default())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            secure_store_get_credentials,
            secure_store_set_credentials,
            secure_store_clear_credentials,
            open_external_url,
            open_text_file_dialog,
            save_text_file_dialog,
            show_desktop_notification,
            begin_account_link_callback,
            get_account_link_callback_status,
            cancel_account_link_callback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running vibe-app-tauri desktop shell");
}

fn credentials_entry() -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|error| error.to_string())
}

fn start_account_link_callback(
    auth_state: &DesktopAuthState,
    server_url: String,
    public_key: String,
    deep_link: String,
    timeout: Duration,
) -> Result<BeginAccountLinkCallbackResponse, String> {
    let mut guard = auth_state
        .attempt
        .lock()
        .map_err(|_| "Failed to lock desktop auth state".to_string())?;

    if let Some(previous) = guard.take() {
        cancel_attempt(previous, AccountLinkAttemptState::Canceled);
    }

    let attempt_id = random_token(16);
    let callback_state = random_token(32);
    let listener = TcpListener::bind(LOOPBACK_BIND)
        .map_err(|error| format!("Failed to bind localhost auth listener: {error}"))?;
    let browser_url = format!(
        "http://127.0.0.1:{}/",
        listener
            .local_addr()
            .map_err(|error| format!("Failed to read listener address: {error}"))?
            .port()
    );

    let status = Arc::new(Mutex::new(AccountLinkAttemptState::Pending));
    let shutdown = Arc::new(AtomicBool::new(false));

    let thread_status = Arc::clone(&status);
    let thread_shutdown = Arc::clone(&shutdown);
    let thread_browser_url = browser_url.clone();
    let thread_server_url = server_url.clone();
    let thread_public_key = public_key.clone();
    let thread_deep_link = deep_link.clone();
    let thread_state = callback_state.clone();

    let join_handle = thread::spawn(move || {
        let server = match Server::from_listener(listener, None) {
            Ok(server) => server,
            Err(error) => {
                update_attempt_state(
                    &thread_status,
                    AccountLinkAttemptState::Failed(format!(
                        "Failed to start localhost auth listener: {error}"
                    )),
                );
                return;
            }
        };

        let deadline = Instant::now() + timeout;
        while !thread_shutdown.load(Ordering::Relaxed) && Instant::now() < deadline {
            match server.recv_timeout(Duration::from_millis(250)) {
                Ok(Some(request)) => handle_request(
                    request,
                    &thread_status,
                    &thread_shutdown,
                    &thread_server_url,
                    &thread_public_key,
                    &thread_deep_link,
                    &thread_state,
                ),
                Ok(None) => {}
                Err(error) => {
                    update_attempt_state(
                        &thread_status,
                        AccountLinkAttemptState::Failed(format!(
                            "Loopback listener error: {error}"
                        )),
                    );
                    thread_shutdown.store(true, Ordering::Relaxed);
                }
            }
        }

        if !thread_shutdown.load(Ordering::Relaxed) {
            update_attempt_state(
                &thread_status,
                AccountLinkAttemptState::Failed(format!(
                    "Authorization timed out after {} seconds",
                    timeout.as_secs()
                )),
            );
            thread_shutdown.store(true, Ordering::Relaxed);
        }

        let _ = server.unblock();
    });

    *guard = Some(AccountLinkAttempt {
        attempt_id: attempt_id.clone(),
        status,
        shutdown,
        join_handle: Some(join_handle),
    });

    Ok(BeginAccountLinkCallbackResponse {
        attempt_id,
        browser_url: thread_browser_url,
    })
}

fn get_account_link_callback_status_for_attempt(
    auth_state: &DesktopAuthState,
    attempt_id: &str,
) -> Result<AccountLinkCallbackStatusResponse, String> {
    let mut guard = auth_state
        .attempt
        .lock()
        .map_err(|_| "Failed to lock desktop auth state".to_string())?;

    let Some(attempt) = guard.as_mut() else {
        return Ok(AccountLinkCallbackStatusResponse::NotFound);
    };

    if attempt.attempt_id != attempt_id {
        return Ok(AccountLinkCallbackStatusResponse::NotFound);
    }

    let snapshot = snapshot_status(&attempt.status)?;
    if !matches!(snapshot, AccountLinkCallbackStatusResponse::Pending) {
        if let Some(handle) = attempt.join_handle.take() {
            let _ = handle.join();
        }
    }

    Ok(snapshot)
}

fn cancel_account_link_callback_for_attempt(
    auth_state: &DesktopAuthState,
    attempt_id: &str,
) -> Result<(), String> {
    let mut guard = auth_state
        .attempt
        .lock()
        .map_err(|_| "Failed to lock desktop auth state".to_string())?;

    let Some(current) = guard.take() else {
        return Ok(());
    };

    if current.attempt_id != attempt_id {
        *guard = Some(current);
        return Ok(());
    }

    cancel_attempt(current, AccountLinkAttemptState::Canceled);
    Ok(())
}

fn cancel_attempt(mut attempt: AccountLinkAttempt, next_state: AccountLinkAttemptState) {
    update_attempt_state(&attempt.status, next_state);
    attempt.shutdown.store(true, Ordering::Relaxed);
    if let Some(handle) = attempt.join_handle.take() {
        let _ = handle.join();
    }
}

fn snapshot_status(
    status: &Arc<Mutex<AccountLinkAttemptState>>,
) -> Result<AccountLinkCallbackStatusResponse, String> {
    let guard = status
        .lock()
        .map_err(|_| "Failed to read auth attempt state".to_string())?;
    Ok(match &*guard {
        AccountLinkAttemptState::Pending => AccountLinkCallbackStatusResponse::Pending,
        AccountLinkAttemptState::Completed => AccountLinkCallbackStatusResponse::Completed,
        AccountLinkAttemptState::Failed(error) => AccountLinkCallbackStatusResponse::Failed {
            error: error.clone(),
        },
        AccountLinkAttemptState::Canceled => AccountLinkCallbackStatusResponse::Canceled,
    })
}

fn update_attempt_state(status: &Arc<Mutex<AccountLinkAttemptState>>, next: AccountLinkAttemptState) {
    if let Ok(mut guard) = status.lock() {
        if matches!(&*guard, AccountLinkAttemptState::Pending) {
            *guard = next;
        }
    }
}

fn handle_request(
    mut request: Request,
    status: &Arc<Mutex<AccountLinkAttemptState>>,
    shutdown: &Arc<AtomicBool>,
    server_url: &str,
    public_key: &str,
    deep_link: &str,
    callback_state: &str,
) {
    if request
        .remote_addr()
        .map(|addr| !addr.ip().is_loopback())
        .unwrap_or(false)
    {
        let _ = request.respond(html_response("Loopback access only", 403));
        return;
    }

    let path = request
        .url()
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();

    if request.method() == &Method::Get && (path == "/" || path == "/index.html") {
        let _ = request.respond(loopback_page_response(
            server_url,
            public_key,
            deep_link,
            callback_state,
        ));
        return;
    }

    if request.method() == &Method::Get && path == "/favicon.ico" {
        let _ = request.respond(Response::empty(204));
        return;
    }

    if request.method() == &Method::Post && path == "/callback" {
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            let _ = request.respond(html_response("Invalid callback payload", 400));
            return;
        }

        let parsed: CallbackBody = match serde_json::from_str(&body) {
            Ok(parsed) => parsed,
            Err(_) => {
                let _ = request.respond(html_response("Invalid callback payload", 400));
                return;
            }
        };

        if parsed.state != callback_state {
            let _ = request.respond(html_response("State mismatch", 400));
            return;
        }

        update_attempt_state(status, AccountLinkAttemptState::Completed);
        shutdown.store(true, Ordering::Relaxed);
        let _ = request.respond(html_response(
            "Authorization completed. You can return to Vibe Desktop Next.",
            200,
        ));
        return;
    }

    let _ = request.respond(html_response("Not found", 404));
}

fn loopback_page_response(
    server_url: &str,
    public_key: &str,
    deep_link: &str,
    callback_state: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let config = BrowserPageConfig {
        server_url,
        public_key,
        deep_link,
        state: callback_state,
    };
    let config_json = serde_json::to_string(&config)
        .unwrap_or_else(|_| "{}".to_string())
        .replace("</", "<\\/");

    let html = format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Vibe Desktop Next Auth</title>
    <style>
      :root {{
        color-scheme: dark;
        font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
        background: #07131d;
        color: #eff8f3;
      }}
      body {{
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
        background:
          radial-gradient(circle at top, rgba(123, 226, 196, 0.16), transparent 32%),
          linear-gradient(180deg, #07131d 0%, #02070b 100%);
      }}
      main {{
        width: min(720px, calc(100vw - 32px));
        padding: 28px;
        border-radius: 24px;
        border: 1px solid rgba(123, 226, 196, 0.22);
        background: rgba(9, 24, 34, 0.9);
        box-shadow: 0 24px 80px rgba(0, 0, 0, 0.32);
      }}
      h1, p {{ margin: 0; }}
      h1 {{ font-size: clamp(2rem, 4vw, 2.8rem); line-height: 0.96; }}
      .eyebrow {{
        margin-bottom: 10px;
        text-transform: uppercase;
        letter-spacing: 0.18em;
        font-size: 0.72rem;
        color: #7be2c4;
      }}
      .stack {{ display: grid; gap: 18px; }}
      .panel {{
        display: grid;
        gap: 12px;
        padding: 16px;
        border-radius: 18px;
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.08);
      }}
      code {{
        display: block;
        padding: 12px 14px;
        border-radius: 14px;
        background: rgba(243, 207, 140, 0.08);
        border: 1px dashed rgba(243, 207, 140, 0.22);
        color: #f3cf8c;
        overflow-wrap: anywhere;
      }}
      button {{
        width: fit-content;
        padding: 10px 14px;
        border-radius: 999px;
        border: 1px solid rgba(123, 226, 196, 0.28);
        background: rgba(123, 226, 196, 0.14);
        color: #eff8f3;
        cursor: pointer;
      }}
      #status {{ color: rgba(239, 248, 243, 0.76); line-height: 1.6; }}
    </style>
  </head>
  <body>
    <main class="stack">
      <div>
        <p class="eyebrow">Desktop auth callback</p>
        <h1>Authorization is waiting for your mobile approval</h1>
      </div>
      <p id="status">Keep this page open while you scan the request in the mobile app. The desktop app will finish sign-in as soon as the callback completes.</p>
      <section class="panel">
        <strong>Account link</strong>
        <code id="deep-link"></code>
        <button id="copy-link" type="button">Copy link</button>
      </section>
    </main>
    <script>
      const config = {config_json};
      const statusEl = document.getElementById("status");
      const deepLinkEl = document.getElementById("deep-link");
      deepLinkEl.textContent = config.deepLink;
      document.getElementById("copy-link").addEventListener("click", async () => {{
        try {{
          await navigator.clipboard.writeText(config.deepLink);
          statusEl.textContent = "Link copied. Approve the request in the mobile app, then return to Vibe Desktop Next.";
        }} catch (_error) {{
          statusEl.textContent = "Copy failed. You can still use the link shown on this page.";
        }}
      }});

      async function pollAuthorization() {{
        try {{
          const response = await fetch(`${{config.serverUrl}}/v1/auth/account/request`, {{
            method: "POST",
            headers: {{ "Content-Type": "application/json" }},
            body: JSON.stringify({{ publicKey: config.publicKey }}),
          }});

          if (!response.ok) {{
            throw new Error(`Remote auth request failed with ${{response.status}}`);
          }}

          const payload = await response.json();
          if (payload.state === "authorized") {{
            const callbackResponse = await fetch("/callback", {{
              method: "POST",
              headers: {{ "Content-Type": "application/json" }},
              body: JSON.stringify({{ state: config.state }}),
            }});

            if (!callbackResponse.ok) {{
              throw new Error("Desktop callback was rejected");
            }}

            statusEl.textContent = "Authorization completed. Return to Vibe Desktop Next.";
            return;
          }}

          statusEl.textContent = "Waiting for mobile approval...";
          window.setTimeout(pollAuthorization, 1000);
        }} catch (error) {{
          statusEl.textContent = `Auth check failed: ${{error.message || error}}`;
          window.setTimeout(pollAuthorization, 2000);
        }}
      }}

      pollAuthorization();
    </script>
  </body>
</html>"#,
        config_json = config_json
    );

    html_response_with_type(html, 200, "text/html; charset=UTF-8")
}

fn html_response(body: &str, status_code: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    html_response_with_type(body.to_string(), status_code, "text/plain; charset=UTF-8")
}

fn html_response_with_type(
    body: String,
    status_code: u16,
    content_type: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let response = Response::from_string(body).with_status_code(status_code);
    response.with_header(
        Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap(),
    )
}

fn random_token(num_bytes: usize) -> String {
    let mut bytes = vec![0u8; num_bytes];
    OsRng.fill_bytes(&mut bytes);
    base64_url(&bytes)
}

fn base64_url(bytes: &[u8]) -> String {
    let mut output = String::new();
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | bytes[index + 2] as u32;
        output.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        output.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        output.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        output.push(ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    match bytes.len() - index {
        1 => {
            let chunk = (bytes[index] as u32) << 16;
            output.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            output.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        }
        2 => {
            let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
            output.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            output.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            output.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        }
        _ => {}
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{
        AccountLinkAttemptState, AccountLinkCallbackStatusResponse, DesktopAuthState, base64_url,
        cancel_account_link_callback_for_attempt, get_account_link_callback_status_for_attempt,
        loopback_page_response, snapshot_status, start_account_link_callback, update_attempt_state,
    };
    use std::{
        io::{Read, Write},
        net::TcpStream,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    fn http_request(method: &str, url: &str, body: Option<&str>) -> (u16, String) {
        let without_scheme = url
            .strip_prefix("http://")
            .expect("loopback test helper only supports http urls");
        let (host_port, path) = without_scheme
            .split_once('/')
            .unwrap_or((without_scheme, ""));
        let path = format!("/{}", path);
        let payload = body.unwrap_or("");
        let mut request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {host_port}\r\nConnection: close\r\n"
        );
        if body.is_some() {
            request.push_str("Content-Type: application/json\r\n");
            request.push_str(&format!("Content-Length: {}\r\n", payload.len()));
        }
        request.push_str("\r\n");
        request.push_str(payload);

        let mut stream = TcpStream::connect(host_port).expect("should connect to loopback server");
        stream
            .write_all(request.as_bytes())
            .expect("should send loopback request");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("should read loopback response");

        let (head, body) = response
            .split_once("\r\n\r\n")
            .expect("response should contain headers");
        let status = head
            .split_whitespace()
            .nth(1)
            .expect("status code should exist")
            .parse::<u16>()
            .expect("status code should parse");

        (status, body.to_string())
    }

    fn extract_state_from_loopback_page(html: &str) -> String {
        let marker = "\"state\":\"";
        let start = html
            .find(marker)
            .expect("loopback page should embed auth state")
            + marker.len();
        let remainder = &html[start..];
        let end = remainder
            .find('"')
            .expect("loopback page should terminate auth state");
        remainder[..end].to_string()
    }

    fn extract_host_port(url: &str) -> String {
        url.strip_prefix("http://")
            .expect("loopback test helper only supports http urls")
            .trim_end_matches('/')
            .to_string()
    }

    #[test]
    fn base64_url_encodes_without_padding() {
        assert_eq!(base64_url(&[0xff]), "_w");
        assert_eq!(base64_url(&[0xff, 0xee]), "_-4");
        assert_eq!(base64_url(&[0xff, 0xee, 0xdd]), "_-7d");
    }

    #[test]
    fn completed_state_does_not_revert_back_to_pending() {
        let status = Arc::new(Mutex::new(AccountLinkAttemptState::Pending));

        update_attempt_state(&status, AccountLinkAttemptState::Completed);
        update_attempt_state(&status, AccountLinkAttemptState::Pending);

        let snapshot = snapshot_status(&status).expect("snapshot should succeed");
        assert!(matches!(
            snapshot,
            super::AccountLinkCallbackStatusResponse::Completed
        ));
    }

    #[test]
    fn loopback_page_embeds_callback_configuration() {
        let response = loopback_page_response(
            "https://api.cluster-fluster.com",
            "public-key-demo",
            "vibe:///account?abc",
            "state-demo",
        );
        let html = String::from_utf8(response.into_reader().into_inner())
            .expect("loopback page should be valid utf-8");

        assert!(html.contains("vibe:///account?abc"));
        assert!(html.contains("public-key-demo"));
        assert!(html.contains("state-demo"));
        assert!(html.contains("Copy link"));
    }

    #[test]
    fn loopback_page_escapes_embedded_script_breakout_sequences() {
        let response = loopback_page_response(
            "https://api.cluster-fluster.com",
            "public-key-demo",
            "vibe:///account?</script><script>alert(1)</script>",
            "state-demo",
        );
        let html = String::from_utf8(response.into_reader().into_inner())
            .expect("loopback page should be valid utf-8");

        assert!(!html.contains("</script><script>alert(1)</script>"));
        assert!(html.contains("<\\/script><script>alert(1)<\\/script>"));
    }

    #[test]
    fn loopback_attempt_rejects_wrong_state_before_accepting_the_active_attempt() {
        let auth_state = DesktopAuthState::default();
        let attempt = start_account_link_callback(
            &auth_state,
            "https://api.cluster-fluster.com".to_string(),
            "public-key-demo".to_string(),
            "vibe:///account?abc".to_string(),
            Duration::from_secs(5),
        )
        .expect("attempt should start");

        let (page_status, html) = http_request("GET", &attempt.browser_url, None);
        assert_eq!(page_status, 200);
        let callback_state = extract_state_from_loopback_page(&html);

        let (wrong_status, wrong_body) = http_request(
            "POST",
            &format!("{}callback", attempt.browser_url),
            Some(r#"{"state":"wrong-state"}"#),
        );
        assert_eq!(wrong_status, 400);
        assert!(wrong_body.contains("State mismatch"));
        assert!(matches!(
            get_account_link_callback_status_for_attempt(&auth_state, &attempt.attempt_id)
                .expect("status should load"),
            AccountLinkCallbackStatusResponse::Pending
        ));

        let (ok_status, ok_body) = http_request(
            "POST",
            &format!("{}callback", attempt.browser_url),
            Some(&format!(r#"{{"state":"{callback_state}"}}"#)),
        );
        assert_eq!(ok_status, 200);
        assert!(ok_body.contains("Authorization completed"));
        assert!(matches!(
            get_account_link_callback_status_for_attempt(&auth_state, &attempt.attempt_id)
                .expect("status should load"),
            AccountLinkCallbackStatusResponse::Completed
        ));
    }

    #[test]
    fn starting_a_second_attempt_replaces_the_previous_attempt() {
        let auth_state = DesktopAuthState::default();
        let first = start_account_link_callback(
            &auth_state,
            "https://api.cluster-fluster.com".to_string(),
            "public-key-demo".to_string(),
            "vibe:///account?first".to_string(),
            Duration::from_secs(5),
        )
        .expect("first attempt should start");
        let second = start_account_link_callback(
            &auth_state,
            "https://api.cluster-fluster.com".to_string(),
            "public-key-demo".to_string(),
            "vibe:///account?second".to_string(),
            Duration::from_secs(5),
        )
        .expect("second attempt should start");

        assert_ne!(first.attempt_id, second.attempt_id);
        assert!(matches!(
            get_account_link_callback_status_for_attempt(&auth_state, &first.attempt_id)
                .expect("status should load"),
            AccountLinkCallbackStatusResponse::NotFound
        ));
        assert!(matches!(
            get_account_link_callback_status_for_attempt(&auth_state, &second.attempt_id)
                .expect("status should load"),
            AccountLinkCallbackStatusResponse::Pending
        ));

        cancel_account_link_callback_for_attempt(&auth_state, &second.attempt_id)
            .expect("second attempt should cancel cleanly");
    }

    #[test]
    fn loopback_attempt_reports_timeout_failures() {
        let auth_state = DesktopAuthState::default();
        let attempt = start_account_link_callback(
            &auth_state,
            "https://api.cluster-fluster.com".to_string(),
            "public-key-demo".to_string(),
            "vibe:///account?timeout".to_string(),
            Duration::from_millis(50),
        )
        .expect("attempt should start");

        thread::sleep(Duration::from_millis(350));

        let status = get_account_link_callback_status_for_attempt(&auth_state, &attempt.attempt_id)
            .expect("status should load");
        match status {
            AccountLinkCallbackStatusResponse::Failed { error } => {
                assert!(error.contains("timed out"));
            }
            other => panic!("expected timeout failure, got {:?}", other),
        }
    }

    #[test]
    fn canceling_an_attempt_tears_down_the_listener_and_rejects_follow_up_callbacks() {
        let auth_state = DesktopAuthState::default();
        let attempt = start_account_link_callback(
            &auth_state,
            "https://api.cluster-fluster.com".to_string(),
            "public-key-demo".to_string(),
            "vibe:///account?cancel".to_string(),
            Duration::from_secs(5),
        )
        .expect("attempt should start");

        cancel_account_link_callback_for_attempt(&auth_state, &attempt.attempt_id)
            .expect("attempt should cancel cleanly");

        assert!(matches!(
            get_account_link_callback_status_for_attempt(&auth_state, &attempt.attempt_id)
                .expect("status should load"),
            AccountLinkCallbackStatusResponse::NotFound
        ));

        if let Ok(mut stream) = TcpStream::connect(extract_host_port(&attempt.browser_url)) {
            stream
                .set_read_timeout(Some(Duration::from_millis(300)))
                .expect("should set read timeout");
            stream
                .write_all(
                    b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
                )
                .expect("should write follow-up request");

            let mut response = String::new();
            let read_result = stream.read_to_string(&mut response);
            assert!(
                read_result.is_err() || !response.contains("200 OK"),
                "canceled listener should not continue serving successful auth pages"
            );
        }
    }
}
