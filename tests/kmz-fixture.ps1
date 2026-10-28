$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.IO.Compression
$memory=New-Object IO.MemoryStream
$zip=New-Object IO.Compression.ZipArchive($memory,[IO.Compression.ZipArchiveMode]::Create,$true)
$names=if($env:SKYLAB_KMZ_AMBIGUOUS -eq '1'){@('a.kml','b.kml')}else{@('doc.kml')}
foreach($name in $names){$entry=$zip.CreateEntry($name);$stream=$entry.Open();$bytes=[Text.Encoding]::UTF8.GetBytes('<kml><Document><Placemark><name>L1</name><LineString><coordinates>-45,-20 -44.999,-20</coordinates></LineString></Placemark></Document></kml>');$stream.Write($bytes,0,$bytes.Length);$stream.Dispose()}
$zip.Dispose()
[Convert]::ToBase64String($memory.ToArray())
$memory.Dispose()
