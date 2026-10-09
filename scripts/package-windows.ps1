param(
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$releaseRoot = Join-Path $projectRoot "release"
$stageRoot = Join-Path $releaseRoot "RemotePad-windows-x64"
$archivePath = Join-Path $releaseRoot "RemotePad-v$Version-windows-x64.zip"

Push-Location $projectRoot
try {
    npm ci
    npm run build:web
    cargo test --workspace
    cargo build --release -p remotepad-server

    if (Test-Path -LiteralPath $stageRoot) {
        Remove-Item -LiteralPath $stageRoot -Recurse -Force
    }
    New-Item -ItemType Directory -Path $stageRoot -Force | Out-Null

    Copy-Item -LiteralPath (Join-Path $projectRoot "target\release\remotepad-server.exe") -Destination (Join-Path $stageRoot "RemotePad.exe")
    Copy-Item -LiteralPath (Join-Path $projectRoot "README.md") -Destination (Join-Path $stageRoot "README.md")
    Copy-Item -LiteralPath (Join-Path $projectRoot "LICENSE") -Destination (Join-Path $stageRoot "LICENSE")

    if (Test-Path -LiteralPath $archivePath) {
        Remove-Item -LiteralPath $archivePath -Force
    }
    Compress-Archive -Path (Join-Path $stageRoot "*") -DestinationPath $archivePath -CompressionLevel Optimal

    Write-Host ""
    Write-Host "RemotePad portable creado:" -ForegroundColor Green
    Write-Host $archivePath
} finally {
    Pop-Location
}

