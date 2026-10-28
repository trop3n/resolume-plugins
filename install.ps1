# Build plugins and copy them into Resolume's plugin folder.
#   powershell -ExecutionPolicy Bypass -File install.ps1              # all plugins
#   powershell -ExecutionPolicy Bypass -File install.ps1 rusci-scope  # one plugin
#   add -Avenue to install for Resolume Avenue instead of Arena
# Restart Resolume afterwards; it only scans plugins at startup.

param(
    [string[]]$Plugins,
    [switch]$Avenue
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot

if (-not $Plugins) {
    $Plugins = Get-ChildItem -Directory (Join-Path $root "plugins") |
        Where-Object { Test-Path (Join-Path $_.FullName "Cargo.toml") } |
        ForEach-Object { $_.Name }
}

# MyDocuments follows OneDrive folder redirection, unlike $HOME\Documents.
$docs = [Environment]::GetFolderPath("MyDocuments")
$app = if ($Avenue) { "Resolume Avenue" } else { "Resolume Arena" }
$dest = Join-Path $docs "$app\Extra Effects"
New-Item -ItemType Directory -Force -Path $dest | Out-Null

$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "$root\target" }

foreach ($plugin in $Plugins) {
    $manifest = Join-Path $root "plugins\$plugin\Cargo.toml"
    if (-not (Test-Path $manifest)) { throw "No $manifest" }
    $toml = Get-Content $manifest -Raw

    $package = [regex]::Match($toml, '(?ms)^\[package\].*?^name = "([^"]+)"').Groups[1].Value
    $lib = [regex]::Match($toml, '(?ms)^\[lib\].*?^name = "([^"]+)"').Groups[1].Value
    if (-not $lib) { $lib = $package -replace '-', '_' }
    $name = [regex]::Match($toml, '(?m)^file-name = "([^"]+)"').Groups[1].Value
    if (-not $name) { throw "$manifest has no [package.metadata.resolume] file-name" }

    cargo build --release --lib -p $package --manifest-path "$root\Cargo.toml"
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed for $plugin" }

    Copy-Item "$target\release\$lib.dll" (Join-Path $dest "$name.dll") -Force
    Write-Host "Installed $plugin to $dest\$name.dll"
}
