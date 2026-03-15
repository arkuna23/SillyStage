[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Target
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Write-Usage {
    @"
Usage: ./scripts/package-app.ps1 [-Target <triple> ...]

Builds the web frontend and the ss-app backend, assembles a self-contained
release directory, and emits a platform archive under dist\.

Examples:
  ./scripts/package-app.ps1
  ./scripts/package-app.ps1 -Target x86_64-pc-windows-msvc
  ./scripts/package-app.ps1 -Target x86_64-pc-windows-msvc -Target x86_64-pc-windows-gnu
"@
}

function Require-Command {
    param([Parameter(Mandatory = $true)][string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "missing required command: $Name"
    }
}

function Get-HostTarget {
    $output = & rustc -vV
    if ($LASTEXITCODE -ne 0) {
        throw "failed to query rustc host target"
    }

    foreach ($line in $output) {
        if ($line -like "host:*") {
            return $line.Substring(5).Trim()
        }
    }

    throw "could not determine rustc host target"
}

function Ensure-RustTarget {
    param([Parameter(Mandatory = $true)][string]$RustTarget)

    $installed = & rustup target list --installed
    if ($LASTEXITCODE -ne 0) {
        throw "failed to query installed rust targets"
    }

    if (-not ($installed | Where-Object { $_ -eq $RustTarget })) {
        throw "missing Rust target: $RustTarget`ninstall it first with: rustup target add $RustTarget"
    }
}

function Binary-NameFor {
    param([Parameter(Mandatory = $true)][string]$RustTarget)

    if ($RustTarget -like "*windows*") {
        return "ss-app.exe"
    }

    return "ss-app"
}

function Write-PackagedConfig {
    param([Parameter(Mandatory = $true)][string]$PackageDir)

    @'
[server]
listen = "127.0.0.1:8080"
open_browser = true

[store]
backend = "fs"
root = "data"

[frontend]
enabled = true
mount_path = "/"
static_dir = "webapp"
'@ | Set-Content -Path (Join-Path $PackageDir "ss-app.toml") -Encoding UTF8
}

function Write-ReleaseNotes {
    param(
        [Parameter(Mandatory = $true)][string]$PackageDir,
        [Parameter(Mandatory = $true)][string]$BinaryName
    )

    @"
SillyStage packaged release

1. Keep the directory structure intact.
2. Run .\$BinaryName
3. The app will serve the bundled web frontend and store local data in .\data

Config:
- ss-app.toml is auto-discovered next to the executable.
- webapp\ contains the prebuilt frontend assets.
- data\ is created on first run if needed.
"@ | Set-Content -Path (Join-Path $PackageDir "README.txt") -Encoding UTF8
}

function Package-Target {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$DistRoot,
        [Parameter(Mandatory = $true)][string]$RustTarget
    )

    Ensure-RustTarget -RustTarget $RustTarget

    $binaryName = Binary-NameFor -RustTarget $RustTarget
    $packageName = "sillystage-$RustTarget"
    $packageDir = Join-Path $DistRoot $packageName
    $archivePath = Join-Path $DistRoot "$packageName.zip"

    if (Test-Path $packageDir) {
        Remove-Item -Recurse -Force $packageDir
    }
    if (Test-Path $archivePath) {
        Remove-Item -Force $archivePath
    }

    New-Item -ItemType Directory -Path (Join-Path $packageDir "webapp") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $packageDir "data") -Force | Out-Null

    & cargo build --release -p ss-app --target $RustTarget
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed for target $RustTarget"
    }

    $binaryPath = Join-Path $RepoRoot "target/$RustTarget/release/$binaryName"
    if (-not (Test-Path $binaryPath)) {
        throw "built binary not found: $binaryPath"
    }

    Copy-Item -Path $binaryPath -Destination (Join-Path $packageDir $binaryName)
    Copy-Item -Path (Join-Path $RepoRoot "webapp/dist/*") -Destination (Join-Path $packageDir "webapp") -Recurse -Force

    Write-PackagedConfig -PackageDir $packageDir
    Write-ReleaseNotes -PackageDir $packageDir -BinaryName $binaryName

    Compress-Archive -Path $packageDir -DestinationPath $archivePath -CompressionLevel Optimal
    Write-Host "packaged $RustTarget -> $archivePath"
}

if ($Target.Count -gt 0 -and $Target[0] -in @("-h", "--help", "/?")) {
    Write-Usage
    exit 0
}

Require-Command -Name cargo
Require-Command -Name rustup
Require-Command -Name pnpm
Require-Command -Name rustc

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$distRoot = Join-Path $repoRoot "dist"

if ($Target.Count -eq 0) {
    $Target = @(Get-HostTarget)
}

New-Item -ItemType Directory -Path $distRoot -Force | Out-Null

Push-Location (Join-Path $repoRoot "webapp")
try {
    & pnpm build
    if ($LASTEXITCODE -ne 0) {
        throw "pnpm build failed"
    }
} finally {
    Pop-Location
}

foreach ($rustTarget in $Target) {
    Package-Target -RepoRoot $repoRoot -DistRoot $distRoot -RustTarget $rustTarget
}
