$ErrorActionPreference = 'Stop'
$installRoot = [IO.Path]::GetFullPath($env:SKYLAB_INSTALL_ROOT)
$appPath = Join-Path $installRoot 'SkyLab-Rust.exe'
$uninstallPath = Join-Path $installRoot 'Uninstall.exe'
$shell = New-Object -ComObject WScript.Shell
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $programsFolder = Join-Path $installRoot 'test-shortcuts'
    $desktopFolder = $programsFolder
} else {
    $programsFolder = Join-Path ([Environment]::GetFolderPath('Programs')) 'SkyTEM'
    $desktopFolder = [Environment]::GetFolderPath('Desktop')
}
New-Item -ItemType Directory -Path $programsFolder -Force | Out-Null
$menuLink = Join-Path $programsFolder 'SkyLab.lnk'
$shortcut = $shell.CreateShortcut($menuLink)
$shortcut.TargetPath = $appPath
$shortcut.WorkingDirectory = $installRoot
$shortcut.Description = 'SkyTEM SkyLab - Planejamento de aerolevantamento'
$shortcut.Save()
if ($env:SKYLAB_DESKTOP -eq '1') {
    $shortcut = $shell.CreateShortcut((Join-Path $desktopFolder 'SkyTEM SkyLab.lnk'))
    $shortcut.TargetPath = $appPath
    $shortcut.WorkingDirectory = $installRoot
    $shortcut.Save()
}
$registration = @{
    DisplayName = 'SkyTEM SkyLab'
    DisplayVersion = $env:SKYLAB_VERSION
    Publisher = 'SkyTEM - uso interno'
    InstallLocation = $installRoot
    DisplayIcon = $appPath
    UninstallString = ('"' + $uninstallPath + '" --uninstall')
}
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $registration | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $installRoot 'test-registration.json') -Encoding UTF8
} else {
    $key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\SkyTEM.SkyLab'
    $existing = Get-ItemProperty -LiteralPath $key -ErrorAction SilentlyContinue
    if ($existing -and $existing.InstallLocation -ne $installRoot) { throw 'Registro existente aponta para outra pasta.' }
    New-Item -Path $key -Force | Out-Null
    foreach ($name in $registration.Keys) { New-ItemProperty -LiteralPath $key -Name $name -Value $registration[$name] -PropertyType String -Force | Out-Null }
    New-ItemProperty -LiteralPath $key -Name NoModify -Value 1 -PropertyType DWord -Force | Out-Null
    New-ItemProperty -LiteralPath $key -Name NoRepair -Value 1 -PropertyType DWord -Force | Out-Null
}
