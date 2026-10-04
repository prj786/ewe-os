//! ewe-installer backend — a thin driver over ewe-install-helper (RFC-003).
//!
//! Nothing privileged happens in this process: every mutating verb goes
//! through `pkexec ewe-install-helper <verb> …` (fixed allowlist, argv-only,
//! the Komble pattern). This side only: read-only probes, streaming the
//! helper's JSON progress lines to the frontend as `install-progress`
//! events, and the timezone suggestion (two-provider agreement — the same
//! rule as the desktop's dispatcher; never silently trusted, the UI shows
//! it for confirmation).

use std::process::Stdio;

use serde_json::{json, Value};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

const HELPER: &str = "/usr/lib/ewe-installer/ewe-install-helper";
/// The live system's ewe payload: the add-ons it carries (ewe 0.25+).
const PAYLOAD_PLUGINS: &str = "/usr/share/ewe/plugins";

fn estr<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

async fn helper(
    app: &tauri::AppHandle,
    args: &[&str],
    stdin_line: Option<&str>,
) -> Result<(), String> {
    let mut cmd = Command::new("pkexec");
    cmd.arg(HELPER)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.stdin(if stdin_line.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut child = cmd.spawn().map_err(estr)?;
    if let (Some(line), Some(mut si)) = (stdin_line, child.stdin.take()) {
        use tokio::io::AsyncWriteExt;
        si.write_all(line.as_bytes()).await.map_err(estr)?;
        si.write_all(b"\n").await.map_err(estr)?;
        drop(si);
    }
    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;
    let app2 = app.clone();
    let reader = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(l)) = lines.next_line().await {
            if let Ok(v) = serde_json::from_str::<Value>(&l) {
                let _ = app2.emit("install-progress", v);
            }
        }
    });
    // The helper's `die` writes {"error":…} to stderr. Drain it concurrently
    // (a full pipe would deadlock the child) and keep only a short tail —
    // that tail is the message the user sees when a verb fails.
    let err_reader = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut tail: Vec<String> = Vec::new();
        while let Ok(Some(l)) = lines.next_line().await {
            if l.trim().is_empty() {
                continue;
            }
            if tail.len() >= 40 {
                tail.remove(0);
            }
            tail.push(l);
        }
        tail
    });
    let status = child.wait().await.map_err(estr)?;
    let _ = reader.await;
    let tail = err_reader.await.unwrap_or_default();
    if status.success() {
        Ok(())
    } else {
        Err(match status.code() {
            Some(126) => "Authentication dialog was dismissed.".into(),
            Some(127) => "Not authorized (polkit refused).".into(),
            c => helper_error(&tail, c),
        })
    }
}

/// The helper's own words, if it left any: the last `{"error":"…"}` line on
/// stderr, else its last plain line, else the bare exit code.
fn helper_error(tail: &[String], code: Option<i32>) -> String {
    let json_err = tail.iter().rev().find_map(|l| {
        serde_json::from_str::<Value>(l)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
    });
    if let Some(e) = json_err {
        return e;
    }
    match tail.last() {
        Some(l) => {
            let l: String = l.chars().take(300).collect();
            format!("{l} (helper exit {code:?})")
        }
        None => format!("helper failed (exit {code:?})"),
    }
}

// ── read-only probes ─────────────────────────────────────────────────────────

#[tauri::command]
async fn probe(app: tauri::AppHandle) -> Result<Value, String> {
    // probe is read-only but lsblk/battery info needs no root — run the
    // helper logic directly here to avoid a pointless auth prompt
    let out = Command::new("lsblk")
        .args(["-J", "-d", "-o", "NAME,PATH,SIZE,MODEL,TYPE,RM,TRAN"])
        .output()
        .await
        .map_err(estr)?;
    let v: Value = serde_json::from_slice(&out.stdout).map_err(estr)?;
    let disks: Vec<Value> = v["blockdevices"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|b| b["type"] == "disk" && b["rm"] != true)
        .collect();
    let battery = std::fs::read_dir("/sys/class/power_supply")
        .map(|d| {
            d.flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with("BAT"))
        })
        .unwrap_or(false);
    let ram_gb = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1).map(String::from))
        })
        .and_then(|kb| kb.parse::<u64>().ok())
        .map(|kb| kb / 1024 / 1024)
        .unwrap_or(0);
    let _ = app;
    Ok(json!({"disks": disks, "battery": battery, "ram_gb": ram_gb}))
}

