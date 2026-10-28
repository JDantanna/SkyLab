$ErrorActionPreference = 'Stop'
function Invoke-Native([string]$file, [string[]]$nativeArgs) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $file
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    foreach ($argument in $nativeArgs) { $info.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    try {
        [void]$process.Start()
        $output = $process.StandardOutput.ReadToEndAsync()
        $errors = $process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit(240000)) { $process.Kill(); $process.WaitForExit(); throw 'Native transport timed out' }
        if ($process.ExitCode -ne 0) { throw $errors.GetAwaiter().GetResult() }
        return $output.GetAwaiter().GetResult()
    } finally { $process.Dispose() }
}
$release = gh api "repos/JDantanna/SkyLab/releases/tags/$env:SKYLAB_RELEASE_TAG" | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Cannot read published release' }
$package = @($release.assets | Where-Object { $_.name -like 'SkyLab-Update-*.zip' })
if ($package.Count -ne 1 -or $package[0].digest -notmatch '^sha256:[0-9a-fA-F]{64}$') { throw 'Published update package missing SHA256' }
$hostBinary = Get-ChildItem target/release/build -Filter SkyLab.Window.exe -Recurse | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (!$hostBinary) { throw 'Built native transport missing' }
$root = Join-Path ([IO.Path]::GetTempPath()) ('skylab-https-test-' + [Guid]::NewGuid().ToString('N'))
$hostRoot = Join-Path $root 'host'
$stage = Join-Path $root 'stage'
[IO.Directory]::CreateDirectory($hostRoot) | Out-Null
[IO.Directory]::CreateDirectory($stage) | Out-Null
$hostFiles = @('SkyLab.Window.exe','Microsoft.Web.WebView2.Core.dll','Microsoft.Web.WebView2.WinForms.dll','WebView2Loader.dll')
try {
    Copy-Item -LiteralPath $hostBinary.FullName -Destination (Join-Path $hostRoot 'SkyLab.Window.exe')
    foreach ($name in $hostFiles | Select-Object -Skip 1) { Copy-Item -LiteralPath (Join-Path native/vendor/webview2 $name) -Destination (Join-Path $hostRoot $name) }
    $progress = Invoke-Native (Join-Path $hostRoot 'SkyLab.Window.exe') @('--download-update', $package[0].browser_download_url, $package[0].digest.Substring(7), $stage)
    Write-Output ($progress.Trim().Split("`n") | Select-Object -Last 1)
    $identity = Invoke-Native (Join-Path $stage 'SkyLab-Rust.exe') @('--update-identity') | ConvertFrom-Json
    if ($identity.product -ne 'skytem.skylab.local' -or $identity.revision -ne [uint32](Get-Content -Raw BUILD_REVISION).Trim()) { throw 'Published binary identity mismatch' }
    Write-Output 'PASS: native HTTPS download from GitHub, redirects, SHA256, package extraction and published identity.'
} finally {
    foreach ($name in $hostFiles) { $file=Join-Path $hostRoot $name; if (Test-Path -LiteralPath $file) { Remove-Item -LiteralPath $file } }
    foreach ($name in @('SkyLab-Rust.exe','package.zip','package.zip.part')) { $file=Join-Path $stage $name; if (Test-Path -LiteralPath $file) { Remove-Item -LiteralPath $file } }
    [IO.Directory]::Delete($hostRoot)
    [IO.Directory]::Delete($stage)
    [IO.Directory]::Delete($root)
}
