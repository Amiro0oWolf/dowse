$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$installDir = Join-Path $env:ProgramFiles "Tesseract-OCR"
$sourceExe = Join-Path $installDir "tesseract.exe"
if (!(Test-Path $sourceExe)) {
  choco install tesseract --yes --no-progress
}
if (!(Test-Path $sourceExe)) {
  Write-Error "Tesseract was installed but tesseract.exe was not found at $sourceExe"
  exit 1
}

$targetDir = Join-Path $repoRoot "crates\dowse-app\src-tauri\resources\tesseract"
$dataDir = Join-Path $targetDir "tessdata"
New-Item -ItemType Directory -Force $dataDir | Out-Null
Copy-Item $sourceExe $targetDir
Get-ChildItem $installDir -Filter "*.dll" -File | Copy-Item -Destination $targetDir

$upstreamLicenses = Join-Path $targetDir "licenses\upstream"
New-Item -ItemType Directory -Force $upstreamLicenses | Out-Null
Get-ChildItem $installDir -Recurse -File |
  Where-Object { $_.Name -match "^(LICENSE|COPYING|NOTICE)(\..*)?$" } |
  ForEach-Object {
    $relative = $_.FullName.Substring($installDir.Length).TrimStart("\")
    $destination = Join-Path $upstreamLicenses $relative
    New-Item -ItemType Directory -Force (Split-Path $destination) | Out-Null
    Copy-Item $_.FullName $destination
  }

foreach ($language in @("eng", "fas")) {
  $url = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/87416418657359cb625c412a48b6e1d6d41c29bd/${language}.traineddata"
  Invoke-WebRequest -Uri $url -OutFile (Join-Path $dataDir "${language}.traineddata")
}

$licenseDir = Join-Path $targetDir "licenses"
New-Item -ItemType Directory -Force $licenseDir | Out-Null
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/tesseract-ocr/tesseract/main/LICENSE" -OutFile (Join-Path $licenseDir "Tesseract-Apache-2.0.txt")
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/main/LICENSE" -OutFile (Join-Path $licenseDir "tessdata_fast-Apache-2.0.txt")

& $sourceExe --version
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$languages = & $sourceExe --list-langs --tessdata-dir $dataDir
if ($LASTEXITCODE -ne 0) {
  Write-Error "Tesseract could not load its bundled language models."
  exit $LASTEXITCODE
}
foreach ($language in @("eng", "fas")) {
  if ($languages -notcontains $language) {
    Write-Error "Tesseract did not load the required language model: $language"
    exit 1
  }
}
foreach ($language in @("eng", "fas")) {
  if (!(Test-Path (Join-Path $dataDir "${language}.traineddata"))) {
    Write-Error "Missing required Tesseract model: $language"
    exit 1
  }
}

"TESSERACT_CMD=$targetDir\tesseract.exe" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
