# E-Soccer Battle V3 — Windows Setup (run as Administrator)
# Usage:右键 → Run with PowerShell

param([switch]$SkipRust, [switch]$SkipNode)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

Write-Host "⚽ E-Soccer Battle V3 — Setup`n" -ForegroundColor Cyan

# ── 1. Rust ──────────────────────────────────────────────────────────────
if (-not $SkipRust) {
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Host "[1/4] Installing Rust..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
        Start-Process -Wait -FilePath "$env:TEMP\rustup-init.exe" -ArgumentList "-y --default-toolchain stable"
        $env:Path += ";$env:USERPROFILE\.cargo\bin"
        Write-Host "  ✅ Rust installed`n" -ForegroundColor Green
    } else {
        Write-Host "[1/4] Rust $(rustc --version) — OK`n" -ForegroundColor Green
    }
}

# ── 2. Node.js ───────────────────────────────────────────────────────────
if (-not $SkipNode) {
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        Write-Host "[2/4] Installing Node.js..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri "https://nodejs.org/dist/v22.14.0/node-v22.14.0-x64.msi" -OutFile "$env:TEMP\node-installer.msi"
        Start-Process -Wait -FilePath "msiexec.exe" -ArgumentList "/i `"$env:TEMP\node-installer.msi`" /quiet"
        $env:Path += ";C:\Program Files\nodejs"
        Write-Host "  ✅ Node.js installed`n" -ForegroundColor Green
    } else {
        Write-Host "[2/4] Node $(node --version) — OK`n" -ForegroundColor Green
    }
}

# ── 3. Visual Studio Build Tools (C++ for whisper-rs) ────────────────────
Write-Host "[3/4] Checking Visual Studio Build Tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasCpp = $false
if (Test-Path $vsWhere) {
    $installPath = & $vsWhere -latest -property installationPath 2>$null
    if ($installPath) {
        $cppComponent = & $vsWhere -latest -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        $hasCpp = [bool]$cppComponent
    }
}

if (-not $hasCpp) {
    Write-Host "  Installing Visual Studio Build Tools (C++ workload)..." -ForegroundColor Yellow
    $bootstrapper = "$env:TEMP\vs_buildtools.exe"
    Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile $bootstrapper
    
    # Install C++ build tools silently
    $args = @(
        "--quiet", "--wait", "--norestart",
        "--add", "Microsoft.VisualStudio.Workload.VCTools",
        "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22621",
        "--includeRecommended"
    )
    Start-Process -Wait -FilePath $bootstrapper -ArgumentList $args
    Write-Host "  ✅ VS Build Tools installed`n" -ForegroundColor Green
} else {
    Write-Host "  ✅ VS Build Tools — OK`n" -ForegroundColor Green
}

# ── 4. CMake ─────────────────────────────────────────────────────────────
if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
    Write-Host "[4/4] Installing CMake..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://github.com/Kitware/CMake/releases/download/v3.31.6/cmake-3.31.6-windows-x86_64.msi" -OutFile "$env:TEMP\cmake-installer.msi"
    Start-Process -Wait -FilePath "msiexec.exe" -ArgumentList "/i `"$env:TEMP\cmake-installer.msi`" /quiet ADD_CMAKE_TO_PATH=System"
    $env:Path += ";C:\Program Files\CMake\bin"
    Write-Host "  ✅ CMake installed`n" -ForegroundColor Green
} else {
    Write-Host "[4/4] CMake $(cmake --version | Select-Object -First 1) — OK`n" -ForegroundColor Green
}

# ── 5. npm install ───────────────────────────────────────────────────────
Write-Host "[5/5] npm install..." -ForegroundColor Yellow
Set-Location $PSScriptRoot
npm install
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✅ Dependencies installed`n" -ForegroundColor Green
} else {
    Write-Host "  ❌ npm install failed`n" -ForegroundColor Red
    exit 1
}

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "Setup completo! 🎉`n" -ForegroundColor Green
Write-Host "Para rodar o projeto:" -ForegroundColor White
Write-Host "  npm run tauri dev`n" -ForegroundColor Yellow
Write-Host "Primeiro build pode demorar ~5-10 min (compila Whisper C++)." -ForegroundColor DarkGray
