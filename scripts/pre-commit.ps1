param(
    [string]$ReportPath = "target/pre-commit-report.json"
)

$startTime = Get-Date

$steps = @()
$failed = $false
$has_warnings = $false

$GREEN = "`033[0;32m"
$RED = "`033[0;31m"
$YELLOW = "`033[1;33m"
$CYAN = "`033[0;36m"
$MAGENTA = "`033[0;35m"
$CLEAR = "`033[0m"
$BOLD = "`033[1m"

function Write-Step { param([string]$Name) Write-Host "`n$MAGENTA==> [$Name]$CLEAR" }

function Write-Pass { param([string]$Msg) Write-Host "  $GREEN[PASS]$CLEAR $Msg" }

function Write-Fail { param([string]$Msg) Write-Host "  $RED[FAIL]$CLEAR $Msg" }

function Write-Warn { param([string]$Msg) Write-Host "  $YELLOW[WARN]$CLEAR $Msg" }

function Invoke-Step {
    param([string]$Name, [scriptblock]$Block)
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

        if ($warnLines) {
            $script:has_warnings = $true
            $stepStatus = "warning"
            $stepWarnings = $warnLines
        }
        if ($errLines -or $exitCode -ne 0) {
            $script:failed = $true
            $stepStatus = "failure"
            $stepErrors = if ($errLines) { $errLines } else { @("Exit code: $exitCode") }
        }
    }
    catch {
        $script:failed = $true
        $stepStatus = "failure"
        $stepErrors = @($_.Exception.Message)
    }

    $stepDuration = [math]::Round(((Get-Date) - $stepStart).TotalMilliseconds)
    if ($stepStatus -eq "success") { Write-Pass "ok (${stepDuration}ms)" }
    elseif ($stepStatus -eq "warning") {
        Write-Warn "ok con advertencias (${stepDuration}ms)"
        foreach ($w in $stepWarnings) { Write-Host "       $YELLOW$w$CLEAR" }
    }
    else {
        Write-Fail "fallo (${stepDuration}ms)"
        foreach ($e in $stepErrors) { Write-Host "       $RED$e$CLEAR" }
    }

    $script:steps += @{
        name = $Name; status = $stepStatus; duration_ms = $stepDuration
        warnings = $stepWarnings; errors = $stepErrors; has_warnings = $stepWarnings.Count -gt 0
    }
}

Write-Host "$BOLD$MAGENTA"
Write-Host "  PRE-COMMIT CHECK (fmt + check)$CLEAR"
Write-Host "  $CYAN$(Get-Date -Format 'HH:mm:ss')$CLEAR`n"

Write-Step "cargo fmt"
Invoke-Step -Name "fmt" -Block {
    cargo fmt -- --check 2>&1
    if ($LASTEXITCODE -ne 0) {
        cargo fmt 2>&1 | Out-Null
        throw "Formato incorrecto. Se ha ejecutado 'cargo fmt' automaticamente. Revisa y vuelve a intentar."
    }
}

Write-Step "cargo check"
Invoke-Step -Name "check" -Block {
    cargo check --all-targets --workspace 2>&1
}

$totalDuration = [math]::Round(((Get-Date) - $startTime).TotalMilliseconds)
$failedSteps = @($steps | Where-Object { $_.status -eq "failure" }).Count

Write-Host "`n$MAGENTA---$CLEAR"
if ($failedSteps -eq 0) {
    Write-Host "$GREEN[PASS]$CLEAR pre-commit check completado en ${totalDuration}ms"
}
else {
    Write-Host "$RED[FAIL]$CLEAR pre-commit check fallo en ${totalDuration}ms"
    exit 1
}

$json = @{
    workflow = "pre-commit"
    timestamp = (Get-Date -Format "o")
    duration_ms = $totalDuration
    status = if ($failedSteps -eq 0) { "success" } else { "failure" }
    summary = @{ total = $steps.Count; passed = ($steps | Where-Object { $_.status -eq "success" }).Count; failed = $failedSteps; warnings = $has_warnings }
    steps = $steps
} | ConvertTo-Json -Depth 5

$reportDir = Split-Path $ReportPath -Parent
if (-not (Test-Path $reportDir)) { New-Item -ItemType Directory -Path $reportDir -Force | Out-Null }
$json | Set-Content $ReportPath -Encoding UTF8
Write-Host "$CYAN[REPORTE]$CLEAR JSON guardado en: $ReportPath"
