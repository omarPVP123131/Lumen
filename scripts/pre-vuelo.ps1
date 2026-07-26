param(
    [switch]$SkipCoverage,
    [switch]$SkipClippy,
    [switch]$SkipTests,
    [string]$ReportPath = "target/pre-vuelo-report.json"
)

$ErrorActionPreference = "Continue"
$startTime = Get-Date

$steps = @()
$failed = $false
$has_warnings = $false

$CLEAR = "`033[0m"
$GREEN = "`033[0;32m"
$RED = "`033[0;31m"
$YELLOW = "`033[1;33m"
$BLUE = "`033[0;34m"
$MAGENTA = "`033[0;35m"
$CYAN = "`033[0;36m"
$BOLD = "`033[1m"

function Write-Step {
    param([string]$Name)
    Write-Host ""
    Write-Host "$MAGENTA==> [$Name]$CLEAR"
    Write-Host "$CYAN    $((Get-Date).ToString('HH:mm:ss'))$CLEAR"
}

function Write-Pass {
    param([string]$Msg)
    Write-Host "  $GREEN[PASS]$CLEAR $Msg"
}

function Write-Fail {
    param([string]$Msg)
    Write-Host "  $RED[FAIL]$CLEAR $Msg"
}

function Write-Skip {
    param([string]$Msg)
    Write-Host "  $YELLOW[SKIP]$CLEAR $Msg"
}

function Write-Warn {
    param([string]$Msg)
    Write-Host "  $YELLOW[WARN]$CLEAR $Msg"
}

function Write-Annotation {
    param([string]$Type, [string]$File, [string]$Line, [string]$Msg)
    Write-Host "  $YELLOW::$Type file=$File,line=$Line::$CLEAR $Msg"
}

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Block,
        [switch]$Optional
    )
    $stepStart = Get-Date
    $stepOutput = @()
    $stepStatus = "success"
    $stepWarnings = @()
    $stepErrors = @()

    try {
        $raw = & $Block
        $exitCode = $LASTEXITCODE
        $output = $raw | ForEach-Object { "$_" }
        $stepOutput = $output

$warnLines = @($output | Where-Object { $_ -match "warning:" -and $_ -notmatch "excluded package" })
$errLines = @($output | Where-Object { $_ -match "^error" })
$testFailLines = @($output | Where-Object { $_ -match "FAILED" })

        if ($warnLines) {
            $script:has_warnings = $true
            $stepStatus = "warning"
            $stepWarnings = $warnLines
        }
        if ($errLines -or $exitCode -ne 0) {
            $script:failed = $true
            $stepStatus = "failure"
            $stepErrors = $errLines
            if (-not $errLines) { $stepErrors = @("Exit code: $exitCode") }
        }
    }
    catch {
        $script:failed = $true
        $stepStatus = "failure"
        $stepErrors = @($_.Exception.Message)
        $stepOutput = @($_.Exception.Message)
    }

    $stepDuration = [math]::Round(((Get-Date) - $stepStart).TotalMilliseconds)

    if ($stepStatus -eq "success") {
        Write-Pass "ok (${stepDuration}ms)"
    }
    elseif ($stepStatus -eq "warning") {
        Write-Warn "ok con advertencias (${stepDuration}ms)"
        foreach ($w in $stepWarnings) { Write-Host "       $YELLOW$w$CLEAR" }
    }
    elseif ($stepStatus -eq "failure" -and $Optional) {
        Write-Skip "no disponible (${stepDuration}ms)"
        $stepStatus = "skipped"
    }
    else {
        Write-Fail "fallo (${stepDuration}ms)"
        foreach ($e in $stepErrors) { Write-Host "       $RED$e$CLEAR" }
    }

    $script:steps += @{
        name = $Name
        status = $stepStatus
        duration_ms = $stepDuration
        warnings = $stepWarnings
        errors = $stepErrors
        has_warnings = $stepWarnings.Count -gt 0
    }
    return $stepStatus
}

# ========== WORKFLOW ==========

Write-Host "$BOLD$MAGENTA"
Write-Host "  _    ____   __  __ _   _ ____  "
Write-Host " | |  |  _ \ /  \|  _ \_ _/ ___| "
Write-Host " | |  | |_) / _ \ |_) | |\___ \ "
Write-Host " | |__|  __/ ___ \  __/| | ___) |"
Write-Host " |____|_| /_/   \_\_|  |___|____/ "
Write-Host ""
Write-Host "  PRE-VUELO CI LOCAL$CLEAR"
Write-Host "  $CYAN$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')$CLEAR"
Write-Host "  $CYAN$((Get-Date -Format 'zzz'))$CLEAR"
Write-Host ""

# Step 1: Format
Write-Step "cargo fmt"
$fmtOk = Invoke-Step -Name "fmt" -Block {
    cargo fmt -- --check 2>&1
    if ($LASTEXITCODE -ne 0) {
        cargo fmt 2>&1 | Out-Null
        throw "Formato incorrecto. Se ha ejecutado 'cargo fmt' automaticamente. Revisa y vuelve a intentar."
    }
}

