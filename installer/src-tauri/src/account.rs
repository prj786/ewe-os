//! The ewe account during the install (2026-10-31): sign in to your own
//! Nextcloud from the live session, see what the account holds, and hand
//! the app password to the helper for the new user's login keyring — so
//! the first boot is signed in and brings everything back.
//!
//! The broker is the live system's `ewe-cloud` (Login Flow v2: the server's
//! own page, in the browser) and `ewe-conf` reads the backup; both run in a
//! private, throwaway config (0700 under `$XDG_RUNTIME_DIR`, a RAM disk on
//! the live ISO) with ewe-cloud's file keyring, so the live user's keyring
//! is never involved. The app password lives in this process and that 0600
//! file until sign-out or the end of the session; it reaches the helper on
//! stdin, never on argv. Skipping is always fine: the desktop signs in later.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde_json::{json, Value};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

pub struct Session {
    pub dir: PathBuf,
    pub server: String,
    pub login: String,
    pub token: String,
}

#[derive(Default)]
pub struct Account {
    pub session: Mutex<Option<Session>>,
    login_pid: Mutex<Option<u32>>,
}

fn private_dir() -> Result<PathBuf, String> {
    use std::os::unix::fs::PermissionsExt;
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join("ewe-installer-account");
    let _ = std::fs::remove_dir_all(&dir);
    for sub in ["config/ewe", "cache", "state", "backup/ewe"] {
        std::fs::create_dir_all(dir.join(sub)).map_err(|e| e.to_string())?;
    }
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| e.to_string())?;
    Ok(dir)
}

/// The environment every ewe tool runs in here: the private config, the
/// file keyring, no compositor or session bus to poke.
fn tool(dir: &Path, name: &str, args: &[&str]) -> Command {
    let mut c = Command::new(name);
    c.args(args)
        .env("XDG_CONFIG_HOME", dir.join("config"))
        .env("XDG_CACHE_HOME", dir.join("cache"))
        .env("XDG_STATE_HOME", dir.join("state"))
        .env("EWE_CLOUD_FAKE_KEYRING", dir.join("keyring"))
        .env("LANG", "C")
        .env_remove("HYPRLAND_INSTANCE_SIGNATURE")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    c
}

fn last_json(bytes: &[u8]) -> Value {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .find_map(|l| serde_json::from_str::<Value>(l).ok())
        .unwrap_or(Value::Null)
}

fn server_ok(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty()
        && s.len() <= 200
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/' | ':' | '~' | '%')
        })
}

/// Login Flow v2 through ewe-cloud. The sign-in page's URL arrives on the
/// tool's stderr long before the result; it is sent to the window as
/// `cloud-login-url` (Open / Copy while the browser step is pending).
#[tauri::command]
pub async fn cloud_login(app: tauri::AppHandle, server: String) -> Result<Value, String> {
    if !server_ok(&server) {
        return Err("That doesn’t look like a server address.".into());
    }
    let st = app.state::<Account>();
    let dir = private_dir()?;
    let mut child = tool(&dir, "ewe-cloud", &["login", server.trim()])
        .spawn()
        .map_err(|e| format!("ewe-cloud: {e}"))?;
    *st.login_pid.lock().await = child.id();
    let stderr = child.stderr.take().ok_or("no stderr")?;
    let app2 = app.clone();
    let reader = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(l)) = lines.next_line().await {
            if let Some(url) = l.strip_prefix("login-url: ") {
                let _ = app2.emit("cloud-login-url", url.trim().to_string());
            }
        }
    });
    let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
    let _ = reader.await;
    *st.login_pid.lock().await = None;
    let j = last_json(&out.stdout);
    if j["ok"].as_bool() != Some(true) {
        let _ = std::fs::remove_dir_all(&dir);
        if j.is_null() {
            return Ok(json!({ "ok": false, "error": "cancelled" }));
        }
        return Ok(j);
    }
    let token = std::fs::read_to_string(dir.join("keyring"))
        .map(|t| t.trim().to_string())
        .unwrap_or_default();
    let server = j["server"].as_str().unwrap_or("").to_string();
    let login = j["login_name"]
        .as_str()
        .or(j["user"].as_str())
        .unwrap_or("")
        .to_string();
    if token.is_empty() || server.is_empty() || login.is_empty() {
        let _ = std::fs::remove_dir_all(&dir);
        return Ok(json!({ "ok": false, "error": "no-app-password",
                          "message": "The sign-in finished without an app password. Try again." }));
    }
    *st.session.lock().await = Some(Session {
        dir,
        server,
        login,
        token,
    });
    Ok(j)
}

/// Stop a sign-in that is waiting for the browser.
#[tauri::command]
pub async fn cloud_cancel(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(pid) = *app.state::<Account>().login_pid.lock().await {
        let _ = Command::new("kill").arg(pid.to_string()).status().await;
    }
    Ok(())
}

