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

// ── Phase 3 ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SslCertInfo {
    pub subject:     String,
    pub issuer:      String,
    pub not_before:  String,
    pub not_after:   String,
    pub days_left:   i64,
    pub sans:        Vec<String>,
    pub serial:      String,
    pub fingerprint: String,   // SHA-256 hex
    pub protocol:    String,
    pub cipher:      String,
}

/// Inspect the TLS certificate of a host (port defaults to 443).
/// Uses openssl s_client (cross-platform) to retrieve certificate info.
#[command]
async fn ssl_inspect(host: String, port: Option<u16>) -> Result<SslCertInfo, String> {
    let valid = host.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-');
    if !valid || host.is_empty() {
        return Err("Invalid host".to_string());
    }
    let port = port.unwrap_or(443);
    let addr = format!("{}:{}", host, port);

    tauri::async_runtime::spawn_blocking(move || {

        // Fetch raw certificate via openssl s_client
        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = Command::new("openssl");
            c.args(["s_client", "-connect", &addr, "-servername", &host,
                    "-showcerts", "-brief"])
             .stdin(std::process::Stdio::null())
             .creation_flags(CREATE_NO_WINDOW);
            c
        };
        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = Command::new("openssl");
            c.args(["s_client", "-connect", &addr, "-servername", &host,
                    "-showcerts", "-brief"])
             .stdin(std::process::Stdio::null());
            c
        };

        let out = cmd.output().map_err(|e| format!("openssl not found: {}", e))?;
        let text = String::from_utf8_lossy(&out.stdout).to_string()
                 + &String::from_utf8_lossy(&out.stderr);

        // Parse key fields from openssl output
        let subject     = extract_field(&text, "subject=").unwrap_or_default();
        let issuer      = extract_field(&text, "issuer=").unwrap_or_default();
        let not_before  = extract_field(&text, "notBefore=").or_else(|| extract_field(&text, "Not Before:")).unwrap_or_default();
        let not_after   = extract_field(&text, "notAfter=").or_else(|| extract_field(&text, "Not After :")).unwrap_or_default();
        let protocol    = extract_field(&text, "Protocol  :").or_else(|| extract_field(&text, "New, ")).unwrap_or_default();
        let cipher      = extract_field(&text, "Cipher    :").or_else(|| extract_field(&text, "Cipher is ")).unwrap_or_default();
        let serial      = extract_field(&text, "serial=").or_else(|| extract_field(&text, "Serial Number:")).unwrap_or_default();
        let fingerprint = extract_field(&text, "SHA256 Fingerprint=").or_else(|| extract_field(&text, "SHA-256")).unwrap_or_default();

        // Parse SANs
        let sans: Vec<String> = text.lines()
            .find(|l| l.contains("DNS:"))
            .map(|l| l.split(',')
                .map(|s| s.trim().trim_start_matches("DNS:").to_string())
                .filter(|s| !s.is_empty())
                .collect())
            .unwrap_or_default();

        // Calculate days left from notAfter
        let days_left = parse_ssl_date(&not_after)
            .map(|exp| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                (exp - now) / 86400
            })
            .unwrap_or(0);

        Ok(SslCertInfo {
            subject, issuer, not_before, not_after, days_left,
            sans, serial, fingerprint, protocol, cipher,
        })
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}

fn extract_field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find(|l| l.contains(key))
        .map(|l| l[l.find(key).unwrap() + key.len()..].trim().to_string())
}

fn parse_ssl_date(s: &str) -> Option<i64> {
    // Try to parse openssl date format "MMM DD HH:MM:SS YYYY GMT"
    // or ISO-like "YYYY-MM-DD"
    let months = ["Jan","Feb","Mar","Apr","May","Jun",
                  "Jul","Aug","Sep","Oct","Nov","Dec"];
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 4 {
        if let Some(mi) = months.iter().position(|&m| m == parts[0]) {
            let day:  i64 = parts[1].parse().ok()?;
            let year: i64 = parts.iter().find(|p| p.len() == 4)?.parse().ok()?;
            // Approximate timestamp (good enough for day-count)
            let days_since_epoch = (year - 1970) * 365 + (year - 1969) / 4
                + mi as i64 * 30 + day;
            return Some(days_since_epoch * 86400);
        }
    }
    None
}

// ── HTTP Header Inspector ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct HttpInspectResult {
    pub url:            String,
    pub status:         u16,
    pub status_text:    String,
    pub protocol:       String,
    pub headers:        Vec<(String, String)>,
    pub redirects:      Vec<RedirectHop>,
    pub timing_ms:      u64,
    pub security_score: u8,
    pub security_notes: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct RedirectHop {
    pub url:    String,
    pub status: u16,
}

