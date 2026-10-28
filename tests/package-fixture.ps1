$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
$zipPath = Join-Path $env:SKYLAB_PACKAGE_TEST_ROOT 'package.zip'
$file = [IO.File]::Create($zipPath)
$zip = New-Object IO.Compression.ZipArchive($file, [IO.Compression.ZipArchiveMode]::Create)
$name = if ($env:SKYLAB_PACKAGE_TEST_EVIL -eq '1') { '../SkyLab-Rust.exe' } else { 'SkyLab-Rust.exe' }
$entry = $zip.CreateEntry($name)
$stream = $entry.Open()
$bytes = [Text.Encoding]::ASCII.GetBytes('MZ-synthetic-update-test')
$stream.Write($bytes,0,$bytes.Length)
$stream.Dispose()
$zip.Dispose()
$file.Dispose()
$sha = [Security.Cryptography.SHA256]::Create()
$inputFile = [IO.File]::OpenRead($zipPath)
try { [BitConverter]::ToString($sha.ComputeHash($inputFile)).Replace('-','').ToLower() }
finally { $inputFile.Dispose(); $sha.Dispose() }
