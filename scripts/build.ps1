$ErrorActionPreference = 'Stop'
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    & "$PSScriptRoot/test-native-startup.ps1"
    node --test tests/interactions.cjs tests/session.cjs tests/window.cjs tests/edit-tools.cjs tests/hotkeys-tiles.cjs tests/layer-model.cjs tests/pan.cjs tests/panels.cjs tests/import-trim.cjs tests/cartography.cjs
    if ($LASTEXITCODE -ne 0) { throw 'Testes de interação falharam.' }
    cargo test --offline
    if ($LASTEXITCODE -ne 0) { throw 'Testes falharam.' }
    cargo build --release --offline --bin skylab-rust
    if ($LASTEXITCODE -ne 0) { throw 'Compilação do aplicativo falhou.' }
    New-Item -ItemType Directory -Path payload,dist -Force | Out-Null
    Copy-Item -LiteralPath target/release/skylab-rust.exe -Destination payload/SkyLab-Rust.exe -Force
    cargo build --release --offline --features installer --bin skylab-setup
    if ($LASTEXITCODE -ne 0) { throw 'Compilação do instalador falhou.' }
    Copy-Item -LiteralPath target/release/skylab-setup.exe -Destination dist/SkyTEM-SkyLab-Setup.exe -Force
    Get-FileHash -LiteralPath dist/SkyTEM-SkyLab-Setup.exe -Algorithm SHA256
} finally { Pop-Location }
