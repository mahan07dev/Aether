#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{Emitter, Manager, State};

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

// --------------------------------------------------------------
// Helpers
// --------------------------------------------------------------

/// Check if `dir` contains a file named "aether" (or "aether.exe" on Windows).
fn check_aether_in_dir(dir: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    let binary = dir.join("aether.exe");
    #[cfg(not(windows))]
    let binary = dir.join("aether");

    if binary.exists() && binary.is_file() {
        Some(binary)
    } else {
        None
    }
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<(), String> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;
    if !meta.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }

    let mode = meta.permissions().mode();
    if mode & 0o111 == 0 {
        let new_perm = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(path, new_perm)
            .map_err(|e| format!("Failed to set execute permissions on {}: {}", path.display(), e))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

/// Get the app's data directory (user‑writable).
fn get_app_data_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))
}

// --------------------------------------------------------------
// Tauri commands
// --------------------------------------------------------------

#[tauri::command]
fn check_installed(app_handle: tauri::AppHandle) -> bool {
    let emit_debug = |msg: &str| {
        let payload = serde_json::json!({ "message": msg, "level": "system" });
        let _ = app_handle.emit("log", payload.to_string());
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe_path.parent().unwrap_or(&Path::new("."));

    let found = check_aether_in_dir(exe_dir)
        .or_else(|| check_aether_in_dir(&exe_dir.join("aether")))
        .or_else(|| check_aether_in_dir(&cwd))
        .or_else(|| check_aether_in_dir(&cwd.join("aether")))
        .or_else(|| {
            get_app_data_dir(&app_handle)
                .ok()
                .and_then(|data_dir| {
                    check_aether_in_dir(&data_dir)
                        .or_else(|| check_aether_in_dir(&data_dir.join("aether")))
                })
        })
        .is_some();

    if found {
        emit_debug("Found aether in executable dir, current dir, or app data dir (or their 'aether' subfolder).");
    } else {
        emit_debug("Aether not found in any location.");
    }

    found
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

    let cwd = std::env::current_dir().map_err(|e| format!("Failed to get current dir: {}", e))?;
    let exe_path = std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path.parent().ok_or("No parent directory")?;
    let data_dir = get_app_data_dir(&app_handle)?;

    // Search for binary in:
    // 1. exe_dir
    // 2. exe_dir/aether
    // 3. cwd
    // 4. cwd/aether
    // 5. data_dir
    // 6. data_dir/aether
    let aether_binary = check_aether_in_dir(exe_dir)
        .or_else(|| check_aether_in_dir(&exe_dir.join("aether")))
        .or_else(|| check_aether_in_dir(&cwd))
        .or_else(|| check_aether_in_dir(&cwd.join("aether")))
        .or_else(|| check_aether_in_dir(&data_dir))
        .or_else(|| check_aether_in_dir(&data_dir.join("aether")))
        .ok_or_else(|| {
            let msg = "Aether binary not found. Place it in the app's directory, the current folder, or the app's data directory, optionally inside an 'aether' subfolder.";
            let payload = serde_json::json!({ "message": msg, "level": "stderr" });
            let _ = app_handle.emit("log", payload.to_string());
            msg.to_string()
        })?;

    // Ensure executable permissions
    if let Err(e) = ensure_executable(&aether_binary) {
        let msg = format!("Cannot execute aether: {}. Try running `chmod +x {}` or move it to a user-writable location.", e, aether_binary.display());
        let payload = serde_json::json!({ "message": &msg, "level": "stderr" });
        let _ = app_handle.emit("log", payload.to_string());
        return Err(msg);
    }

    // Working directory: inside app data dir (user‑writable)
    let work_dir = data_dir.join("aether");
    std::fs::create_dir_all(&work_dir)
        .map_err(|e| format!("Failed to create working directory {}: {}", work_dir.display(), e))?;

    let payload = serde_json::json!({
        "message": &format!("Aether will run with working directory: {}", work_dir.display()),
        "level": "system"
    });
    let _ = app_handle.emit("log", payload.to_string());

    // Build arguments
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

    // Spawn the process
    let mut child = Command::new(&aether_binary)
        .current_dir(&work_dir)
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

    // Output readers
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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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