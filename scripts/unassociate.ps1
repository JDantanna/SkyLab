$ErrorActionPreference = 'Stop'
$installRoot = [IO.Path]::GetFullPath($env:SKYLAB_INSTALL_ROOT)
if ($env:SKYLAB_TEST_MODE -eq '1') {
    $record = Join-Path $installRoot 'test-association.json'
    if (Test-Path -LiteralPath $record) {Remove-Item -LiteralPath $record}
    return
}
foreach ($type in @(@{Extension='.mis';ProgId='SkyTEM.SkyLab.MIS'},@{Extension='.skylab';ProgId='SkyTEM.SkyLab.Project'})) {
    $progId=$type.ProgId
    $progKey='HKCU:\Software\Classes\'+$progId
    $owner=Get-ItemProperty -LiteralPath $progKey -ErrorAction SilentlyContinue
    if ($owner -and $owner.SkyLabOwner -eq $installRoot) {
        $extension='HKCU:\Software\Classes\'+$type.Extension
        $extKey=Get-Item -LiteralPath $extension -ErrorAction SilentlyContinue
        if ($extKey -and $extKey.GetValue('') -eq $progId) {Remove-ItemProperty -LiteralPath $extension -Name '(default)' -ErrorAction SilentlyContinue}
        Remove-ItemProperty -LiteralPath ($extension+'\OpenWithProgids') -Name $progId -ErrorAction SilentlyContinue
        foreach ($suffix in @('shell\open\command','shell\open','shell','DefaultIcon','')) { $key=if($suffix){$progKey+'\'+$suffix}else{$progKey};if(Test-Path -LiteralPath $key){Remove-Item -LiteralPath $key} }
    }
}
$capabilities='HKCU:\Software\SkyTEM\SkyLab\Capabilities'
$cap=Get-ItemProperty -LiteralPath $capabilities -ErrorAction SilentlyContinue
if ($cap -and $cap.SkyLabOwner -eq $installRoot) {
    foreach ($key in @(($capabilities+'\FileAssociations'),$capabilities)){if(Test-Path -LiteralPath $key){Remove-Item -LiteralPath $key}}
    $registered='HKCU:\Software\RegisteredApplications'
    $entry=Get-ItemProperty -LiteralPath $registered -ErrorAction SilentlyContinue
    if ($entry.'SkyTEM SkyLab' -eq 'Software\SkyTEM\SkyLab\Capabilities'){Remove-ItemProperty -LiteralPath $registered -Name 'SkyTEM SkyLab'}
}
Add-Type -Namespace SkyLab -Name AssociationNotify -MemberDefinition '[System.Runtime.InteropServices.DllImport("shell32.dll")] public static extern void SHChangeNotify(uint eventId, uint flags, System.IntPtr item1, System.IntPtr item2);'
[SkyLab.AssociationNotify]::SHChangeNotify(0x08000000,0,[IntPtr]::Zero,[IntPtr]::Zero)
