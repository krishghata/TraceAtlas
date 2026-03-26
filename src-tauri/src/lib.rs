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

// ── DNS Lookup ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DnsResult {
    pub output: String,
}

/// Run nslookup for the given domain and record type against an optional server.
/// record_type must be one of: A AAAA MX NS TXT CNAME SOA PTR SRV
#[command]
async fn dns_lookup(domain: String, record_type: String, server: String) -> Result<DnsResult, String> {
    let valid_domain = !domain.is_empty()
        && domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-');
    if !valid_domain {
        return Err("Invalid domain".to_string());
    }
    let allowed = ["A","AAAA","MX","NS","TXT","CNAME","SOA","PTR","SRV"];
    if !allowed.contains(&record_type.to_uppercase().as_str()) {
        return Err("Invalid record type".to_string());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let type_arg = format!("-type={}", record_type.to_uppercase());
        let mut args: Vec<&str> = vec![&type_arg, &domain];
        if !server.is_empty() { args.push(&server); }

        #[cfg(target_os = "windows")]
        let result = Command::new("nslookup")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        #[cfg(not(target_os = "windows"))]
        let result = Command::new("nslookup").args(&args).output();

        match result {
            Ok(o) => Ok(DnsResult {
                output: String::from_utf8_lossy(&o.stdout).to_string()
                      + &String::from_utf8_lossy(&o.stderr).to_string(),
            }),
            Err(e) => Err(format!("Failed to run nslookup: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}

// ── Whois Lookup ──────────────────────────────────────────────────────────────

/// Query IANA whois first to discover the authoritative server, then query it.
/// Works for domains and IP addresses.
#[command]
async fn whois_lookup(query: String) -> Result<String, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};

    let valid = !query.is_empty()
        && query.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '/');
    if !valid {
        return Err("Invalid query".to_string());
    }

    async fn query_server(server: &str, q: &str) -> Result<String, String> {
        let addr = format!("{}:43", server);
        let conn = timeout(Duration::from_secs(8), TcpStream::connect(&addr))
            .await
            .map_err(|_| format!("Timeout connecting to {}", server))?
            .map_err(|e| format!("Cannot connect to {}: {}", server, e))?;
        let mut stream = conn;
        stream.write_all(format!("{}\r\n", q).as_bytes()).await
            .map_err(|e| format!("Write error: {}", e))?;
        let mut resp = String::new();
        timeout(Duration::from_secs(10), stream.read_to_string(&mut resp))
            .await
            .map_err(|_| "Timeout reading response".to_string())?
            .map_err(|e| format!("Read error: {}", e))?;
        Ok(resp)
    }

    // Step 1: ask IANA for referral
    let iana = query_server("whois.iana.org", &query).await?;
    let referral = iana.lines()
        .find(|l| l.to_lowercase().starts_with("refer:"))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .map(|s| s.trim().to_string());

    if let Some(ref_server) = referral {
        let full = query_server(&ref_server, &query).await?;
        Ok(format!("[ via {} ]\n\n{}", ref_server, full))
    } else {
        Ok(iana)
    }
}

// ── Port Scanner ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PortResult {
    pub port:    u16,
    pub open:    bool,
    pub service: String,
}

fn known_service(port: u16) -> &'static str {
    match port {
        21    => "FTP",         22    => "SSH",
        23    => "Telnet",      25    => "SMTP",
        53    => "DNS",         80    => "HTTP",
        110   => "POP3",        143   => "IMAP",
        443   => "HTTPS",       445   => "SMB",
        587   => "SMTP-TLS",    993   => "IMAPS",
        995   => "POP3S",       1433  => "MSSQL",
        3306  => "MySQL",       3389  => "RDP",
        5432  => "PostgreSQL",  5900  => "VNC",
        6379  => "Redis",       8080  => "HTTP-Alt",
        8443  => "HTTPS-Alt",   27017 => "MongoDB",
        _     => "",
    }
}

