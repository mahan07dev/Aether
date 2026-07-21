#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{Emitter, State};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

struct AppState {
    process: Arc<Mutex<Option<Child>>>,
}

#[derive(Deserialize)]
struct ConnectParams {
    protocol: u8,
    http_version: u8,
    speed: u8,
    use_last: bool,
}

// Helper to find aether binary
fn check_aether_in_dir(dir: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    let binary = dir.join("aether.exe");
    #[cfg(not(windows))]
    let binary = dir.join("aether");

    if binary.exists() && binary.is_file() {
        #[cfg(unix)]
        {
            if let Ok(metadata) = binary.metadata() {
                let mode = metadata.permissions().mode();
                if mode & 0o111 == 0 {
                    let new_mode = mode | 0o755;
                    let _ = std::fs::set_permissions(&binary, PermissionsExt::from_mode(new_mode));
                }
            }
        }
        Some(binary)
    } else {
        None
    }
}

#[tauri::command]
fn check_installed(app_handle: tauri::AppHandle) -> bool {
    let emit_debug = |msg: &str| {
        let payload = serde_json::json!({ "message": msg, "level": "system" });
        let _ = app_handle.emit("log", payload.to_string());
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_path = cwd.join("aether");
    emit_debug(&format!("Checking current dir: {}", cwd_path.display()));

    if check_aether_in_dir(&cwd_path).is_some() {
        emit_debug("Found aether in current directory.");
        return true;
    }

    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe_path.parent().unwrap_or(&Path::new("."));
    emit_debug(&format!("Checking executable dir: {}", exe_dir.display()));

    if check_aether_in_dir(exe_dir).is_some() {
        emit_debug("Found aether in executable directory.");
        return true;
    }

    emit_debug("Aether not found in either location.");
    false
}

#[tauri::command]
async fn connect(
    params: ConnectParams,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    {
        let guard = state.process.lock().unwrap();
        if guard.is_some() {
            return Err("Aether is already running".to_string());
        }
    }

    let exe_path = std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path.parent().ok_or("No parent directory")?;

    let aether_binary = if let Some(path) = check_aether_in_dir(exe_dir) {
        path
    } else if let Some(path) = check_aether_in_dir(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
        path
    } else {
        let msg = "Aether binary not found. Place the 'aether' folder in the app's directory or next to the executable.";
        let payload = serde_json::json!({ "message": msg, "level": "stderr" });
        let _ = app_handle.emit("log", payload.to_string());
        return Err(msg.to_string());
    };

    let mut args = Vec::new();
    match params.protocol {
        1 => { /* --masque default */ }
        2 => args.push("--wg".to_string()),
        3 => args.push("--gool".to_string()),
        _ => return Err("Invalid protocol".to_string()),
    }

    if params.protocol == 1 && params.http_version == 1 {
        args.push("--http2".to_string());
    }

    let speed_flag = match params.speed {
        1 => "turbo",
        2 => "balanced",
        3 => "thorough",
        4 => "stealth",
        _ => return Err("Invalid speed".to_string()),
    };
    args.push("--scan".to_string());
    args.push(speed_flag.to_string());

    if params.use_last {
        args.push("--quick-reconnect".to_string());
    } else {
        args.push("--no-quick-reconnect".to_string());
    }

    let cmd_display = format!("{} {}", aether_binary.display(), args.join(" "));
    let payload = serde_json::json!({ "message": &format!("Running: {}", cmd_display), "level": "system" });
    let _ = app_handle.emit("log", payload.to_string());

    let mut child = Command::new(&aether_binary)
        .current_dir(aether_binary.parent().unwrap_or(Path::new(".")))
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            let msg = format!("Failed to spawn aether: {}", e);
            let payload = serde_json::json!({ "message": &msg, "level": "stderr" });
            let _ = app_handle.emit("log", payload.to_string());
            msg
        })?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    {
        let mut guard = state.process.lock().unwrap();
        *guard = Some(child);
    }

    let app_handle_clone = app_handle.clone();
    thread::spawn(move || {
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);
        let (tx, rx) = std::sync::mpsc::channel::<(String, String)>();

        let tx_stdout = tx.clone();
        thread::spawn(move || {
            for line in stdout_reader.lines() {
                if let Ok(l) = line {
                    if !l.is_empty() {
                        let _ = tx_stdout.send((l, "stdout".to_string()));
                    }
                } else {
                    break;
                }
            }
        });

        let tx_stderr = tx.clone();
        thread::spawn(move || {
            for line in stderr_reader.lines() {
                if let Ok(l) = line {
                    if !l.is_empty() {
                        let _ = tx_stderr.send((l, "stderr".to_string()));
                    }
                } else {
                    break;
                }
            }
        });

        let emit_log = |msg: &str, level: &str| {
            let payload = serde_json::json!({ "message": msg, "level": level });
            let _ = app_handle_clone.emit("log", payload.to_string());
        };

        for (line, source) in rx {
            emit_log(&line, &source);
        }

        emit_log("Aether process exited.", "system");
        let _ = app_handle_clone.emit("process_exited", "true");
    });

    Ok(())
}

#[tauri::command]
fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    let child_opt = {
        let mut guard = state.process.lock().unwrap();
        guard.take()
    };

    if let Some(mut child) = child_opt {
        if let Err(e) = child.kill() {
            return Err(format!("Failed to kill process: {}", e));
        }
        thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    } else {
        Err("No process running".to_string())
    }
}

#[tauri::command]
fn is_running(state: State<'_, AppState>) -> bool {
    let mut guard = state.process.lock().unwrap();
    if let Some(ref mut child) = *guard {
        match child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    } else {
        false
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init()) // optional, but keep it if you want
        .manage(AppState {
            process: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            check_installed,
            connect,
            disconnect,
            is_running,
            open_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    Ok(())
}