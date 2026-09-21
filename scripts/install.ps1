<#Requires -Version 5.1
<#
.SYNOPSIS
  Installs Boring Commit (bcommit) system-wide on Windows.

.DESCRIPTION
  irm https://raw.githubusercontent.com/iamprasadraju/Boring-Commit/main/scripts/install.ps1 | iex

  Downloads the prebuilt Windows x64 binary from GitHub releases, verifies
  its SHA256 checksum, installs to Program Files, adds it to the machine
  PATH, and smoke-tests it. Requires elevation (re-launches itself as admin).

.PARAMETER Version
  Release tag, e.g. v0.1.0. Defaults to the latest release.
  The BCOMMIT_VERSION environment variable is honored as well.

.PARAMETER InstallDir
  Install directory. Defaults to "$env:ProgramFiles\bcommit".
#>
param(
    [string]$Version = $env:BCOMMIT_VERSION,
    [string]$InstallDir = (Join-Path $env:ProgramFiles "bcommit")
)

function Install-BCommit {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $ErrorActionPreference = "Stop"

    $Repo = "iamprasadraju/Boring-Commit"
    $Asset = "bcommit-windows-x86_64.zip"

    function Fail([string]$msg) { Write-Error $msg; exit 1 }

    # --- 1. Platform checks ---------------------------------------------------
    if (-not [Environment]::Is64BitOperatingSystem) {
        Fail "Only 64-bit Windows has prebuilt binaries. Fallback: cargo install bcommit"
    }
    $osBuild = [int](Get-CimInstance Win32_OperatingSystem).BuildNumber
    if ($osBuild -lt 10240) {
        Fail "Requires Windows 10 or later (build 10240+)."
    }

    # --- 2. Elevate (system-wide install needs admin) --------------------------
    $isAdmin = ([Security.Principal.WindowsPrincipal] `
        [Security.Principal.WindowsIdentity]::GetCurrent() `
        ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) {
        Write-Host ">>> Restarting elevated (system-wide install needs admin)..."
        # Re-run via a temp file so arguments survive the elevation boundary.
        $tmp = Join-Path $env:TEMP "bcommit-install.ps1"
        Invoke-WebRequest -UseBasicParsing `
            -Uri "https://raw.githubusercontent.com/iamprasadraju/Boring-Commit/main/scripts/install.ps1" `
            -OutFile $tmp
        $psiArgs = "-NoProfile -ExecutionPolicy Bypass -File `"$tmp`""
        if ($Version) { $psiArgs += " -Version `"$Version`"" }
        $psiArgs += " -InstallDir `"$InstallDir`""
        $proc = Start-Process powershell -ArgumentList $psiArgs -Verb RunAs -Wait -PassThru
        exit $proc.ExitCode
    }

    # --- 3. Resolve version -----------------------------------------------------
    if ([string]::IsNullOrWhiteSpace($Version)) {
        Write-Host ">>> Resolving latest release..."
        try {
            $rel = Invoke-RestMethod -UseBasicParsing `
                -Uri "https://api.github.com/repos/$Repo/releases/latest"
            $Version = $rel.tag_name
        } catch {
            Fail "Could not reach api.github.com to resolve the latest release. Check your network or pass -Version explicitly."
        }
    }
    if (-not $Version.StartsWith("v")) { $Version = "v$Version" }
    Write-Host ">>> Installing bcommit $Version for Windows/x64..."

    # --- 4. Download + verify checksum ------------------------------------------
    $base = "https://github.com/$Repo/releases/download/$Version"
    $work = Join-Path $env:TEMP "bcommit-install"
    if (Test-Path $work) { Remove-Item -Recurse -Force $work }
    New-Item -ItemType Directory -Force -Path $work | Out-Null
    try {
        $zip = Join-Path $work $Asset
        Write-Host ">>> Downloading $Asset..."
        Invoke-WebRequest -UseBasicParsing -Uri "$base/$Asset" -OutFile $zip

        Write-Host ">>> Verifying checksum..."
        $sidecar = "$zip.sha256"
        Invoke-WebRequest -UseBasicParsing -Uri "$base/$Asset.sha256" -OutFile $sidecar
        $expected = ((Get-Content $sidecar -TotalCount 1) -split '\s+')[0].ToLower()
        $actual = (Get-FileHash -Algorithm SHA256 -Path $zip).Hash.ToLower()
        if ($expected -ne $actual) {
            Fail "Checksum mismatch for $Asset. Aborting before install."
        }

        # --- 5. Install + machine PATH -------------------------------------------
        Write-Host ">>> Installing to $InstallDir..."
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        Expand-Archive -Path $zip -DestinationPath $InstallDir -Force

        $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if (($machinePath -split ";" | ForEach-Object { $_.TrimEnd("\") }) -notcontains $InstallDir) {
            Write-Host ">>> Adding to machine PATH..."
            [Environment]::SetEnvironmentVariable("Path", "$machinePath;$InstallDir", "Machine")
            # Notify running programs (e.g. Explorer) that the environment changed.
            # Guarded: re-running the script in the same session must not fail.
            if (-not ([System.Management.Automation.PSTypeName]"BCommit.Win32").Type) {
                Add-Type -Namespace BCommit -Name Win32 -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("user32.dll", SetLastError = true, CharSet = System.Runtime.InteropServices.CharSet.Auto)]
public static extern System.IntPtr SendMessageTimeout(System.IntPtr hWnd, uint Msg, System.UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out System.UIntPtr lpdwResult);
"@
            }
            [UIntPtr]$r = 0
            [BCommit.Win32]::SendMessageTimeout([IntPtr]0xffff, 0x1a, [UIntPtr]::Zero, "Environment", 2, 5000, [ref]$r) | Out-Null
        }

        # --- 6. Smoke test ---------------------------------------------------------
        & (Join-Path $InstallDir "bcommit.exe") --version
        Write-Host ">>> Install complete. Next step: bcommit config"
        Write-Host "    (open a NEW terminal so it picks up the updated PATH)"
    } finally {
        if (Test-Path $work) { Remove-Item -Recurse -Force $work }
    }
}

Install-BCommit
