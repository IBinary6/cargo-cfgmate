$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$Repo = "IBinary6/cargo-cfgmate"
$AppName = "Cargo CfgMate"
$ExeName = "cargo-cfgmate.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\CargoCfgMate"
$ShortcutName = "Cargo CfgMate.lnk"
$UserAgent = "cargo-cfgmate-installer"

function Write-Step {
    param([string]$Message)
    Write-Host "[cargo-cfgmate] $Message"
}

function Get-WindowsAssetLabel {
    if ([Environment]::Is64BitOperatingSystem) {
        return "windows-x64"
    }

    return "windows-win32"
}

function Invoke-GitHubJson {
    param([string]$Uri)

    return Invoke-RestMethod -Uri $Uri -Headers @{ "User-Agent" = $UserAgent }
}

function New-AppShortcut {
    param(
        [string]$ShortcutPath,
        [string]$TargetPath,
        [string]$WorkingDirectory
    )

    $parent = Split-Path -Parent $ShortcutPath
    New-Item -ItemType Directory -Path $parent -Force | Out-Null

    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($ShortcutPath)
    $shortcut.TargetPath = $TargetPath
    $shortcut.WorkingDirectory = $WorkingDirectory
    $shortcut.IconLocation = $TargetPath
    $shortcut.Save()
}

function Install-PackageContents {
    param(
        [string]$PackageDir,
        [string]$DestinationDir
    )

    $backupDir = "$DestinationDir.backup"
    $stagingDir = "$DestinationDir.new"

    if (Test-Path -LiteralPath $backupDir) {
        Remove-Item -LiteralPath $backupDir -Recurse -Force
    }

    if (Test-Path -LiteralPath $stagingDir) {
        Remove-Item -LiteralPath $stagingDir -Recurse -Force
    }

    New-Item -ItemType Directory -Path $stagingDir -Force | Out-Null
    Copy-Item -Path (Join-Path $PackageDir "*") -Destination $stagingDir -Recurse -Force

    try {
        if (Test-Path -LiteralPath $DestinationDir) {
            Move-Item -LiteralPath $DestinationDir -Destination $backupDir -Force
        }

        Move-Item -LiteralPath $stagingDir -Destination $DestinationDir -Force

        if (Test-Path -LiteralPath $backupDir) {
            Remove-Item -LiteralPath $backupDir -Recurse -Force
        }
    }
    catch {
        if (Test-Path -LiteralPath $DestinationDir) {
            Remove-Item -LiteralPath $DestinationDir -Recurse -Force
        }

        if (Test-Path -LiteralPath $backupDir) {
            Move-Item -LiteralPath $backupDir -Destination $DestinationDir -Force
        }

        throw
    }
    finally {
        if (Test-Path -LiteralPath $stagingDir) {
            Remove-Item -LiteralPath $stagingDir -Recurse -Force
        }
    }
}

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
}
catch {
}

$tempDir = Join-Path ([IO.Path]::GetTempPath()) ("cargo-cfgmate-install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

try {
    $assetLabel = Get-WindowsAssetLabel
    Write-Step "Detected $assetLabel."

    $release = Invoke-GitHubJson "https://api.github.com/repos/$Repo/releases/latest"
    $asset = $release.assets |
        Where-Object { $_.name -like "*for-$assetLabel.zip" } |
        Select-Object -First 1

    if (-not $asset) {
        throw "No release asset found for $assetLabel in $($release.tag_name)."
    }

    $zipPath = Join-Path $tempDir $asset.name
    $extractDir = Join-Path $tempDir "extract"

    Write-Step "Downloading $($asset.name)."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath -Headers @{ "User-Agent" = $UserAgent }

    if ($asset.digest -and $asset.digest -like "sha256:*") {
        $expectedHash = $asset.digest.Substring("sha256:".Length).ToLowerInvariant()
        $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $zipPath).Hash.ToLowerInvariant()
        if ($actualHash -ne $expectedHash) {
            throw "SHA256 mismatch for $($asset.name)."
        }
    }

    Write-Step "Extracting package."
    Expand-Archive -LiteralPath $zipPath -DestinationPath $extractDir -Force

    $exe = Get-ChildItem -LiteralPath $extractDir -Filter $ExeName -Recurse -File |
        Select-Object -First 1

    if (-not $exe) {
        throw "$ExeName was not found in the downloaded package."
    }

    Write-Step "Installing to $InstallDir."
    Install-PackageContents -PackageDir $exe.DirectoryName -DestinationDir $InstallDir

    $installedExe = Join-Path $InstallDir $ExeName
    if (-not (Test-Path -LiteralPath $installedExe)) {
        throw "Install finished but $ExeName is missing."
    }

    $startMenuShortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\$ShortcutName"
    New-AppShortcut -ShortcutPath $startMenuShortcut -TargetPath $installedExe -WorkingDirectory $InstallDir

    if ($env:CARGO_CFGMATE_NO_DESKTOP -ne "1") {
        $desktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) $ShortcutName
        New-AppShortcut -ShortcutPath $desktopShortcut -TargetPath $installedExe -WorkingDirectory $InstallDir
    }

    Write-Step "Installed $AppName $($release.tag_name)."

    if ($env:CARGO_CFGMATE_NO_LAUNCH -ne "1") {
        Write-Step "Starting $AppName. Windows may ask for administrator permission."
        Start-Process -FilePath $installedExe -WorkingDirectory $InstallDir
    }
}
finally {
    if (Test-Path -LiteralPath $tempDir) {
        Remove-Item -LiteralPath $tempDir -Recurse -Force
    }
}
