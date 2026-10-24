# Build Rusci Scope and copy it into Resolume's plugin folder.
#   powershell -ExecutionPolicy Bypass -File install.ps1
# Restart Resolume afterwards; it only scans plugins at startup.

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot

cargo build --release --manifest-path "$root\Cargo.toml"
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

# MyDocuments follows OneDrive folder redirection, unlike $HOME\Documents.
$docs = [Environment]::GetFolderPath("MyDocuments")
$dest = Join-Path $docs "Resolume Arena\Extra Effects"
New-Item -ItemType Directory -Force -Path $dest | Out-Null

$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "$root\target" }
Copy-Item "$target\release\rusci_scope.dll" (Join-Path $dest "RusciScope.dll") -Force
Write-Host "Installed to $dest\RusciScope.dll"
