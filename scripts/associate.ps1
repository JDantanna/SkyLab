$ErrorActionPreference = 'Stop'
$installRoot = [IO.Path]::GetFullPath($env:SKYLAB_INSTALL_ROOT)
$appPath = Join-Path $installRoot 'SkyLab-Rust.exe'
$command = '"' + $appPath + '" --open "%1"'
$types = @(@{Extension='.mis';ProgId='SkyTEM.SkyLab.MIS';Name='SkyLab Mission'},@{Extension='.skylab';ProgId='SkyTEM.SkyLab.Project';Name='SkyLab Project'})
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $json = @{Extension='.mis';ProgId=$types[0].ProgId;ProjectExtension='.skylab';ProjectProgId=$types[1].ProgId;ProjectIcon=('"'+$appPath+'",0');Command=$command;InstallRoot=$installRoot} | ConvertTo-Json
    [IO.File]::WriteAllText((Join-Path $installRoot 'test-association.json'),$json,(New-Object Text.UTF8Encoding($false)))
    return
}
$classes = 'HKCU:\Software\Classes'
$capabilities='HKCU:\Software\SkyTEM\SkyLab\Capabilities'
foreach ($type in $types) {
    $prior=Get-ItemProperty -LiteralPath (Join-Path $classes $type.ProgId) -ErrorAction SilentlyContinue
    if ($prior -and $prior.SkyLabOwner -ne $installRoot) {throw 'File registration belongs to another installation.'}
}
$prior=Get-ItemProperty -LiteralPath $capabilities -ErrorAction SilentlyContinue
if ($prior -and $prior.SkyLabOwner -ne $installRoot) {throw 'Application registration belongs to another installation.'}
New-Item -Path $capabilities -Force | Out-Null
New-ItemProperty -LiteralPath $capabilities -Name ApplicationName -Value 'SkyLab' -PropertyType String -Force | Out-Null
New-ItemProperty -LiteralPath $capabilities -Name ApplicationDescription -Value 'SkyTEM Survey Laboratory' -PropertyType String -Force | Out-Null
New-ItemProperty -LiteralPath $capabilities -Name SkyLabOwner -Value $installRoot -PropertyType String -Force | Out-Null
New-Item -Path (Join-Path $capabilities 'FileAssociations') -Force | Out-Null
foreach ($type in $types) {
    $progKey=Join-Path $classes $type.ProgId
    New-Item -Path $progKey -Force | Out-Null
    Set-Item -LiteralPath $progKey -Value $type.Name
    New-ItemProperty -LiteralPath $progKey -Name SkyLabOwner -Value $installRoot -PropertyType String -Force | Out-Null
    foreach ($part in @('DefaultIcon','shell','shell\open','shell\open\command')) {New-Item -Path (Join-Path $progKey $part) -Force | Out-Null}
    Set-Item -LiteralPath (Join-Path $progKey 'DefaultIcon') -Value ('"' + $appPath + '",0')
    Set-Item -LiteralPath (Join-Path $progKey 'shell\open\command') -Value $command
    $extension=Join-Path $classes $type.Extension
    New-Item -Path $extension -Force | Out-Null
    # Protected UserChoice and existing defaults remain under the user's control.
    if (-not (Get-Item -LiteralPath $extension).GetValue('')) {Set-Item -LiteralPath $extension -Value $type.ProgId}
    $openWith=Join-Path $extension 'OpenWithProgids'
    New-Item -Path $openWith -Force | Out-Null
    New-ItemProperty -LiteralPath $openWith -Name $type.ProgId -Value '' -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath (Join-Path $capabilities 'FileAssociations') -Name $type.Extension -Value $type.ProgId -PropertyType String -Force | Out-Null
}
New-Item -Path 'HKCU:\Software\RegisteredApplications' -Force | Out-Null
New-ItemProperty -LiteralPath 'HKCU:\Software\RegisteredApplications' -Name 'SkyTEM SkyLab' -Value 'Software\SkyTEM\SkyLab\Capabilities' -PropertyType String -Force | Out-Null
Add-Type -Namespace SkyLab -Name AssociationNotify -MemberDefinition '[System.Runtime.InteropServices.DllImport("shell32.dll")] public static extern void SHChangeNotify(uint eventId, uint flags, System.IntPtr item1, System.IntPtr item2);'
[SkyLab.AssociationNotify]::SHChangeNotify(0x08000000,0,[IntPtr]::Zero,[IntPtr]::Zero)
