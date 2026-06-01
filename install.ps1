$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
} catch {
}

$Repo = if ($env:SETTUPPER_REPO) { $env:SETTUPPER_REPO } else { "devbaraus/settupper" }
$Version = if ($env:SETTUPPER_VERSION) { $env:SETTUPPER_VERSION } else { $null }
$InstallDir = if ($env:SETTUPPER_INSTALL_DIR) { $env:SETTUPPER_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs\settupper\bin" }
$BinaryName = "settupper.exe"

function Fail($Message) {
    Write-Error $Message
    exit 1
}

function Get-SettupperAssetName {
    if (-not $env:OS -or $env:OS -ne "Windows_NT") {
        Fail "Operating system not supported by this installer."
    }

    $arch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    switch ($arch) {
        "AMD64" { return "settupper-windows-x86_64.zip" }
        "ARM64" { return "settupper-windows-aarch64.zip" }
        default { Fail "Unsupported Windows architecture: $arch" }
    }
}

function Get-LatestTag {
    $uri = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $release = Invoke-RestMethod -Uri $uri -Headers @{ "User-Agent" = "settupper-installer" }
        if (-not $release.tag_name) {
            Fail "Could not determine the latest release tag at $uri"
        }
        return $release.tag_name
    } catch {
        Fail "Failed to query the latest release at $uri. $($_.Exception.Message)"
    }
}

$Archive = Get-SettupperAssetName
if (-not $Version) {
    $Version = Get-LatestTag
}

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
$ArchivePath = Join-Path $TempDir $Archive
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$Archive"

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    Write-Host "Downloading settupper $Version ($Archive)..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchivePath -Headers @{ "User-Agent" = "settupper-installer" }

    Expand-Archive -Path $ArchivePath -DestinationPath $TempDir -Force

    $SourceBinary = Join-Path $TempDir $BinaryName
    if (-not (Test-Path $SourceBinary)) {
        Fail "Binary $BinaryName not found in archive $Archive"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Path $SourceBinary -Destination (Join-Path $InstallDir $BinaryName) -Force

    Write-Host ""
    Write-Host "Settupper installed at: $(Join-Path $InstallDir $BinaryName)"

    $PathEntries = ($env:PATH -split ";") | ForEach-Object { $_.TrimEnd("\") }
    if ($PathEntries -contains $InstallDir.TrimEnd("\")) {
        Write-Host "Run with: settupper"
    } else {
        Write-Host "Add it to PATH to run with just 'settupper':"
        Write-Host "  setx PATH `"$env:PATH;$InstallDir`""
    }
} finally {
    if (Test-Path $TempDir) {
        Remove-Item -Recurse -Force $TempDir
    }
}
