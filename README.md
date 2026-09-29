# WeMod Pro Unlocker

<div align="center">
  <h3>Automated patching tool for WeMod</h3>
  <p>Unlocks WeMod Pro functionality locally with lightweight Rust & Go binaries.</p>
</div>

## 📖 How to Use

Follow these simple steps to use the unlocker:

### 1️⃣ Step 1: Install Wand / WeMod
Make sure you have Wand (or WeMod) installed from the official/original source:
- 🌐 Official Website: [https://wand.com](https://wand.com)
- 📥 Official Download: [https://wand.com/download](https://wand.com/download)

### 2️⃣ Step 2: Log In to Your Account
- Open the Wand / WeMod application.
- Log in normally with your account credentials.

### 3️⃣ Step 3: Download & Run the Unlocker
1. Download the latest release: 👉 **[Download Latest Release (`wemod-pro-unlocker-windows-x64.zip`)](https://github.com/Allain-afk/wemod-pro-unlocker/releases/latest)**
2. Extract the downloaded `.zip` file.
3. Run `wemod-pro-unlocker.exe`.
   - *Note: The tool will automatically locate your installation, close running processes, and apply the Pro patch.*

### 4️⃣ Step 4: Launch Wand / WeMod
- Open Wand / WeMod again. Your Pro features are now enabled!

> [!TIP]
> **After App Updates**: Whenever Wand/WeMod automatically updates to a new version, simply run `wemod-pro-unlocker.exe` again to patch the new version.

---

## ⚙️ Advanced Configuration (CLI Options)

If you installed WeMod in a custom location or want to pass custom parameters, you can run the executable from CMD / PowerShell with the following options:

| Flag / Option | Description | Example |
|---|---|---|
| `--wemod-dir <path>` | Path to custom WeMod/Wand directory (Default: `%LOCALAPPDATA%\WeMod`) | `--wemod-dir "D:\Games\WeMod"` |
| `--wemod-version <version>` | Specific version to patch (Default: latest detected version) | `--wemod-version "8.3.6"` |
| `--account <json>` | Custom account metadata override | `--account "username:'pro',email:'test@test.com'"` |
| `-no-update` / `-offline` | Skip checking for unlocker updates on launch | `-offline` |
| `-v` | Display the unlocker version | `-v` |

---

## 📦 Features & Architecture

This repository consists of two modular components built with system languages for high efficiency and tiny binary size:

- **CLI (`cli/`)**: Written in **Rust (2021 edition)**. Patches WeMod files and application assets.
- **Updater (`updater/`)**: Written in **Go**. Handles automatic process management and binary downloads from GitHub Releases.

---

## 🛠️ Building From Source Locally

### Prerequisites

To compile the binaries locally on Windows, ensure you have the following installed:

1. **Rust Toolchain (Cargo & rustc)** (v1.65+ or latest stable):
   - Install via [rustup.rs](https://rustup.rs/)
2. **Go Compiler** (v1.19+):
   - Install via [go.dev](https://go.dev/dl/)
3. **PowerShell** (5.1+ or PowerShell Core 7+)

### One-Click Build Command

Run the automated build script from the repository root:

#### Using PowerShell:
```powershell
.\build.ps1
```

#### Using Command Prompt (CMD):
```cmd
build.bat
```

### Manual Build Instructions

If you prefer building each project individually:

#### 1. Build CLI (Rust):
```cmd
cd cli
cargo build --release
```
*Binary output: `cli/target/release/wemod-pro-unlocker.exe`*

#### 2. Build Updater (Go):
```cmd
cd updater
go build -ldflags="-s -w" -o ..\dist\updater.exe .
```
*Binary output: `dist/updater.exe`*

---

## 🚀 CI/CD Automated Binary Releases

GitHub Actions workflow is configured under [`.github/workflows/release.yml`](.github/workflows/release.yml).

- **Triggers**:
  - Pushing a new Git version tag (e.g. `v0.14.0`): `git tag v0.14.0 && git push origin v0.14.0`
  - Manual trigger via **Actions tab > Release Build & Publish > Run workflow** (`workflow_dispatch`).
- **Pipeline Actions**:
  - Compiles Rust CLI with size & performance optimizations (`opt-level = "z"`, LTO enabled, stripped symbols).
  - Compiles Go Updater with stripped symbol flags (`-ldflags="-s -w"`).
  - Packages binaries into a ZIP archive.
  - Automatically creates a GitHub Release and attaches artifacts.

---

## 📄 License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.
