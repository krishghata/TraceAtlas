# TraceAtlas — Tauri & Rust KT Document

A brief knowledge transfer guide for contributors who are new to Rust and/or Tauri.
No prior Rust or Tauri experience assumed.

---

## Part 1 — Rust Basics

### What is Rust?

Rust is a systems programming language like C/C++, but with memory safety guarantees built into the compiler. It produces native binaries with no runtime or garbage collector.

In TraceAtlas, Rust is used for exactly one thing: executing OS commands (`traceroute`/`tracert`) that JavaScript cannot run in a browser/WebView context.

---

### Key Rust Concepts Used in This Project

#### Variables and types
```rust
let name: String = "google.com".to_string();  // immutable by default
let mut count: i32 = 0;                        // mutable — needs `mut`
let flag: bool = true;
```
Unlike JavaScript, types are explicit and checked at compile time. `String` is a heap-allocated string. `&str` is a string slice (a reference, no allocation).

#### Functions
```rust
fn greet(name: String) -> String {
    format!("Hello, {}!", name)   // no semicolon = return value
}
```
The last expression without a semicolon is the implicit return value. `format!` is like template literals in JS.

#### Structs (like classes, but no methods by default)
```rust
struct TracerouteResult {
    output: String,
    is_win: bool,
}
```
Structs group related data. No inheritance. Methods are added separately via `impl`.

#### Enums and Result
```rust
// Result is a built-in enum — Rust's error handling
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```
`Result<T, E>` means "either success (`Ok`) with value `T`, or failure (`Err`) with error `E`". There are no exceptions in Rust — errors are values. JavaScript sees `Ok` as a resolved Promise and `Err` as a rejected Promise when called via Tauri IPC.

#### match (like switch, but exhaustive)
```rust
match result {
    Ok(value) => println!("Got: {}", value),
    Err(e)    => println!("Error: {}", e),
}
```
`match` must handle every possible case — the compiler enforces this.

#### Attributes (the `#[...]` annotations)
```rust
#[derive(Serialize)]   // auto-generates code for JSON serialization
#[command]             // marks function as callable from JavaScript
#[cfg(target_os = "windows")]  // compile-time conditional
```
Attributes are Rust's equivalent of decorators. `#[derive(...)]` is especially common — it auto-generates trait implementations so you don't write boilerplate.

#### Ownership (brief)
Rust's most unique concept. Every value has one owner. When the owner goes out of scope, the value is freed — no garbage collector needed. You won't need to worry about this much in TraceAtlas since the Rust layer is only ~30 lines, but it's why Rust is memory-safe.

#### Macros (the `!` suffix)
```rust
format!("hello {}", name)   // like template literals
println!("debug: {}", x)    // print to console
vec![1, 2, 3]               // create a vector
```
Macros look like functions but generate code at compile time. Identifiable by the `!`.

---

### Cargo — Rust's Package Manager

Equivalent of `npm` for Rust.

| npm | Cargo equivalent |
|-----|-----------------|
| `package.json` | `Cargo.toml` |
| `node_modules/` | `~/.cargo/registry/` (shared across projects) |
| `npm install` | `cargo build` (auto-downloads on first build) |
| `npm run` | `cargo run` |
| dependencies | `[dependencies]` in `Cargo.toml` |

Cargo downloads crates (Rust packages) from [crates.io](https://crates.io) automatically on first build. No manual install step.

---

## Part 2 — Tauri Basics

### What is Tauri?

Tauri is a framework for building desktop apps using web technologies (HTML/CSS/JS) for the UI, and Rust for anything that needs OS access.

Think of it as: **your Vue/React/HTML app running in the OS's built-in browser** (WebView2 on Windows, WebKit on macOS), with a thin Rust bridge for native capabilities.

```
┌──────────────────────────────┐
│  Your web app (Vue / React)  │  ← runs in OS WebView (like a browser tab)
├──────────────────────────────┤
│  Tauri Rust bridge           │  ← handles OS calls, file system, shell, etc.
├──────────────────────────────┤
│  Operating System            │
└──────────────────────────────┘
```

---

### Tauri vs Electron

| | Tauri | Electron |
|--|-------|----------|
| UI | OS WebView (pre-installed) | Bundled Chromium |
| Installer size | ~10–15 MB | ~150 MB |
| RAM usage | ~30–60 MB | ~200+ MB |
| Backend language | Rust | Node.js |
| Learning curve | Rust required for native features | JS throughout |

---

### How JavaScript Talks to Rust — IPC

IPC (Inter-Process Communication) is how the WebView (JS) calls Rust functions.

**Step 1 — Define a command in Rust:**
```rust
#[command]
fn my_command(input: String) -> Result<String, String> {
    Ok(format!("You sent: {}", input))
}
```

**Step 2 — Register it in `run()`:**
```rust
.invoke_handler(tauri::generate_handler![my_command])
```

**Step 3 — Call it from JavaScript:**
```javascript
import { invoke } from '@tauri-apps/api/core'

const result = await invoke('my_command', { input: 'hello' })
// result === "You sent: hello"
```

The IPC call is async — `invoke()` always returns a Promise. `Ok(...)` in Rust → resolved Promise. `Err(...)` → rejected Promise (catch it with try/catch).

---

### Tauri Plugins

Plugins extend Tauri with additional native capabilities. They are registered in `run()` just like commands.

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_sql::Builder::new().build())   // adds SQLite support
```

TraceAtlas uses one plugin: **`@tauri-apps/plugin-sql`** — gives JavaScript direct SQLite access via `db.execute()` and `db.select()`.

---

### Tauri Capabilities & Permissions

Tauri 2 uses an explicit permission system — everything is denied by default. You declare what your app is allowed to do in `capabilities/default.json`.

```json
{
  "permissions": [
    "core:default",        // basic window/app operations
    "sql:allow-execute",   // db.execute() calls
    "sql:allow-select",    // db.select() calls
    "sql:allow-load"       // Database.load() to open the DB
  ]
}
```

If a permission is missing, the call silently fails or throws. This is a security feature — a compromised plugin can't do more than what's declared.

---

### `tauri.conf.json` — App Configuration

The main config file. Read at compile time by `generate_context!()`.

```json
{
  "productName": "TraceAtlas",        // app display name
  "identifier": "com.traceatlas.app", // unique app ID (like a bundle ID)
  "build": {
    "devUrl": "http://localhost:5173", // Vite dev server URL
    "beforeDevCommand": "npm run dev"  // starts Vite before opening the window
  },
  "app": {
    "windows": [{ "width": 1280, "height": 800 }]  // initial window size
  },
  "bundle": {
    "icon": ["icons/32x32.png", "icons/icon.ico"]   // app icons
  }
}
```

---

### Tauri Build System

```bash
npm run tauri dev     # dev mode: starts Vite + compiles Rust + opens window
npm run tauri build   # production: bundles into installer (.msi / .dmg / .deb)
npm run icons         # regenerate all icon sizes from src-tauri/icons/icon.svg
```

**First run is slow** (2–5 minutes) — Rust compiles all crates from source.
**Subsequent runs are fast** — only changed files recompile.
**JS/Vue changes** never trigger Rust recompilation — Vite hot-reload handles them instantly.

---

### WebView by Platform

Tauri uses the OS-native WebView — nothing bundled, nothing to install:

| Platform | Engine | Notes |
|----------|--------|-------|
| Windows 10/11 | WebView2 (Chromium-based) | Pre-installed |
| macOS | WebKit (Safari engine) | Pre-installed |
| Linux | WebKitGTK | Usually pre-installed; `sudo apt install libwebkit2gtk-4.1-dev` if missing |

---

## Part 3 — TraceAtlas Rust Layer

### Entry Points by Platform

```
Desktop (Windows / Linux / macOS):
  OS → main() in main.rs → traceatlas_lib::run() in lib.rs

