# fire_all.ps1 — Fire all Jules tasks from scripts/jules_tasks/ and save session IDs
# Usage: .\scripts\fire_all.ps1

$repo = "Tobiscuit/wizwalker-rs"
$taskDir = "$PSScriptRoot\jules_tasks"
$sessionFile = "$PSScriptRoot\jules_sessions.json"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Jules Batch Dispatcher" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Get all task files sorted
$taskFiles = Get-ChildItem -Path $taskDir -Filter "*.md" | Sort-Object Name

Write-Host "Found $($taskFiles.Count) task files:" -ForegroundColor Yellow
foreach ($f in $taskFiles) {
    Write-Host "  - $($f.Name)" -ForegroundColor Gray
}
Write-Host ""

# Fire each task and capture output
$sessions = @()

foreach ($taskFile in $taskFiles) {
    $taskName = $taskFile.BaseName
    $prompt = Get-Content -Path $taskFile.FullName -Raw
    
    Write-Host "Firing: $taskName ..." -ForegroundColor Green -NoNewline
    
    # Fire Jules and capture the output
    $output = jules new --repo $repo $prompt 2>&1 | Out-String
    
    # Try to extract session ID from output (Jules typically prints it)
    $sessionId = ""
    if ($output -match '(\d{10,})') {
        $sessionId = $Matches[1]
    }
    
    $sessions += @{
        Name = $taskName
        File = $taskFile.Name
        SessionId = $sessionId
        FiredAt = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
        Output = $output.Trim()
    }
    
    if ($sessionId) {
        Write-Host " Session: $sessionId" -ForegroundColor Cyan
    } else {
        Write-Host " (check output below)" -ForegroundColor Yellow
        Write-Host "    $($output.Trim().Substring(0, [Math]::Min(200, $output.Trim().Length)))" -ForegroundColor Gray
    }
    
    # Small delay to avoid rate limiting
    Start-Sleep -Seconds 2
}

# Save session info to JSON
$sessions | ConvertTo-Json -Depth 3 | Out-File -FilePath $sessionFile -Encoding utf8

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  All $($taskFiles.Count) tasks fired!" -ForegroundColor Green
Write-Host "  Sessions saved to: $sessionFile" -ForegroundColor Yellow
Write-Host "  Monitor at: https://jules.google.com/" -ForegroundColor Yellow
Write-Host "  Run check_status.ps1 to check progress" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
