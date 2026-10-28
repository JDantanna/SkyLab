$ErrorActionPreference = 'Stop'
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $path = Join-Path $env:SKYLAB_INSTALL_ROOT 'test-registration.json'
    $registration = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json
    $registration.DisplayVersion = $env:SKYLAB_VERSION
    $registration | ConvertTo-Json | Set-Content -LiteralPath $path -Encoding UTF8
} else {
    $key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\SkyTEM.SkyLab'
    $existing = Get-ItemProperty -LiteralPath $key
    if ([IO.Path]::GetFullPath($existing.InstallLocation) -ne [IO.Path]::GetFullPath($env:SKYLAB_INSTALL_ROOT)) { throw 'Installation registration points to another directory.' }
    Set-ItemProperty -LiteralPath $key -Name DisplayVersion -Value $env:SKYLAB_VERSION
}
