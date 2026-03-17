use serde::Serialize;
use std::process::Command;
use tauri::command;

#[derive(Serialize)]
pub struct TracerouteResult {
    pub output: String,
    pub is_win: bool,
}

/// Run traceroute/tracert on the target and return raw output + platform flag.
/// Input is validated to allow only safe characters (alphanumeric, dot, hyphen).
#[command]
fn run_traceroute(target: String) -> Result<TracerouteResult, String> {
    // Input validation — prevent command injection
    let valid = target
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-');
    if !valid || target.is_empty() {
        return Err("Invalid target — must be a domain or IP address".to_string());
    }

    #[cfg(target_os = "windows")]
    let result = Command::new("tracert")
        .args(["-d", "-h", "20", "-w", "3000", target.as_str()])
        .output();

    #[cfg(not(target_os = "windows"))]
    let result = Command::new("traceroute")
        .args(["-n", "-m", "20", "-w", "3", target.as_str()])
        .output();

    match result {
        Ok(output) => Ok(TracerouteResult {
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            is_win: cfg!(target_os = "windows"),
        }),
        Err(e) => Err(format!("Failed to run traceroute: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .invoke_handler(tauri::generate_handler![run_traceroute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
