#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::time::{Duration, Instant};
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Serialize)]
struct PingResult {
    ok: bool,
    ms: u64,
}

/// Допускаем только IPv4 / имя хоста, чтобы в ping.exe нельзя было подсунуть ключи.
fn valid_host(h: &str) -> bool {
    !h.is_empty()
        && h.len() <= 253
        && !h.starts_with('-')
        && h.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// Настоящий ICMP-пинг через системный ping (без прав администратора).
/// Успех = в ответе есть "TTL=" (так же и в русской локали); ответы вида
/// «Заданный узел недоступен» TTL не содержат и считаются потерей.
#[tauri::command]
async fn ping_host(ip: String, timeout_ms: u64) -> Result<PingResult, String> {
    if !valid_host(&ip) {
        return Err("bad host".into());
    }
    let timeout_ms = timeout_ms.clamp(200, 30_000);

    let mut cmd = Command::new("ping");
    #[cfg(windows)]
    {
        cmd.args(["-n", "1", "-w", &timeout_ms.to_string(), &ip]);
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let secs = ((timeout_ms + 999) / 1000).to_string();
        cmd.args(["-c", "1", "-W", &secs, &ip]);
    }
    cmd.kill_on_drop(true);

    let t0 = Instant::now();
    let fut = cmd.output();
    let out = tokio::time::timeout(Duration::from_millis(timeout_ms + 2_000), fut).await;
    let ms = t0.elapsed().as_millis() as u64;

    match out {
        Ok(Ok(o)) => {
            let text = String::from_utf8_lossy(&o.stdout).to_lowercase();
            let ok = o.status.success() && text.contains("ttl=");
            Ok(PingResult { ok, ms })
        }
        _ => Ok(PingResult { ok: false, ms }),
    }
}

/// hosts.json рядом с exe (или в текущей папке).
#[tauri::command]
fn load_hosts() -> Option<String> {
    let mut paths = vec![];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("hosts.json"));
        }
    }
    paths.push(std::path::PathBuf::from("hosts.json"));
    for p in paths {
        if let Ok(s) = std::fs::read_to_string(&p) {
            return Some(s.trim_start_matches('\u{feff}').to_string());
        }
    }
    None
}

/// Открыть веб-интерфейс точки в браузере по умолчанию.
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    let host = url.strip_prefix("http://").ok_or("only http")?;
    let host = host.trim_end_matches('/');
    if !valid_host(host) {
        return Err("bad host".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &format!("http://{host}")])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("xdg-open").arg(format!("http://{host}")).spawn();
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping_host, load_hosts, open_url])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
