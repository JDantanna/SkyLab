$ErrorActionPreference = 'Stop'
$installRoot = [IO.Path]::GetFullPath($env:SKYLAB_INSTALL_ROOT)
$appPath = Join-Path $installRoot 'SkyLab-Rust.exe'
$progId = 'SkyTEM.SkyLab.MIS'
$command = '"' + $appPath + '" --open "%1"'
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $json = @{Extension='.mis';ProgId=$progId;Command=$command;InstallRoot=$installRoot} | ConvertTo-Json
    [IO.File]::WriteAllText((Join-Path $installRoot 'test-association.json'),$json,(New-Object Text.UTF8Encoding($false)))
    return
}
$classes = 'HKCU:\Software\Classes'
$progKey = Join-Path $classes $progId
$prior = Get-ItemProperty -LiteralPath $progKey -ErrorAction SilentlyContinue
if ($prior -and $prior.SkyLabOwner -ne $installRoot) { throw 'MIS registration belongs to another installation.' }
New-Item -Path $progKey -Force | Out-Null
Set-Item -LiteralPath $progKey -Value 'SkyLab Mission'
New-ItemProperty -LiteralPath $progKey -Name SkyLabOwner -Value $installRoot -PropertyType String -Force | Out-Null
foreach ($part in @('DefaultIcon','shell','shell\open','shell\open\command')) {New-Item -Path (Join-Path $progKey $part) -Force | Out-Null}
Set-Item -LiteralPath (Join-Path $progKey 'DefaultIcon') -Value ('"' + $appPath + '",0')
Set-Item -LiteralPath (Join-Path $progKey 'shell\open\command') -Value $command
$extension = Join-Path $classes '.mis'
New-Item -Path $extension -Force | Out-Null
# Respect existing explicit Windows defaults and the protected UserChoice entry.
if (-not (Get-Item -LiteralPath $extension).GetValue('')) {Set-Item -LiteralPath $extension -Value $progId}
$openWith = Join-Path $extension 'OpenWithProgids'
New-Item -Path $openWith -Force | Out-Null
New-ItemProperty -LiteralPath $openWith -Name $progId -Value '' -PropertyType String -Force | Out-Null
$capabilities='HKCU:\Software\SkyTEM\SkyLab\Capabilities'
New-Item -Path $capabilities -Force | Out-Null
New-ItemProperty -LiteralPath $capabilities -Name ApplicationName -Value 'SkyLab' -PropertyType String -Force | Out-Null
New-ItemProperty -LiteralPath $capabilities -Name ApplicationDescription -Value 'SkyTEM Survey Laboratory' -PropertyType String -Force | Out-Null
New-ItemProperty -LiteralPath $capabilities -Name SkyLabOwner -Value $installRoot -PropertyType String -Force | Out-Null
New-Item -Path (Join-Path $capabilities 'FileAssociations') -Force | Out-Null
New-ItemProperty -LiteralPath (Join-Path $capabilities 'FileAssociations') -Name '.mis' -Value $progId -PropertyType String -Force | Out-Null
New-Item -Path 'HKCU:\Software\RegisteredApplications' -Force | Out-Null
New-ItemProperty -LiteralPath 'HKCU:\Software\RegisteredApplications' -Name 'SkyTEM SkyLab' -Value 'Software\SkyTEM\SkyLab\Capabilities' -PropertyType String -Force | Out-Null
Add-Type -Namespace SkyLab -Name AssociationNotify -MemberDefinition '[System.Runtime.InteropServices.DllImport("shell32.dll")] public static extern void SHChangeNotify(uint eventId, uint flags, System.IntPtr item1, System.IntPtr item2);'
[SkyLab.AssociationNotify]::SHChangeNotify(0x08000000,0,[IntPtr]::Zero,[IntPtr]::Zero)