/// Concurrently probe TCP ports on a host. timeout_ms is capped 200–5000ms.
#[command]
async fn scan_ports(host: String, ports: Vec<u16>, timeout_ms: u64) -> Result<Vec<PortResult>, String> {
    use std::net::SocketAddr;
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};

    let valid = !host.is_empty()
        && host.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == ':');
    if !valid { return Err("Invalid host".to_string()); }
    if ports.len() > 1024 { return Err("Too many ports (max 1024)".to_string()); }

    // Resolve host once
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host(format!("{}:80", host))
        .await
        .map_err(|e| format!("Cannot resolve {}: {}", host, e))?
        .collect();
    let ip = addrs.first().ok_or("No addresses found")?.ip();

    let dur = Duration::from_millis(timeout_ms.clamp(200, 5000));

    let mut handles = Vec::with_capacity(ports.len());
    for port in ports {
        let ip = ip;
        handles.push(tokio::spawn(async move {
            let addr = SocketAddr::new(ip, port);
            let open = timeout(dur, TcpStream::connect(addr)).await
                .map(|r| r.is_ok())
                .unwrap_or(false);
            PortResult { port, open, service: known_service(port).to_string() }
        }));
    }

    let mut results = Vec::with_capacity(handles.len());
    for h in handles {
        if let Ok(r) = h.await { results.push(r); }
    }
    results.sort_by_key(|r| r.port);
    Ok(results)
}

// ── MTR (My Traceroute) ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct MtrHop {
    pub ip:     String,
    pub rtt_ms: Option<f64>,
}

/// Parse a single RTT value from a one-packet ping output (Windows or Unix).
fn parse_single_rtt(text: &str) -> Option<f64> {
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("time<1ms") || lower.contains("time<1 ms") {
            return Some(0.5);
        }
        if let Some(pos) = lower.find("time=") {
            let after = &lower[pos + 5..];
            let num: String = after.chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(v) = num.parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}

/// Probe a list of IPs with a single ping each, all concurrently.
/// Returns RTT (ms) or None for each IP in the same order.
#[command]
async fn mtr_probe(ips: Vec<String>, timeout_ms: u64) -> Result<Vec<MtrHop>, String> {
    let timeout = timeout_ms.clamp(500, 5000);

    let mut handles = Vec::with_capacity(ips.len());
    for ip in ips {
        handles.push(tokio::spawn(async move {
            let ip2 = ip.clone();
            let rtt = tauri::async_runtime::spawn_blocking(move || {
                #[cfg(target_os = "windows")]
                let out = Command::new("ping")
                    .args(["-n", "1", "-w", &timeout.to_string(), &ip2])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
                #[cfg(not(target_os = "windows"))]
                let out = Command::new("ping")
                    .args(["-c", "1", "-W", &(timeout / 1000).max(1).to_string(), &ip2])
                    .output();
                out.ok().and_then(|o| {
                    parse_single_rtt(&String::from_utf8_lossy(&o.stdout))
                })
            })
            .await
            .unwrap_or(None);
            MtrHop { ip, rtt_ms: rtt }
        }));
    }

    let mut results = Vec::with_capacity(handles.len());
    for h in handles {
        if let Ok(hop) = h.await { results.push(hop); }
    }
    Ok(results)
}

// ── Network Scanner ───────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct NetworkDevice {
    pub ip:          String,
    pub mac:         String,
    pub vendor:      String,
    pub hostname:    String,
    pub device_type: String,
}

