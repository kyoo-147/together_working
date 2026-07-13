param(
    [string]$OutputDir = "dist"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$DistDir = Join-Path $RepoRoot $OutputDir
$PackageRoot = Join-Path $DistDir "together-windows-x64"
$ZipPath = Join-Path $DistDir "together-windows-x64.zip"

Push-Location $RepoRoot
try {
    cargo build -p cli --release
} finally {
    Pop-Location
}

Remove-Item -LiteralPath $PackageRoot -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $PackageRoot | Out-Null

Copy-Item -LiteralPath (Join-Path $RepoRoot "target\release\together.exe") -Destination $PackageRoot -Force
Copy-Item -LiteralPath (Join-Path $RepoRoot "README.md") -Destination $PackageRoot -Force
Copy-Item -LiteralPath (Join-Path $RepoRoot "INSTALL.md") -Destination $PackageRoot -Force
if (Test-Path (Join-Path $RepoRoot "LICENSE")) {
    Copy-Item -LiteralPath (Join-Path $RepoRoot "LICENSE") -Destination $PackageRoot -Force
}

Remove-Item -LiteralPath $ZipPath -Force -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $PackageRoot "*") -DestinationPath $ZipPath -Force

Write-Host "Created $ZipPath"
