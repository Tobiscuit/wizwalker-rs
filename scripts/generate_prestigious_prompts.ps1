# Math Glossary Prompt Generator — Prestigious University Edition
# Step 1: Gemini 3.1 Pro generates the proper unit breakdown per course
# Step 2: Gemini 3.1 Pro generates a tailored prompt for each unit
# Step 3: YOU paste each prompt into Deep Research web GUI
#
# Usage: .\scripts\generate_prestigious_prompts.ps1

$model = "gemini-3.1-pro-preview"
$baseDir = "$env:USERPROFILE\Desktop\math-glossary-prompts"

$courses = @(
    "Pre-Algebra"
    "Algebra 1"
    "Geometry"
    "Algebra 2"
    "Trigonometry"
    "Pre-Calculus"
    "Calculus 1"
    "Calculus 2"
    "Calculus 3 (Multivariable Calculus)"
    "Discrete Mathematics"
    "Linear Algebra"
    "Ordinary Differential Equations"
)

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Prestigious Math Glossary Generator" -ForegroundColor Cyan
Write-Host "  Step 1: Generate unit structure" -ForegroundColor Cyan
Write-Host "  Step 2: Generate tailored prompts" -ForegroundColor Cyan
Write-Host "  Model: $model" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$courseNum = 0
$totalPrompts = 0

foreach ($course in $courses) {
    $courseNum++
    $slug = ($course -replace '[^a-zA-Z0-9 ]', '' -replace ' +', '-').ToLower()
    $courseDir = Join-Path $baseDir ("{0:D2}-{1}" -f $courseNum, $slug)
    New-Item -ItemType Directory -Path $courseDir -Force | Out-Null

    $unitsFile = Join-Path $courseDir "_units.json"

    # --- STEP 1: Generate unit breakdown ---
    if (-not (Test-Path $unitsFile)) {
        Write-Host "[$courseNum/12] Generating units for: $course ..." -ForegroundColor Yellow -NoNewline

        $unitPrompt = @"
You are a curriculum designer at MIT. List the units/chapters taught in a rigorous $course course at a top university (MIT, Stanford, Caltech level).

For each unit, provide:
- A short slug (e.g. "limits-and-continuity")
- The full title (e.g. "Limits and Continuity")
- 3-5 key topics covered in that unit

Output ONLY valid JSON array, no markdown fences, no explanation. Example format:
[{"slug":"example-unit","title":"Example Unit","topics":["Topic A","Topic B","Topic C"]}]
"@

        try {
            $raw = gemini -m $model $unitPrompt 2>$null | Out-String

            # Strip markdown fences if present
            $json = $raw -replace '(?s)```json\s*', '' -replace '(?s)```\s*', ''
            $json = $json.Trim()

            # Validate it's JSON
            $parsed = $json | ConvertFrom-Json
            $json | Out-File -FilePath $unitsFile -Encoding utf8 -NoNewline
            Write-Host " $($parsed.Count) units" -ForegroundColor Green
        }
        catch {
            Write-Host " FAILED (saving raw output)" -ForegroundColor Red
            $raw | Out-File -FilePath "$unitsFile.error.txt" -Encoding utf8
            continue
        }
    }
    else {
        $parsed = Get-Content $unitsFile -Raw | ConvertFrom-Json
        Write-Host "[$courseNum/12] Loaded cached units for: $course ($($parsed.Count) units)" -ForegroundColor DarkGray
    }

    # --- STEP 2: Generate tailored prompts per unit ---
    $units = Get-Content $unitsFile -Raw | ConvertFrom-Json
    $unitNum = 0

    foreach ($unit in $units) {
        $unitNum++
        $promptFile = Join-Path $courseDir ("{0:D2}-{1}.txt" -f $unitNum, $unit.slug)

        if (Test-Path $promptFile) {
            Write-Host "  [$unitNum/$($units.Count)] SKIP: $($unit.title) (exists)" -ForegroundColor DarkGray
            $totalPrompts++
            continue
        }

        Write-Host "  [$unitNum/$($units.Count)] Crafting prompt: $($unit.title) ..." -ForegroundColor Cyan -NoNewline

        $topicsList = ($unit.topics | ForEach-Object { "- $_" }) -join "`n"

        $metaPrompt = @"
You are creating a study prompt for Deep Research. The goal is to produce a comprehensive, university-level glossary document.

Course: $course (at MIT/Stanford/Caltech rigor level)
Unit: $($unit.title)
Key topics in this unit:
$topicsList

Write a detailed prompt that I will paste into Google's Deep Research tool. The prompt should ask Deep Research to:

1. Research and create a comprehensive glossary for this specific unit at a prestigious university level
2. Cover EVERY theorem, definition, lemma, corollary, property, and key concept taught in this unit
3. Include formal mathematical notation (LaTeX)
4. Include rigorous definitions alongside intuitive explanations
5. Provide proof sketches for key theorems where appropriate
6. Include worked examples for each major concept
7. Note common exam questions and problem types for this unit
8. Explain applications in computer science, engineering, physics, and data science
9. List prerequisite knowledge needed
10. Recommend specific textbook sections (from Stewart, Rudin, Strang, Rosen, etc.)

The prompt should be specific to THIS unit's content — reference the exact theorems, methods, and concepts by name. Don't be generic.

Output ONLY the prompt text, nothing else. No preamble, no "Here's the prompt:" — just the raw prompt I'll paste.
"@

        try {
            $result = gemini -m $model $metaPrompt 2>$null | Out-String
            $result.Trim() | Out-File -FilePath $promptFile -Encoding utf8 -NoNewline
            $lines = ($result.Trim() -split "`n").Count
            Write-Host " done ($lines lines)" -ForegroundColor Green
            $totalPrompts++
        }
        catch {
            Write-Host " FAILED: $($_.Exception.Message)" -ForegroundColor Red
        }
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Complete! $totalPrompts prompts generated" -ForegroundColor Green
Write-Host "  Output: $baseDir" -ForegroundColor Yellow
Write-Host "  Paste each .txt into Deep Research" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
