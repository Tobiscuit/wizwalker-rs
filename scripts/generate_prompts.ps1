# Generates 136 Deep Research prompt .txt files for copy-pasting
# Output: Desktop\math-glossary-prompts\{course}\{unit}.txt

$courses = @{
    "01-pre-algebra" = @("01-whole-numbers-and-operations","02-fractions-and-decimals","03-ratios-and-proportions","04-percents","05-integers-and-rational-numbers","06-expressions-and-equations-intro","07-inequalities","08-geometry-basics","09-data-and-statistics","10-probability")
    "02-algebra-1" = @("01-real-numbers-and-properties","02-linear-equations","03-linear-inequalities","04-functions-and-relations","05-graphing-linear-functions","06-systems-of-equations","07-exponents-and-polynomials","08-factoring","09-quadratic-equations","10-radical-expressions","11-rational-expressions")
    "03-geometry" = @("01-foundations-and-reasoning","02-parallel-and-perpendicular-lines","03-congruent-triangles","04-triangle-relationships","05-quadrilaterals-and-polygons","06-similarity","07-right-triangles-and-trigonometry","08-circles","09-area-and-perimeter","10-surface-area-and-volume","11-transformations","12-coordinate-geometry")
    "04-algebra-2" = @("01-equations-and-inequalities-review","02-linear-functions-and-systems","03-quadratic-functions","04-polynomial-functions","05-radical-functions","06-exponential-and-logarithmic-functions","07-rational-functions","08-sequences-and-series","09-conic-sections","10-probability-and-statistics","11-matrices")
    "05-trigonometry" = @("01-angles-and-radian-measure","02-unit-circle","03-trigonometric-functions","04-graphs-of-trig-functions","05-inverse-trig-functions","06-trig-identities","07-sum-and-difference-formulas","08-double-and-half-angle-formulas","09-law-of-sines-and-cosines","10-polar-coordinates","11-complex-numbers-in-trig-form")
    "06-pre-calculus" = @("01-functions-and-graphs-review","02-polynomial-and-rational-functions","03-exponential-and-logarithmic-functions","04-trigonometric-functions-review","05-analytic-trigonometry","06-vectors","07-parametric-equations","08-polar-coordinates-and-graphs","09-conic-sections-advanced","10-sequences-series-and-induction","11-limits-introduction")
    "07-calculus-1" = @("01-limits-and-continuity","02-definition-of-the-derivative","03-differentiation-rules","04-applications-of-derivatives","05-curve-sketching","06-optimization","07-related-rates","08-antiderivatives-and-indefinite-integrals","09-definite-integrals-and-ftc","10-integration-by-substitution","11-area-between-curves")
    "08-calculus-2" = @("01-integration-techniques","02-integration-by-parts","03-trigonometric-integrals","04-partial-fractions","05-improper-integrals","06-arc-length-and-surface-area","07-volumes-of-revolution","08-sequences","09-infinite-series","10-power-series","11-taylor-and-maclaurin-series","12-parametric-and-polar-calculus")
    "09-calculus-3" = @("01-vectors-and-3d-space","02-vector-valued-functions","03-partial-derivatives","04-directional-derivatives-and-gradients","05-lagrange-multipliers","06-double-integrals","07-triple-integrals","08-change-of-variables","09-line-integrals","10-surface-integrals","11-greens-stokes-divergence-theorems")
    "10-discrete-math" = @("01-logic-and-proofs","02-sets-and-set-operations","03-functions-and-relations","04-mathematical-induction","05-counting-and-combinatorics","06-permutations-and-combinations","07-graph-theory","08-trees-and-spanning-trees","09-boolean-algebra","10-recurrence-relations","11-number-theory","12-formal-languages-and-automata")
    "11-linear-algebra" = @("01-systems-of-linear-equations","02-matrices-and-matrix-operations","03-determinants","04-vector-spaces","05-subspaces","06-linear-independence-and-basis","07-linear-transformations","08-eigenvalues-and-eigenvectors","09-diagonalization","10-inner-product-spaces","11-orthogonality","12-singular-value-decomposition")
    "12-ordinary-differential-equations" = @("01-introduction-and-classification","02-first-order-separable-equations","03-first-order-linear-equations","04-exact-equations","05-second-order-linear-equations","06-homogeneous-equations-with-constant-coefficients","07-nonhomogeneous-equations","08-variation-of-parameters","09-laplace-transforms","10-systems-of-differential-equations","11-series-solutions","12-numerical-methods")
}

function ToTitle($slug) {
    return ((($slug -replace '^\d+-', '') -replace '-', ' ').Split(' ') | ForEach-Object { $_.Substring(0,1).ToUpper() + $_.Substring(1) }) -join ' '
}

$baseDir = "$env:USERPROFILE\Desktop\math-glossary-prompts"
$count = 0

foreach ($course in $courses.GetEnumerator() | Sort-Object Key) {
    $courseTitle = ToTitle $course.Key
    $courseDir = Join-Path $baseDir $course.Key
    New-Item -ItemType Directory -Path $courseDir -Force | Out-Null

    foreach ($unit in $course.Value) {
        $count++
        $unitTitle = ToTitle $unit
        $promptFile = Join-Path $courseDir "$unit.txt"

        $prompt = @"
Create a comprehensive, deeply researched glossary for the following mathematics unit. This is for a software developer learning mathematics.

Course: $courseTitle
Unit: $unitTitle

Format as a Markdown document with these sections:

# $unitTitle ($courseTitle)

## Overview
2-3 sentences on what this unit covers and why it matters for developers.

## Key Concepts
Every important term, symbol, and concept in this unit. For each:
- **Bold term name**
- Clear definition (1-2 sentences)
- Mathematical notation in LaTeX (e.g. `$$notation$$`) if applicable
- A programming analogy or code example where relevant

## Essential Formulas
All must-know formulas with LaTeX notation and brief explanation of when/why to use each.

## Common Pitfalls
Mistakes developers commonly make when learning this material.

## Programming Applications
Specific, concrete ways this math is used in real software (graphics, ML, cryptography, game dev, etc).

## Prerequisites
What the student should already know before studying this unit (link to prior units).

## Further Reading
Recommend 2-3 high-quality free resources (textbooks, videos, interactive sites).

Make it thorough and reference-quality. A developer should be able to Ctrl+F any term they encounter while studying.
"@

        $prompt | Out-File -FilePath $promptFile -Encoding utf8 -NoNewline
    }
    Write-Host "Created $courseTitle ($($course.Value.Count) prompts)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Done! $count prompt files in: $baseDir" -ForegroundColor Cyan
Write-Host "Copy-paste each into Deep Research at gemini.google.com" -ForegroundColor Yellow
