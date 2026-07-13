param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Together",
    [switch]$AddToPath
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$ReleaseExe = Join-Path $RepoRoot "target\release\together.exe"
$DebugExe = Join-Path $RepoRoot "target\debug\together.exe"

if (Test-Path $ReleaseExe) {
    $SourceExe = $ReleaseExe
} elseif (Test-Path $DebugExe) {
    $SourceExe = $DebugExe
} else {
    Write-Host "Building Together release binary..."
    Push-Location $RepoRoot
    try {
        cargo build -p cli --release
    } finally {
        Pop-Location
    }
    $SourceExe = $ReleaseExe
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -LiteralPath $SourceExe -Destination (Join-Path $InstallDir "together.exe") -Force

if ($AddToPath) {
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (($UserPath -split ";") -notcontains $InstallDir) {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
        Write-Host "Added $InstallDir to user PATH. Open a new terminal to use it."
    }
}

$InstalledExe = Join-Path $InstallDir "together.exe"
Write-Host "Installed Together to $InstalledExe"
& $InstalledExe doctor
Write-Host ""
Write-Host "Launch with: together"
