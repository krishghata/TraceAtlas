use serde::Serialize;
use std::process::Command;
use tauri::command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Suppresses the flash of a console window on Windows when spawning child processes
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize)]
pub struct TracerouteResult {
    pub output: String,
    pub is_win: bool,
}

/// Run traceroute/tracert on the target and return raw output + platform flag.
/// Input is validated to allow only safe characters (alphanumeric, dot, hyphen).
#[command]
async fn run_traceroute(target: String) -> Result<TracerouteResult, String> {
    let valid = target
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-');
    if !valid || target.is_empty() {
        return Err("Invalid target — must be a domain or IP address".to_string());
    }

    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(target_os = "windows")]
        let result = Command::new("tracert")
            .args(["-d", "-h", "20", "-w", "3000", target.as_str()])
            .creation_flags(CREATE_NO_WINDOW)
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
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}

#[derive(Serialize)]
pub struct PingResult {
    pub output: String,
    pub is_win: bool,
}

/// Run ping against target for `count` packets (capped at 50).
/// Input validated same as traceroute — only alphanumeric, dot, hyphen.
#[command]
async fn run_ping(target: String, count: u32) -> Result<PingResult, String> {
    let valid = target
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-');
    if !valid || target.is_empty() {
        return Err("Invalid target".to_string());
    }
    let count = count.min(50).max(1).to_string();

    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(target_os = "windows")]
        let result = Command::new("ping")
            .args(["-n", &count, "-w", "3000", target.as_str()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        #[cfg(not(target_os = "windows"))]
        let result = Command::new("ping")
            .args(["-c", &count, "-W", "3", target.as_str()])
            .output();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok(PingResult {
                    output: stdout + &stderr,
                    is_win: cfg!(target_os = "windows"),
                })
            }
            Err(e) => Err(format!("Failed to run ping: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}

#[derive(Serialize)]
pub struct GatewayResult {
    pub gateway: Option<String>,
}

/// Detect the default gateway IP (router address) for network diagnostics.
/// On Windows parses `ipconfig`; on Linux/Mac uses `ip route` then `route -n`.
#[command]
async fn get_default_gateway() -> Result<GatewayResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        {
            match Command::new("ipconfig")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                Ok(output) => {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    for line in text.lines() {
                        let lower = line.to_lowercase();
                        if lower.contains("default gateway") {
                            if let Some(ip) = line.split(':').last() {
                                let ip = ip.trim().to_string();
                                // Must contain a dot (IPv4) and not be empty/placeholder
                                if !ip.is_empty() && ip.contains('.') && ip != "." {
                                    return Ok(GatewayResult { gateway: Some(ip) });
                                }
                            }
                        }
                    }
                    Ok(GatewayResult { gateway: None })
                }
                Err(e) => Err(format!("Failed to run ipconfig: {}", e)),
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Primary: `ip route show default`
            if let Ok(output) = Command::new("ip").args(["route", "show", "default"]).output() {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                for line in text.lines() {
                    if line.starts_with("default via ") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            return Ok(GatewayResult {
                                gateway: Some(parts[2].to_string()),
                            });
                        }
                    }
                }
            }
            // Fallback: `route -n`
            if let Ok(output) = Command::new("route").args(["-n"]).output() {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                for line in text.lines() {
                    if line.starts_with("0.0.0.0") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return Ok(GatewayResult {
                                gateway: Some(parts[1].to_string()),
                            });
                        }
                    }
                }
            }
            Ok(GatewayResult { gateway: None })
        }
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            run_traceroute,
            run_ping,
            get_default_gateway
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
