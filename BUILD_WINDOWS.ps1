[CmdletBinding()]
param(
    [ValidateSet('standalone', 'bundle')]
    [string]$Mode = 'standalone'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Step([string]$Message) { Write-Host "==> $Message" -ForegroundColor Cyan }
function Write-Ok([string]$Message)   { Write-Host "    $Message" -ForegroundColor Green }
function Write-Warn([string]$Message) { Write-Host "    $Message" -ForegroundColor Yellow }
function Write-Fail([string]$Message) { Write-Host "ERROR: $Message" -ForegroundColor Red }

$Root = $PSScriptRoot
$Dist = Join-Path $Root 'dist'
$BinName = 'locomotive.exe'
$Target = 'x86_64-pc-windows-msvc'
$ExitCode = 1

try {
    Write-Step 'Checking optional Steam name and cover lookup'
    $curl = Get-Command 'curl.exe' -ErrorAction SilentlyContinue
    if ($curl) {
        Write-Ok 'curl.exe found; missing game names and cover art can be resolved online'
    } else {
        Write-Warn 'curl.exe not found; Locomotive will use local Steam metadata and cached artwork'
        Write-Warn 'Optional install command: winget install --id cURL.cURL -e'
    }

    Write-Step 'Checking for the Rust toolchain'
    $cargo = Get-Command 'cargo.exe' -ErrorAction SilentlyContinue
    $rustup = Get-Command 'rustup.exe' -ErrorAction SilentlyContinue
    if (-not $cargo -or -not $rustup) {
        throw 'Rust was not found. Install the stable Rust toolchain from https://rustup.rs, then close and reopen your terminal.'
    }
    Write-Ok "cargo: $($cargo.Source)"
    Write-Ok "rustup: $($rustup.Source)"

    Write-Step "Verifying target $Target"
    $targetList = & $rustup.Source target list --installed
    if ($LASTEXITCODE -ne 0) {
        throw "rustup could not list installed targets (exit code $LASTEXITCODE). Run 'rustup toolchain install stable' and try again."
    }
    if ($targetList -notcontains $Target) {
        Write-Warn "Installing missing Rust target $Target"
        & $rustup.Source target add $Target
        if ($LASTEXITCODE -ne 0) {
            throw "Could not install Rust target $Target (exit code $LASTEXITCODE)."
        }
    }
    Write-Ok "$Target available"

    Write-Step 'Checking for the Visual C++ build tools and Windows SDK'
    $programFilesX86 = ${env:ProgramFiles(x86)}
    if ([string]::IsNullOrWhiteSpace($programFilesX86)) {
        $programFilesX86 = $env:ProgramFiles
    }
    $vswhere = Join-Path $programFilesX86 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) {
        throw 'Visual Studio Build Tools were not found. Install Visual Studio Build Tools with the "Desktop development with C++" workload and a Windows 10/11 SDK.'
    }

    $vsInstallPath = & $vswhere -latest -products '*' -requires 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64' -property installationPath 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace(($vsInstallPath | Select-Object -First 1))) {
        throw 'The MSVC C++ compiler tools were not found. Open Visual Studio Installer, modify your installation, and add the "Desktop development with C++" workload.'
    }
    Write-Ok "MSVC installation: $(($vsInstallPath | Select-Object -First 1).Trim())"

    Write-Step 'Building release executable'
    Push-Location $Root
    try {
        $lockFile = Join-Path $Root 'Cargo.lock'
        if (Test-Path -LiteralPath $lockFile -PathType Leaf) {
            Write-Ok 'Cargo.lock found; Cargo will reuse its pinned versions and update the lockfile if the manifest requires it'
        } else {
            Write-Warn 'Cargo.lock is missing; Cargo will resolve dependencies and create it during this build.'
            Write-Warn 'Keep the generated Cargo.lock with the source project to make later builds reproducible.'
        }

        # Do not pass --locked here. It makes Cargo fail before compilation if
        # the checked-in lockfile does not match Cargo.toml (for example after
        # a dependency feature change). Normal `cargo build` still respects an
        # existing valid lockfile and updates it only when resolution requires it.
        $cargoArguments = @('build', '--release', '--workspace', '--target', $Target)
        & $cargo.Source @cargoArguments
        $cargoExitCode = $LASTEXITCODE
        if ($cargoExitCode -ne 0) {
            throw "cargo build failed (exit code $cargoExitCode). The Cargo error above is the relevant failure; fix that issue and rerun this script."
        }
    } finally {
        Pop-Location
    }

    $targetDir = Join-Path $Root "target\$Target\release"
    $source = Join-Path $targetDir $BinName
    if (-not (Test-Path -LiteralPath $source)) {
        throw "Build finished without producing the expected executable: $source"
    }

    Write-Step 'Checking executable architecture'
    function Get-PeArchitecture([string]$Path) {
        $bytes = [System.IO.File]::ReadAllBytes($Path)
        if ($bytes.Length -lt 0x40) { return 'unknown' }
        $peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
        if ($peOffset -lt 0 -or ($peOffset + 6) -gt $bytes.Length) { return 'unknown' }
        if ([BitConverter]::ToUInt32($bytes, $peOffset) -ne 0x00004550) { return 'unknown' }
        $machine = [BitConverter]::ToUInt16($bytes, $peOffset + 4)
        switch ($machine) {
            0x8664 { return 'x64' }
            0x014C { return 'x86' }
            default { return 'other' }
        }
    }

    $architecture = Get-PeArchitecture -Path $source
    if ($architecture -ne 'x64') {
        throw "$BinName has architecture '$architecture'; expected x64."
    }
    Write-Ok "$BinName : x64"

    Write-Step 'Staging release files'
    if (-not (Test-Path -LiteralPath $Dist)) {
        New-Item -ItemType Directory -Path $Dist -Force | Out-Null
    }

    # Remove obsolete artifacts from older builds that packaged a Steam hook.
    foreach ($stale in @('dwmapi.dll', 'winhttp.dll', 'locomotive_hook.dll')) {
        $path = Join-Path $Dist $stale
        if (Test-Path -LiteralPath $path) {
            Remove-Item -LiteralPath $path -Force
            Write-Warn "Removed stale dist\$stale"
        }
    }
    $staleHookDir = Join-Path $Dist 'steamhook'
    if (Test-Path -LiteralPath $staleHookDir) {
        Remove-Item -LiteralPath $staleHookDir -Recurse -Force
        Write-Warn 'Removed stale dist\steamhook directory'
    }

    if ($Mode -eq 'standalone') {
        Copy-Item -LiteralPath $source -Destination (Join-Path $Dist $BinName) -Force
        Write-Ok "Wrote $BinName to $Dist"
    } else {
        $bundle = Join-Path $Dist 'locomotive'
        if (Test-Path -LiteralPath $bundle) {
            Remove-Item -LiteralPath $bundle -Recurse -Force
        }
        New-Item -ItemType Directory -Path $bundle -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $bundle 'assets') -Force | Out-Null
        Copy-Item -LiteralPath $source -Destination (Join-Path $bundle $BinName) -Force

        foreach ($document in @('README.md', 'CHANGELOG.txt', 'LICENSE')) {
            $documentPath = Join-Path $Root $document
            if (Test-Path -LiteralPath $documentPath) {
                Copy-Item -LiteralPath $documentPath -Destination $bundle -Force
            }
        }
        foreach ($asset in @('icon.png', 'icon.ico')) {
            $assetPath = Join-Path $Root "assets\$asset"
            if (Test-Path -LiteralPath $assetPath) {
                Copy-Item -LiteralPath $assetPath -Destination (Join-Path $bundle 'assets') -Force
            }
        }
        Write-Ok "Wrote portable bundle to $bundle"
    }

    Write-Host ''
    Write-Host 'Build complete.' -ForegroundColor Green
    $ExitCode = 0
} catch {
    Write-Host ''
    Write-Fail $_.Exception.Message
    Write-Host 'See BUILD_INSTRUCTIONS.md for prerequisites and troubleshooting.' -ForegroundColor Yellow
} finally {
    if ($ExitCode -ne 0) {
        Write-Host ''
        Write-Host 'Build failed.' -ForegroundColor Red
    }
}

exit $ExitCode