/// Fetch HTTP headers and follow redirects, reporting security score.
/// Uses curl (available on Windows 10+, macOS, Linux).
#[command]
async fn http_inspect(url: String) -> Result<HttpInspectResult, String> {
    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let start = std::time::Instant::now();

        #[cfg(target_os = "windows")]
        let out = Command::new("curl")
            .args(["-sI", "--max-redirs", "10", "-L", "--write-out",
                   "\n---CURL-INFO---\n%{http_code}\n%{url_effective}\n%{redirect_url}",
                   &url])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("curl not found: {}", e))?;

        #[cfg(not(target_os = "windows"))]
        let out = Command::new("curl")
            .args(["-sI", "--max-redirs", "10", "-L", "--write-out",
                   "\n---CURL-INFO---\n%{http_code}\n%{url_effective}\n%{redirect_url}",
                   &url])
            .output()
            .map_err(|e| format!("curl not found: {}", e))?;

        let timing_ms = start.elapsed().as_millis() as u64;
        let raw = String::from_utf8_lossy(&out.stdout).to_string();

        // Split into header blocks (each redirect has its own HTTP/ block)
        let blocks: Vec<&str> = raw.split("\n\n").collect();
        let mut redirects: Vec<RedirectHop> = Vec::new();
        let mut final_status: u16 = 0;
        let mut final_headers: Vec<(String, String)> = Vec::new();
        let mut protocol = String::new();
        let mut status_text = String::new();

        for (_bi, block) in blocks.iter().enumerate() {
            let lines: Vec<&str> = block.lines().collect();
            if lines.is_empty() { continue; }

            // Status line: HTTP/1.1 200 OK
            if lines[0].starts_with("HTTP/") {
                let parts: Vec<&str> = lines[0].splitn(3, ' ').collect();
                let status: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let stext = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
                let proto = parts.first().map(|s| s.to_string()).unwrap_or_default();

                // Intermediate redirect
                if status >= 300 && status < 400 {
                    let loc = lines.iter()
                        .find(|l| l.to_lowercase().starts_with("location:"))
                        .map(|l| l[9..].trim().to_string())
                        .unwrap_or_default();
                    redirects.push(RedirectHop { url: loc, status });
                } else {
                    final_status = status;
                    status_text  = stext;
                    protocol     = proto;
                    final_headers = lines[1..].iter()
                        .filter_map(|l| {
                            let ci = l.find(':')?;
                            Some((l[..ci].trim().to_string(), l[ci+1..].trim().to_string()))
                        })
                        .collect();
                }
            }
        }

        // Security score
        let (score, notes) = score_headers(&final_headers);

        Ok(HttpInspectResult {
            url:            url.clone(),
            status:         final_status,
            status_text,
            protocol,
            headers:        final_headers,
            redirects,
            timing_ms,
            security_score: score,
            security_notes: notes,
        })
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}

fn score_headers(headers: &[(String, String)]) -> (u8, Vec<String>) {
    let mut score: i32 = 100;
    let mut notes = Vec::new();
    let hmap: std::collections::HashMap<String, &str> = headers.iter()
        .map(|(k, v)| (k.to_lowercase(), v.as_str()))
        .collect();

    let checks: &[(&str, i32, &str)] = &[
        ("strict-transport-security", 20, "Missing HSTS — downgrade attacks possible"),
        ("content-security-policy",   15, "Missing CSP — XSS risk"),
        ("x-frame-options",           10, "Missing X-Frame-Options — clickjacking risk"),
        ("x-content-type-options",    10, "Missing X-Content-Type-Options"),
        ("referrer-policy",            5, "Missing Referrer-Policy"),
        ("permissions-policy",         5, "Missing Permissions-Policy"),
    ];
    for (header, penalty, note) in checks {
        if !hmap.contains_key(*header) {
            score -= penalty;
            notes.push(note.to_string());
        }
    }
    if let Some(v) = hmap.get("x-powered-by") {
        score -= 5;
        notes.push(format!("X-Powered-By exposes server info: {}", v));
    }
    if let Some(v) = hmap.get("server") {
        if v.len() > 7 { // bare "Server:" ok, detailed version not ok
            score -= 5;
            notes.push(format!("Server header exposes version info: {}", v));
        }
    }
    (score.max(0) as u8, notes)
}

// ── Wake on LAN ───────────────────────────────────────────────────────────────

/// Send a Wake-on-LAN magic packet to the given MAC address.
/// Broadcasts a UDP packet to 255.255.255.255:9 containing the magic payload.
#[command]
fn wake_on_lan(mac: String) -> Result<String, String> {
    use std::net::UdpSocket;

    // Parse MAC: accept XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX
    let clean: String = mac.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() != 12 {
        return Err(format!("Invalid MAC address: {}", mac));
    }
    let bytes: Vec<u8> = (0..6)
        .map(|i| u8::from_str_radix(&clean[i*2..i*2+2], 16)
            .map_err(|e| format!("MAC parse error: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    // Magic packet: 6× 0xFF then 16× MAC
    let mut packet = vec![0xFF_u8; 6];
    for _ in 0..16 { packet.extend_from_slice(&bytes); }

    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    sock.set_broadcast(true).map_err(|e| e.to_string())?;
    sock.send_to(&packet, "255.255.255.255:9").map_err(|e| e.to_string())?;

    Ok(format!("Magic packet sent to {}", mac))
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
            get_local_ip,
            ssl_inspect,
            http_inspect,
            wake_on_lan
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