#[tauri::command]
async fn timezones() -> Result<Value, String> {
    let out = Command::new("timedatectl")
        .arg("list-timezones")
        .output()
        .await
        .map_err(estr)?;
    let zones: Vec<&str> = std::str::from_utf8(&out.stdout)
        .map_err(estr)?
        .lines()
        .collect();
    Ok(json!(zones))
}

/// The live system's xkb registry — every layout xkeyboard-config ships,
/// with human names, same as the DE's Settings. base.lst: "! layout"
/// section, one "  code   Name" line each.
#[tauri::command]
async fn keyboard_layouts() -> Result<Value, String> {
    let text = std::fs::read_to_string("/usr/share/X11/xkb/rules/base.lst").map_err(estr)?;
    let mut in_layout = false;
    let mut out: Vec<Value> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(s) = t.strip_prefix('!') {
            in_layout = s.trim() == "layout";
            continue;
        }
        if !in_layout || t.is_empty() {
            continue;
        }
        if let Some((code, name)) = t.split_once(char::is_whitespace) {
            out.push(json!({"c": code, "n": name.trim()}));
        }
    }
    Ok(json!(out))
}

#[tauri::command]
async fn locales() -> Result<Value, String> {
    let s = std::fs::read_to_string("/usr/share/i18n/SUPPORTED").unwrap_or_default();
    let l: Vec<String> = s
        .lines()
        .filter(|l| l.contains("UTF-8"))
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .collect();
    Ok(json!(l))
}

