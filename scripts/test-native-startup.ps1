$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$testDir = Join-Path $root 'target/native-startup-tests'
New-Item -ItemType Directory -Path $testDir -Force | Out-Null
$sdk = Join-Path $root 'native/vendor/webview2'
Copy-Item -LiteralPath (Join-Path $root 'assets/app.ico') -Destination $testDir -Force
foreach ($name in @('Microsoft.Web.WebView2.Core.dll','Microsoft.Web.WebView2.WinForms.dll','WebView2Loader.dll')) {
    Copy-Item -LiteralPath (Join-Path $sdk $name) -Destination $testDir -Force
}
$csc = Join-Path $env:WINDIR 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
$testExe = Join-Path $testDir 'NativeStartup.exe'
& $csc /nologo /target:exe /platform:x64 /codepage:65001 /warnaserror+ /main:NativeStartup "/out:$testExe" /reference:System.Drawing.dll /reference:System.Windows.Forms.dll /reference:System.Web.Extensions.dll /reference:System.Net.Http.dll /reference:System.Core.dll /reference:System.IO.Compression.dll "/reference:$sdk/Microsoft.Web.WebView2.Core.dll" "/reference:$sdk/Microsoft.Web.WebView2.WinForms.dll" (Join-Path $root 'native/Host.cs') (Join-Path $root 'tests/NativeStartup.cs')
if ($LASTEXITCODE -ne 0) { throw 'Native startup test compilation failed.' }
& $testExe
if ($LASTEXITCODE -ne 0) { throw 'Native startup tests failed.' }
