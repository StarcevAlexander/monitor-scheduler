$ErrorActionPreference = "Stop"

$repo = (Get-Location).Path
if ($repo -match '[^\x00-\x7F]' -or $repo.Contains(' ')) {
    $env:CARGO_TARGET_DIR = Join-Path $env:USERPROFILE ".monitors-cargo-target"
    Write-Output "repo path is not linker-safe, build dir: $env:CARGO_TARGET_DIR"
}

if (-not (Get-Command dlltool -ErrorAction SilentlyContinue)) {
    Write-Output "warning: dlltool not found in PATH; install MinGW, e.g. winget install BrechtSanders.WinLibs.POSIX.UCRT"
}

cargo fmt --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo clippy --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$built = if ($env:CARGO_TARGET_DIR) { Join-Path $env:CARGO_TARGET_DIR "release\monitors.exe" } else { "target\release\monitors.exe" }
Copy-Item $built monitors.exe -Force
Write-Output "built monitors.exe"
