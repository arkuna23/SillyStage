# Installation Guide

SillyStage supports two installation paths:

- install from a release package
- build from source

Release packages are better for quick trials and local use. Building from source is better for development, debugging, and contributions.

For most users, start with a release package.

## 1. Install from a Release

### 1.1 Download the Package

- Open GitHub Releases
- Download the archive for your platform
- Extract it locally without changing the directory structure

Typical artifacts:

- Linux: `sillystage-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `sillystage-x86_64-pc-windows-msvc.zip`

### 1.2 Understand the Extracted Layout

Release packages usually include:

- the app binary
- `ss-app.toml`
- `webapp/`
- `data/`, which can be created automatically on first launch

Notes:

- `ss-app.toml` is the default config file
- `webapp/` contains the built frontend assets
- `data/` stores local data

### 1.3 Start the Application

Linux:

```bash
./ss-app
```

On Windows:

```powershell
.\ss-app.exe
```

## 2. Build from Source

### 2.1 Prerequisites

- Rust toolchain
- Cargo
- `pnpm`
- `just`, recommended

### 2.2 Clone the Repository

```bash
git clone <repo-url>
cd SillyStage
```

### 2.3 Install Frontend Dependencies

Application frontend:

```bash
cd webapp
pnpm install
cd ..
```

### 2.4 Start in Development Mode

Recommended:

```bash
just dev
```

This starts:

- the Rust backend
- the `webapp` development server

If you only want the backend:

```bash
just backend
```