/// The add-ons the live ISO's ewe payload offers (`plugins/bundle.json` +
/// each `plugins/<id>/manifest.json`): id, name, description, icon (a Theme
/// icon NAME the frontend maps to a Lucide glyph), category, version — the
/// `available` list of `ewe-plugin list --json`, read straight from the
/// files so no user config has to exist. An ISO carrying an older ewe has no
/// bundle.json: the list is empty and the Add-ons step hides itself.
/// `EWE_PAYLOAD_PLUGINS` overrides the directory (dev, tests).
fn read_addons(dir: &std::path::Path) -> Vec<Value> {
    let bundle: Value = match std::fs::read_to_string(dir.join("bundle.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
    {
        Some(v) => v,
        None => return Vec::new(),
    };
    let Some(plugins) = bundle["plugins"].as_object() else {
        return Vec::new();
    };
    let mut out: Vec<Value> = Vec::new();
    for (id, meta) in plugins {
        let Ok(text) = std::fs::read_to_string(dir.join(id).join("manifest.json")) else {
            continue; // listed but not shipped: nothing to offer
        };
        let Ok(m) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        if m["id"].as_str() != Some(id.as_str()) {
            continue; // a manifest that is not the add-on bundle.json names
        }
        let pick = |k: &str| m[k].as_str().unwrap_or("").to_string();
        out.push(json!({
            "id": id,
            "name": if pick("name").is_empty() { id.clone() } else { pick("name") },
            "description": pick("description"),
            "icon": pick("icon"),
            "category": pick("category"),
            "version": m["version"].as_str().or(meta["version"].as_str()).unwrap_or(""),
        }));
    }
    out.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    out
}

#[tauri::command]
async fn addons() -> Result<Value, String> {
    let dir = std::env::var_os("EWE_PAYLOAD_PLUGINS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(PAYLOAD_PLUGINS));
    Ok(json!(read_addons(&dir)))
}

/// The suggestion rule from the desktop's dispatcher: TWO providers must
/// agree, otherwise no suggestion at all. The UI always shows the result
/// for confirmation — automatic never silently wins.
#[tauri::command]
async fn suggest_timezone() -> Result<Value, String> {
    async fn ask(url: &str) -> Option<String> {
        let out = Command::new("curl")
            .args(["-sf", "--max-time", "7", url])
            .output()
            .await
            .ok()?;
        let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
        (!s.is_empty() && !s.contains(' ')).then_some(s)
    }
    let a = ask("https://ipinfo.io/timezone").await;
    let b = ask("https://ipapi.co/timezone").await;
    Ok(match (a, b) {
        (Some(x), Some(y)) if x == y => json!({"zone": x, "confident": true}),
        (x, y) => json!({"zone": x.or(y), "confident": false}),
    })
}

// ── network (unprivileged — NetworkManager lets an active session scan and
// join; no helper verb, no polkit prompt) ──────────────────────────────────

/// nmcli terse output escapes ':' and '\' inside values.
fn nm_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(n) = chars.next() {
                out.push(n);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Split one nmcli -t line into at most `n` fields, honouring '\:' escapes.
fn nm_fields(line: &str, n: usize) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            cur.push(c);
            if let Some(n2) = chars.next() {
                cur.push(n2);
            }
        } else if c == ':' && fields.len() + 1 < n {
            fields.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    fields.push(cur);
    fields.into_iter().map(|f| nm_unescape(&f)).collect()
}

async fn nmcli(args: &[&str]) -> Result<String, String> {
    let out = Command::new("nmcli")
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("nmcli: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() { stdout } else { err })
    }
}

/// {"connectivity": full|limited|portal|none|unknown, "wired": bool,
///  "wifi_device": bool, "ssid": current Wi-Fi network or ""}
#[tauri::command]
async fn net_status() -> Result<Value, String> {
    let connectivity = nmcli(&["networking", "connectivity", "check"])
        .await
        .unwrap_or_else(|_| "unknown".into());
    let devices = nmcli(&["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "device"])
        .await
        .unwrap_or_default();
    let mut wired = false;
    let mut wifi_device = false;
    let mut ssid = String::new();
    for line in devices.lines() {
        let f = nm_fields(line, 4);
        if f.len() < 3 {
            continue;
        }
        let connected = f[2].starts_with("connected");
        match f[1].as_str() {
            "ethernet" if connected => wired = true,
            "wifi" => {
                wifi_device = true;
                if connected {
                    ssid = f.get(3).cloned().unwrap_or_default();
                }
            }
            _ => {}
        }
    }
    Ok(json!({
        "connectivity": connectivity,
        "wired": wired,
        "wifi_device": wifi_device,
        "ssid": ssid,
    }))
}

/// Visible networks, one per SSID (strongest wins), strongest first.
/// [{"ssid", "signal", "secured", "in_use"}]
#[tauri::command]
async fn wifi_list(rescan: Option<bool>) -> Result<Value, String> {
    let rescan = if rescan.unwrap_or(false) {
        "yes"
    } else {
        "auto"
    };
    let out = match nmcli(&[
        "-t",
        "-f",
        "IN-USE,SIGNAL,SECURITY,SSID",
        "device",
        "wifi",
        "list",
        "--rescan",
        rescan,
    ])
    .await
    {
        Ok(o) => o,
        // no Wi-Fi hardware is not an error for the UI — an empty list is
        Err(e) if e.contains("No Wi-Fi device") => return Ok(json!([])),
        Err(e) => return Err(e),
    };
    let mut best: std::collections::BTreeMap<String, (i64, bool, bool)> = Default::default();
    for line in out.lines() {
        let f = nm_fields(line, 4);
        if f.len() < 4 || f[3].is_empty() {
            continue;
        }
        let in_use = f[0].trim() == "*";
        let signal = f[1].trim().parse::<i64>().unwrap_or(0);
        let secured = !f[2].trim().is_empty() && f[2].trim() != "--";
        let e = best
            .entry(f[3].clone())
            .or_insert((signal, secured, in_use));
        if signal > e.0 || in_use {
            *e = (signal.max(e.0), secured, in_use || e.2);
        }
    }
    let mut list: Vec<(String, i64, bool, bool)> = best
        .into_iter()
        .map(|(s, (g, sec, u))| (s, g, sec, u))
        .collect();
    list.sort_by(|a, b| b.3.cmp(&a.3).then(b.1.cmp(&a.1)));
    Ok(json!(list
        .into_iter()
        .map(|(ssid, signal, secured, in_use)| {
            json!({"ssid": ssid, "signal": signal, "secured": secured, "in_use": in_use})
        })
        .collect::<Vec<_>>()))
}

/// Join a network. argv only — the password never touches a shell. Returns
/// nmcli's own explanation on failure (wrong password, timeout, …).
#[tauri::command]
async fn wifi_connect(ssid: String, password: Option<String>) -> Result<(), String> {
    if ssid.is_empty() || ssid.len() > 32 {
        return Err("invalid network name".into());
    }
    let mut args: Vec<&str> = vec!["device", "wifi", "connect", ssid.as_str()];
    let pw = password.unwrap_or_default();
    if !pw.is_empty() {
        args.push("password");
        args.push(pw.as_str());
    }
    nmcli(&args).await.map(|_| ())
}

// ── the install sequence (each step = one helper verb, streamed) ────────────

#[tauri::command]
async fn run_step(
    app: tauri::AppHandle,
    step: String,
    args: Vec<String>,
    secret: Option<String>,
) -> Result<(), String> {
    let allowed = [
        "partition",
        "mkfs",
        "pacstrap",
        "hibernate",
        "bootloader",
        "user",
        "settz",
        "setlocale",
        "sethostname",
        "layer",
        "upgrade",
        "addons",
        "reboot",
    ];
    if !allowed.contains(&step.as_str()) {
        return Err(format!("unknown step {step}"));
    }
    let mut argv: Vec<&str> = vec![step.as_str()];
    argv.extend(args.iter().map(String::as_str));
    helper(&app, &argv, secret.as_deref()).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            probe,
            timezones,
            keyboard_layouts,
            locales,
            suggest_timezone,
            addons,
            net_status,
            wifi_list,
            wifi_connect,
            run_step
        ])
        .run(tauri::generate_context!())
        .expect("error while running ewe-installer");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d =
            std::env::temp_dir().join(format!("ewe-installer-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn manifest(dir: &std::path::Path, id: &str, body: &str) {
        std::fs::create_dir_all(dir.join(id)).unwrap();
        std::fs::write(dir.join(id).join("manifest.json"), body).unwrap();
    }

    #[test]
    fn addons_from_a_payload() {
        let d = tmp("payload");
        std::fs::write(
            d.join("bundle.json"),
            r#"{"plugins":{"ewe.dock":{"version":"1.0.1","default":false,"migrate":true},
                           "ewe.media":{"version":"1.0.1"},
                           "ewe.ghost":{"version":"9"},
                           "ewe.liar":{"version":"1"}}}"#,
        )
        .unwrap();
        manifest(
            &d,
            "ewe.media",
            r#"{"id":"ewe.media","name":"Music","description":"Now playing","icon":"icMusic","category":"Media","version":"1.0.1"}"#,
        );
        manifest(
            &d,
            "ewe.dock",
            r#"{"id":"ewe.dock","name":"Dock","description":"A dock","icon":"icApps","category":"Desktop"}"#,
        );
        // in bundle.json, but its manifest names another id: not offered
        manifest(&d, "ewe.liar", r#"{"id":"ewe.other","name":"Liar"}"#);
        // ewe.ghost: listed, no directory — skipped
        let got = read_addons(&d);
        let ids: Vec<&str> = got.iter().map(|a| a["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["ewe.dock", "ewe.media"]);
        assert_eq!(got[0]["version"], "1.0.1"); // from bundle.json when the manifest has none
        assert_eq!(got[0]["category"], "Desktop");
        assert_eq!(got[1]["icon"], "icMusic");
        assert_eq!(got[1]["description"], "Now playing");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn no_payload_means_no_addons() {
        let d = tmp("empty");
        assert!(read_addons(&d).is_empty());
        assert!(read_addons(&d.join("does-not-exist")).is_empty());
        std::fs::write(d.join("bundle.json"), "not json").unwrap();
        assert!(read_addons(&d).is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn the_real_payload_when_present() {
        // the dev box or the ISO: every add-on bundle.json lists is offered
        let d = std::path::Path::new(PAYLOAD_PLUGINS);
        if !d.join("bundle.json").exists() {
            return;
        }
        let got = read_addons(d);
        assert!(!got.is_empty());
        for a in &got {
            assert!(a["id"].as_str().unwrap().starts_with("ewe."));
            assert!(!a["name"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    fn helper_error_prefers_the_json_line() {
        let tail = vec![
            "noise".to_string(),
            r#"{"error":"target not mounted"}"#.to_string(),
        ];
        assert_eq!(helper_error(&tail, Some(1)), "target not mounted");
        assert_eq!(
            helper_error(&["plain".to_string()], Some(3)),
            "plain (helper exit Some(3))"
        );
        assert_eq!(helper_error(&[], None), "helper failed (exit None)");
    }
}
