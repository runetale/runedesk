# Sign files with DigiCert KeyLocker (smctl) and verify the Authenticode
# signature. smctl can exit 0 on failure, so the signature is checked
# independently.
param([Parameter(Mandatory = $true, ValueFromRemainingArguments = $true)] [string[]] $Files)
$ErrorActionPreference = 'Stop'

if (-not $env:SM_CODE_SIGNING_CERT_SHA1_HASH) {
  Write-Error "SM_CODE_SIGNING_CERT_SHA1_HASH is not set"
}

# smctl signs through signtool and the KeyLocker certificate in the
# Windows store; set both up first (as runetale-desktop does).
$sdkBase = "C:\Program Files (x86)\Windows Kits\10\bin"
$signtool = Get-ChildItem $sdkBase -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
  Where-Object { $_.FullName -match "\\x64\\" } |
  Sort-Object { $_.Directory.Name } -Descending |
  Select-Object -First 1
if (-not $signtool) { Write-Error "signtool.exe not found under $sdkBase" }
$env:Path = "$($signtool.Directory.FullName);$env:Path"
& smctl windows certsync 2>&1 | ForEach-Object { Write-Host "  $_" }

foreach ($file in $Files) {
  $name = [System.IO.Path]::GetFileName($file)
  Write-Host "Signing $name"
  $out = & smctl sign --fingerprint $env:SM_CODE_SIGNING_CERT_SHA1_HASH --input $file 2>&1
  $code = $LASTEXITCODE
  $out | ForEach-Object { Write-Host "  $_" }
  if ($code -ne 0 -or ($out -match 'FAILED')) {
    Write-Error "$name signing failed (smctl exit code $code)"
  }
  $sig = Get-AuthenticodeSignature -FilePath $file
  if ($sig.Status -ne 'Valid') {
    Write-Error "$name signature is not valid: $($sig.Status) $($sig.StatusMessage)"
  }
  Write-Host "  $($sig.SignerCertificate.Subject)"
}
