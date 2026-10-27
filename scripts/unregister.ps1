$ErrorActionPreference = 'Stop'
$installRoot = [IO.Path]::GetFullPath($env:SKYLAB_INSTALL_ROOT)
$appPath = Join-Path $installRoot 'SkyLab-Rust.exe'
$shell = New-Object -ComObject WScript.Shell
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $programsFolder = Join-Path $installRoot 'test-shortcuts'
    $desktopFolder = $programsFolder
} else {
    $programsFolder = Join-Path ([Environment]::GetFolderPath('Programs')) 'SkyTEM'
    $desktopFolder = [Environment]::GetFolderPath('Desktop')
}
foreach ($linkPath in @((Join-Path $programsFolder 'SkyLab.lnk'), (Join-Path $desktopFolder 'SkyTEM SkyLab.lnk'))) {
    if (Test-Path -LiteralPath $linkPath) {
        $shortcut = $shell.CreateShortcut($linkPath)
        if ($shortcut.TargetPath -eq $appPath) { Remove-Item -LiteralPath $linkPath -Force }
    }
}
if ((Test-Path -LiteralPath $programsFolder) -and -not (Get-ChildItem -LiteralPath $programsFolder -Force | Select-Object -First 1)) { Remove-Item -LiteralPath $programsFolder }
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $testRecord = Join-Path $installRoot 'test-registration.json'
    if (Test-Path -LiteralPath $testRecord) { Remove-Item -LiteralPath $testRecord }
} else {
    $key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\SkyTEM.SkyLab'
    $existing = Get-ItemProperty -LiteralPath $key -ErrorAction SilentlyContinue
    if ($existing -and $existing.InstallLocation -eq $installRoot) { Remove-Item -LiteralPath $key }
}