/// What the account holds, read without keeping anything: who saved the
/// backup and when, the apps Komble will offer, the folders that come
/// down, the plugins that computer had, its look.
#[tauri::command]
pub async fn cloud_backup(app: tauri::AppHandle) -> Result<Value, String> {
    let st = app.state::<Account>();
    let guard = st.session.lock().await;
    let s = guard.as_ref().ok_or("Not signed in.")?;
    let status = tool(&s.dir, "ewe-conf", &["sync-status"])
        .output()
        .await
        .map_err(|e| format!("ewe-conf: {e}"))?;
    let status = last_json(&status.stdout);
    if status["remote"].is_null() {
        return Ok(json!({ "exists": false, "error": status["error"] }));
    }
    let out = s.dir.join("backup/ewe/ewe.conf");
    let out_s = out.display().to_string();
    let pulled = tool(&s.dir, "ewe-conf", &["pull", "--out", &out_s])
        .output()
        .await
        .map_err(|e| format!("ewe-conf: {e}"))?;
    let pulled = last_json(&pulled.stdout);
    if pulled["ok"].as_bool() != Some(true) {
        return Err(pulled["message"]
            .as_str()
            .or(pulled["error"].as_str())
            .unwrap_or("Couldn’t read the backup.")
            .to_string());
    }
    // ewe-conf reads the copy as if it were a machine's one file
    let dump = tool(&s.dir, "ewe-conf", &["dump"])
        .env("XDG_CONFIG_HOME", s.dir.join("backup"))
        .output()
        .await
        .map_err(|e| format!("ewe-conf: {e}"))?;
    let doc: Value = serde_json::from_slice(&dump.stdout).unwrap_or(Value::Null);
    let _ = std::fs::remove_file(&out);
    Ok(summarize(&doc, &status))
}

/// The backup in a few words for the account step and the Plugins step.
pub fn summarize(doc: &Value, status: &Value) -> Value {
    let installed = &doc["apps"]["installed"];
    let apps = installed["packages"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0)
        + installed["appimages"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
    let folders: Vec<String> = doc["sync"]["folders"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|f| {
                    let l = f["local"].as_str()?.trim_end_matches('/');
                    let name = l.rsplit('/').next().unwrap_or(l);
                    Some(if name.is_empty() || name == "~" {
                        f["remote"].as_str().unwrap_or("").to_string()
                    } else {
                        name.to_string()
                    })
                })
                .filter(|n| !n.is_empty())
                .collect()
        })
        .unwrap_or_default();
    // [plugins.sources] is THE installed set of the machine that saved it;
    // [plugins].removed are the ones it took out
    let removed: Vec<String> = doc["plugins"]["removed"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut plugins: Vec<String> = doc["plugins"]["sources"]
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    plugins.retain(|p| !removed.contains(p));
    plugins.sort();
    json!({
        "exists": true,
        "machine": status["remote_machine"],
        "modified": status["remote_modified"],
        "apps": apps,
        "folders": folders,
        "plugins": plugins,
        "scheme": doc["desktop"]["theme"]["scheme"],
    })
}

/// "Use another account" / leaving the step signed out: revoke the app
/// password on the server (it was never used) and forget everything.
#[tauri::command]
pub async fn cloud_signout(app: tauri::AppHandle) -> Result<(), String> {
    let st = app.state::<Account>();
    let taken = st.session.lock().await.take();
    if let Some(s) = taken {
        let _ = tool(&s.dir, "ewe-cloud", &["logout"]).output().await;
        let _ = std::fs::remove_dir_all(&s.dir);
    }
    Ok(())
}

/// The helper's stdin for the `account` verb: the new user's password, then
/// the app password — two lines, never argv.
pub async fn account_args(
    app: &tauri::AppHandle,
    args: &[String],
    password: Option<&str>,
) -> Result<(Vec<String>, String), String> {
    let st = app.state::<Account>();
    let guard = st.session.lock().await;
    let s = guard.as_ref().ok_or("not signed in to an ewe account")?;
    let user = args.first().ok_or("no username")?.clone();
    let restore = if args.get(1).map(String::as_str) == Some("yes") {
        "yes"
    } else {
        "no"
    };
    let pw = password.ok_or("no password")?;
    Ok((
        vec![user, s.server.clone(), s.login.clone(), restore.to_string()],
        format!("{pw}\n{}", s.token),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backup_in_a_few_words() {
        let doc = json!({
            "apps": { "installed": { "packages": [{"package": "gimp"}, {"package": "zed"}], "appimages": [{"name": "x"}] } },
            "sync": { "folders": [
                { "remote": "Documents", "local": "~/Documents" },
                { "remote": "", "local": "~/Nextcloud/" },
                { "remote": "Photos", "local": "~" }
            ] },
            "plugins": { "sources": { "ewe.dock": "bundled", "ewe.media": "bundled", "acme.x": "https://x" },
                         "removed": ["ewe.media"] },
            "desktop": { "theme": { "scheme": "ewe-light" } }
        });
        let st = json!({ "remote_machine": "emoh", "remote_modified": "2026-10-31T10:00:00Z" });
        let s = summarize(&doc, &st);
        assert_eq!(s["apps"], 3);
        assert_eq!(s["folders"], json!(["Documents", "Nextcloud", "Photos"]));
        assert_eq!(s["plugins"], json!(["acme.x", "ewe.dock"]));
        assert_eq!(s["machine"], "emoh");
        assert_eq!(s["scheme"], "ewe-light");
        // an empty or partial file still answers
        let s = summarize(&json!({}), &json!({}));
        assert_eq!(s["apps"], 0);
        assert_eq!(s["plugins"], json!([]));
    }

    #[test]
    fn server_addresses() {
        assert!(server_ok("https://cloud.example.org"));
        assert!(server_ok("cloud.example.org:8443/nc"));
        assert!(!server_ok(""));
        assert!(!server_ok("https://x; rm -rf /"));
        assert!(!server_ok("a b"));
    }
}