# Step 2: Compile check
Write-Step "cargo check"
$checkOk = Invoke-Step -Name "check" -Block {
    cargo check --all-targets --workspace 2>&1
}

# Step 3: Clippy
Write-Step "cargo clippy"
if ($SkipClippy) {
    Write-Skip "omitido por flag -SkipClippy"
    $script:steps += @{
        name = "clippy"
        status = "skipped"
        duration_ms = 0
        warnings = @()
        errors = @()
        has_warnings = $false
    }
    $clippyOk = "skipped"
}
else {
    $clippyOk = Invoke-Step -Name "clippy" -Block {
        cargo clippy --all -- -D warnings 2>&1
    }
}

# Step 4: Tests
Write-Step "cargo test"
if ($SkipTests) {
    Write-Skip "omitido por flag -SkipTests"
    $script:steps += @{
        name = "test"
        status = "skipped"
        duration_ms = 0
        warnings = @()
        errors = @()
        has_warnings = $false
    }
    $testOk = "skipped"
}
else {
    $testOk = Invoke-Step -Name "test" -Block {
        cargo test --workspace 2>&1
    }
}

# Step 5: Coverage
Write-Step "cargo llvm-cov"
if ($SkipCoverage) {
    Write-Skip "omitido por flag -SkipCoverage"
    $script:steps += @{
        name = "coverage"
        status = "skipped"
        duration_ms = 0
        warnings = @()
        errors = @()
        has_warnings = $false
        coverage_pct = $null
    }
    $covOk = "skipped"
}
else {
    $covOk = Invoke-Step -Name "coverage" -Optional -Block {
        $null = cargo llvm-cov --version 2>&1
        if ($LASTEXITCODE -ne 0) { throw "cargo-llvm-cov no instalado. Ejecuta: cargo install cargo-llvm-cov" }

        rustup run nightly -- cargo llvm-cov --workspace --html --output-dir target/coverage 2>&1

        if ($LASTEXITCODE -eq 0) {
            Write-Host ""
            Write-Host "       $GREEN[COVERAGE]$CLEAR Reporte HTML: target/coverage/html/index.html"
        }
    }
}

# ========== SUMMARY ==========
Write-Host ""
Write-Host "$BOLD$MAGENTA========== RESUMEN ==========$CLEAR"
$totalDuration = [math]::Round(((Get-Date) - $startTime).TotalMilliseconds)

$passed = @($steps | Where-Object { $_.status -eq "success" }).Count
$warned = @($steps | Where-Object { $_.status -eq "warning" }).Count
$skipped = @($steps | Where-Object { $_.status -eq "skipped" }).Count
$failedSteps = @($steps | Where-Object { $_.status -eq "failure" }).Count

foreach ($s in $steps) {
    $icon = switch ($s.status) {
        "success" { "$GREEN[PASS]$CLEAR" }
        "warning" { "$YELLOW[WARN]$CLEAR" }
        "failure" { "$RED[FAIL]$CLEAR" }
        "skipped" { "$YELLOW[SKIP]$CLEAR" }
    }
    $dur = if ($s.duration_ms -ge 1000) { "$([math]::Round($s.duration_ms/1000, 1))s" } else { "$($s.duration_ms)ms" }
    Write-Host "  $icon $($s.name) ($dur)"
}

Write-Host ""
if ($failedSteps -eq 0 -and -not $has_warnings) {
    Write-Host "  $GREEN$BOLD[TODO OK]$CLEAR $passed pasos completados en ${totalDuration}ms"
    Write-Host "  $GREEN$Bold Listo para el push$CLEAR"
}
else {
    if ($failedSteps -gt 0) {
        Write-Host "  $RED$BOLD[ERRORES]$CLEAR $failedSteps paso(s) fallaron"
    }
    if ($has_warnings) {
        Write-Host "  $YELLOW$BOLD[WARNINGS]$CLEAR $warned paso(s) con advertencias"
    }
}
Write-Host ""

# ========== JSON REPORT ==========
$json = @{
    workflow = "pre-vuelo"
    timestamp = (Get-Date -Format "o")
    duration_ms = $totalDuration
    status = if ($failedSteps -eq 0) { if ($has_warnings) { "warning" } else { "success" } } else { "failure" }
    summary = @{
        total = $steps.Count
        passed = $passed
        warned = $warned
        failed = $failedSteps
        skipped = $skipped
    }
    steps = $steps
} | ConvertTo-Json -Depth 5

$reportDir = Split-Path $ReportPath -Parent
if (-not (Test-Path $reportDir)) { New-Item -ItemType Directory -Path $reportDir -Force | Out-Null }
$json | Set-Content $ReportPath -Encoding UTF8
Write-Host "  $CYAN[REPORTE]$CLEAR JSON guardado en: $ReportPath"

if ($failedSteps -gt 0) { exit 1 } else { exit 0 }
