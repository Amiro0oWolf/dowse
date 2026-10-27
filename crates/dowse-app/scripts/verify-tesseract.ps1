# Verify the checked-in Windows Tesseract runtime and expose it to CI tests.
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$targetDir = Join-Path $repoRoot "crates\dowse-app\src-tauri\resources\tesseract"
$dataDir = Join-Path $targetDir "tessdata"
$sourceExe = Join-Path $targetDir "tesseract.exe"
if (!(Test-Path $sourceExe)) {
  Write-Error "The repository Tesseract runtime is missing: $sourceExe"
  exit 1
}
foreach ($requiredFile in @(
  "libtesseract-5.dll",
  "libleptonica-6.dll",
  "libstdc++-6.dll",
  "libicudt78.dll",
  "tessdata\eng.traineddata",
  "tessdata\fas.traineddata",
  "tessdata\osd.traineddata"
)) {
  $path = Join-Path $targetDir $requiredFile
  if (!(Test-Path $path)) {
    Write-Error "The repository Tesseract runtime is incomplete; missing $path"
    exit 1
  }
}

$languages = & $sourceExe --list-langs --tessdata-dir $dataDir
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
foreach ($language in @("eng", "fas")) {
  if ($languages -notcontains $language) {
    Write-Error "Repository Tesseract did not load the required language model: $language"
    exit 1
  }
}

$env:TESSERACT_CMD = $sourceExe
if ($env:GITHUB_ENV) {
  "TESSERACT_CMD=$sourceExe" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
}