/// Map a MAC OUI prefix (6 hex chars, uppercase, no separators) to (vendor, type).
fn mac_vendor(mac: &str) -> (&'static str, &'static str) {
    let clean: String = mac.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(6)
        .collect::<String>()
        .to_uppercase();
    match clean.as_str() {
        // Apple
        "A4C361"|"F0DCE2"|"3C2282"|"DC2B2A"|"A45E60"|"788C54"|"F0B479"|"60F4EC" => ("Apple",          "Phone/Mac"),
        // Samsung
        "9C9281"|"D022BE"|"8801A7"|"843DC6"|"CC07AB"|"E4B021" => ("Samsung",        "Phone/TV"),
        // Raspberry Pi
        "B827EB"|"DC2494"|"E45F01"|"2CCF67"                   => ("Raspberry Pi",   "IoT"),
        // Cisco
        "00000C"|"001120"|"002702"|"7C690F"|"E8BA70"|"3C5731" => ("Cisco",          "Router/Switch"),
        // TP-Link
        "14EBE6"|"50C7BF"|"8CFAB1"|"B0487A"|"C46E1F"|"6C5AB5" => ("TP-Link",       "Router/AP"),
        // Netgear
        "10BF48"|"20E52A"|"28C68E"|"4487FC"|"6CB0CE"|"9C3DCF" => ("Netgear",       "Router/AP"),
        // ASUS
        "107B44"|"2C56DC"|"50465D"|"AC220B"|"E03F49"          => ("ASUS",          "Router/PC"),
        // Intel (PCs/laptops)
        "A4C494"|"8C8D28"|"B4969C"|"CC3D82"|"F8341F"|"00E04C" => ("Intel",         "PC/Laptop"),
        // Amazon Echo / Fire
        "0C7018"|"44650D"|"8871E5"|"A002DC"|"FC65DE"|"7485C4" => ("Amazon",        "Smart Device"),
        // Google (Nest, Chromecast)
        "54604A"|"F4F5E8"|"48D705"|"A4977A"|"F88FCA"          => ("Google",        "Smart Device"),
        // Microsoft
        "606BFF"|"7845C4"|"985FD3"|"60457F"                   => ("Microsoft",     "PC/Xbox"),
        // Sony (PlayStation, TV)
        "000D4B"|"001315"|"00D9D1"|"F8D0AC"                   => ("Sony",          "PlayStation/TV"),
        _ => ("Unknown", "Device"),
    }
}

fn parse_arp_output(text: &str) -> Vec<NetworkDevice> {
    let mut devices = vec![];
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue }

        // Windows format: "  192.168.1.1   aa-bb-cc-dd-ee-ff   dynamic"
        #[cfg(target_os = "windows")]
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0].contains('.') && parts[1].contains('-') {
                let ip  = parts[0].to_string();
                let mac = parts[1].to_uppercase();
                let oui = mac.replace('-', "");
                let (vendor, dtype) = mac_vendor(&oui);
                devices.push(NetworkDevice {
                    ip, mac,
                    vendor:      vendor.to_string(),
                    hostname:    String::new(),
                    device_type: dtype.to_string(),
                });
            }
        }

        // Unix format: "hostname (192.168.1.1) at aa:bb:cc:dd:ee:ff [ether] on eth0"
        #[cfg(not(target_os = "windows"))]
        {
            if let (Some(s), Some(e)) = (line.find('('), line.find(')')) {
                let ip       = line[s + 1..e].to_string();
                let hostname = line[..s].trim().to_string();
                let mac = line[e..].find(" at ")
                    .and_then(|p| line[e + p + 4..].split_whitespace().next())
                    .unwrap_or("")
                    .to_uppercase();
                let oui = mac.replace(':', "");
                let (vendor, dtype) = mac_vendor(&oui);
                devices.push(NetworkDevice {
                    ip, mac,
                    vendor:      vendor.to_string(),
                    hostname,
                    device_type: dtype.to_string(),
                });
            }
        }
    }
    // Deduplicate by IP
    let mut seen = std::collections::HashSet::new();
    devices.retain(|d| seen.insert(d.ip.clone()));
    devices
}

/// Read the OS ARP table — no elevated privileges required.
#[command]
async fn arp_scan() -> Result<Vec<NetworkDevice>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        let out = Command::new("arp")
            .args(["-a"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("arp failed: {}", e))?;

        #[cfg(not(target_os = "windows"))]
        let out = Command::new("arp")
            .args(["-a"])
            .output()
            .map_err(|e| format!("arp failed: {}", e))?;

        Ok(parse_arp_output(&String::from_utf8_lossy(&out.stdout)))
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}

/// Get the local machine's outbound IP (no packets sent — UDP trick).
#[command]
fn get_local_ip() -> Result<String, String> {
    use std::net::UdpSocket;
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    sock.connect("8.8.8.8:80").map_err(|e| e.to_string())?;
    Ok(sock.local_addr().map_err(|e| e.to_string())?.ip().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            run_traceroute,
            run_ping,
            get_default_gateway,
            dns_lookup,
            whois_lookup,
            scan_ports,
            mtr_probe,
            arp_scan,
            get_local_ip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