Mobile (Android / iOS):
  OS → run() in lib.rs   (main.rs bypassed — no main() on mobile)
```

All logic lives in `lib.rs` — it is the single entry point that works on every platform.

---

### File Roles

| File | When it runs | What it does |
|------|-------------|-------------|
| `build.rs` | Compile time | Reads `tauri.conf.json`, generates permission code, sets linker flags |
| `main.rs` | App launch (desktop) | Hides console window on Windows release, calls `run()` |
| `lib.rs` | App runtime (all platforms) | Registers SQLite plugin, registers `run_traceroute`, opens window |

---

### `main.rs` Explained

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    traceatlas_lib::run();
}
```

- `windows_subsystem = "windows"` — hides the black CMD console on Windows release builds.
- Only active in release mode (`debug_assertions = false`). Dev mode still shows console output.

---

### `build.rs` Explained

```rust
fn main() {
    tauri_build::build()
}
```

- Cargo runs this **before** compiling everything else.
- Validates `tauri.conf.json`, generates capability code, sets platform linker flags.
- Without it, `generate_context!()` in `lib.rs` fails.

---

### `lib.rs` Explained

**Struct with auto JSON serialization:**
```rust
#[derive(Serialize)]
pub struct TracerouteResult {
    pub output: String,   // raw traceroute stdout
    pub is_win: bool,     // JS uses this to pick the right parser
}
```

**Input validation — injection prevention:**
```rust
let valid = target.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-');
```

**Platform-specific OS command:**
```rust
#[cfg(target_os = "windows")]
Command::new("tracert").args(["-d", "-h", "20", "-w", "3000", target.as_str()])

#[cfg(not(target_os = "windows"))]
Command::new("traceroute").args(["-n", "-m", "20", "-w", "3", target.as_str()])
```
Args as array = no shell injection possible. `#[cfg]` = compile-time platform switch.

**App startup:**
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .invoke_handler(tauri::generate_handler![run_traceroute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
```

---

### JavaScript ↔ Rust IPC (TraceAtlas)

```
JavaScript                           Rust
----------                           ----
invoke('run_traceroute', {target})  →  fn run_traceroute(target: String)
       ↓ awaits Promise                         ↓ Result<TracerouteResult, String>
{ output: "...", is_win: true }     ←  Ok(TracerouteResult { ... })
throws error string                 ←  Err("Invalid target")
```

---

### SQLite Access Pattern

SQLite is accessed from JavaScript, not Rust directly:

```
JavaScript  →  @tauri-apps/plugin-sql
               ↓
            Tauri SQL plugin (Rust, registered in run())
               ↓
            traceatlas.db  (OS app-data folder)
```

Requires permissions in `capabilities/default.json`:
```json
"sql:allow-execute", "sql:allow-select", "sql:allow-load"
```

---

## Quick Reference

| Concept | Rust | JavaScript equivalent |
|---------|------|-----------------------|
| Package manager | Cargo / `Cargo.toml` | npm / `package.json` |
| Packages | Crates | npm packages |
| Error handling | `Result<Ok, Err>` | try/catch / Promise reject |
| Null safety | `Option<T>` (Some/None) | `null` / `undefined` |
| Struct | `struct Foo { }` | `class Foo { }` / plain object |
| Compile-time condition | `#[cfg(target_os = "windows")]` | — (no equivalent) |
| Code generator | Macro (`format!`, `vec!`) | — (no equivalent) |
| Immutable by default | `let x = 1` | `const x = 1` |
| Mutable variable | `let mut x = 1` | `let x = 1` |
