# check_status.ps1 — Check status of all fired Jules sessions
# Usage: .\scripts\check_status.ps1

$sessionFile = "$PSScriptRoot\jules_sessions.json"

if (-not (Test-Path $sessionFile)) {
    Write-Host "No session file found. Run fire_all.ps1 first." -ForegroundColor Red
    exit 1
}

$sessions = Get-Content -Path $sessionFile -Raw | ConvertFrom-Json

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Jules Session Status Check" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Get all remote sessions
Write-Host "Fetching session list..." -ForegroundColor Yellow
$remoteOutput = jules remote list --session 2>&1 | Out-String

Write-Host ""
Write-Host "--- All Sessions from Jules Remote ---" -ForegroundColor Cyan
Write-Host $remoteOutput
Write-Host "--------------------------------------" -ForegroundColor Cyan
Write-Host ""

# Display our tracked sessions
Write-Host "Our fired tasks:" -ForegroundColor Yellow
Write-Host ""

$table = @()
foreach ($session in $sessions) {
    $status = "unknown"
    $needsFeedback = $false
    
    # Try to find this session in remote output
    if ($session.SessionId -and $remoteOutput -match "$($session.SessionId).*?(running|completed|waiting|failed|pending)") {
        $status = $Matches[1]
    }
    if ($remoteOutput -match "$($session.SessionId).*?(waiting|feedback|review)") {
        $needsFeedback = $true
    }
    
    $icon = switch ($status) {
        "completed" { "[OK]" }
        "running"   { "[..]" }
        "waiting"   { "[!!]" }
        "failed"    { "[XX]" }
        "pending"   { "[--]" }
        default     { "[??]" }
    }
    
    $color = switch ($status) {
        "completed" { "Green" }
        "running"   { "Cyan" }
        "waiting"   { "Yellow" }
        "failed"    { "Red" }
        default     { "Gray" }
    }
    
    Write-Host "  $icon " -ForegroundColor $color -NoNewline
    Write-Host "$($session.Name)" -ForegroundColor White -NoNewline
    Write-Host " | Session: $($session.SessionId)" -ForegroundColor Gray -NoNewline
    if ($needsFeedback) {
        Write-Host " | NEEDS FEEDBACK" -ForegroundColor Yellow
    } else {
        Write-Host " | $status" -ForegroundColor $color
    }
}

Write-Host ""
Write-Host "Quick actions:" -ForegroundColor Yellow
Write-Host '  Pull a result:   jules remote pull --session <ID> --apply' -ForegroundColor Gray
Write-Host '  Teleport:        jules teleport <ID>' -ForegroundColor Gray
Write-Host '  View in browser: https://jules.google.com/' -ForegroundColor Gray
Write-Host ""
