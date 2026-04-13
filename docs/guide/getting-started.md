# Getting Started

## Prerequisites

| Tool | Minimum version | Install |
|------|----------------|---------|
| **Rust** + Cargo | 1.78 | [rustup.rs](https://rustup.rs) |
| **Node.js** | 20 | [nodejs.org](https://nodejs.org) |
| **Tauri CLI** | 2.x | `cargo install tauri-cli` |
| **WebView2** (Windows only) | latest | Ships with Windows 11; [download](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) for Win 10 |

## Clone the repository

```bash
git clone https://github.com/lichman0405/lightpdf.git
cd lightpdf
```

## Run the desktop app (development)

```bash
cd pdfops-gui
npm install
npm run tauri dev
```

The window opens automatically. Any change to the React frontend hot-reloads via Vite; Rust changes trigger a recompile.

## Build a production installer

```bash
cd pdfops-gui
npm run tauri build
```

Outputs to `src-tauri/target/release/bundle/`:

| Platform | Artifact |
|----------|----------|
| Windows | `.msi` + `.exe` (NSIS) |
| macOS | `.dmg` + `.app` |
| Linux | `.deb` + `.AppImage` |

## Run the MCP server

```bash
# From the repository root
cargo run -p pdfops-mcp
```

The server communicates via **stdin/stdout** using the MCP protocol. See the [MCP Server guide](./mcp-server) for integration instructions.

## Run all tests

```bash
cargo test --workspace
```
