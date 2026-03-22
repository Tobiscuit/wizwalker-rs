// Math Glossary Generator — Powered by Gemini Deep Research Pro
// Uses the GenAI Interactions API to create deeply researched glossaries
// Each unit gets its own research task running in parallel

import { GoogleGenAI } from '@google/genai';
import fs from 'fs/promises';
import path from 'path';

const client = new GoogleGenAI({});

const COURSES = {
    "01-pre-algebra": [
        "01-whole-numbers-and-operations",
        "02-fractions-and-decimals",
        "03-ratios-and-proportions",
        "04-percents",
        "05-integers-and-rational-numbers",
        "06-expressions-and-equations-intro",
        "07-inequalities",
        "08-geometry-basics",
        "09-data-and-statistics",
        "10-probability"
    ],
    "02-algebra-1": [
        "01-real-numbers-and-properties",
        "02-linear-equations",
        "03-linear-inequalities",
        "04-functions-and-relations",
        "05-graphing-linear-functions",
        "06-systems-of-equations",
        "07-exponents-and-polynomials",
        "08-factoring",
        "09-quadratic-equations",
        "10-radical-expressions",
        "11-rational-expressions"
    ],
    "03-geometry": [
        "01-foundations-and-reasoning",
        "02-parallel-and-perpendicular-lines",
        "03-congruent-triangles",
        "04-triangle-relationships",
        "05-quadrilaterals-and-polygons",
        "06-similarity",
        "07-right-triangles-and-trigonometry",
        "08-circles",
        "09-area-and-perimeter",
        "10-surface-area-and-volume",
        "11-transformations",
        "12-coordinate-geometry"
    ],
    "04-algebra-2": [
        "01-equations-and-inequalities-review",
        "02-linear-functions-and-systems",
        "03-quadratic-functions",
        "04-polynomial-functions",
        "05-radical-functions",
        "06-exponential-and-logarithmic-functions",
        "07-rational-functions",
        "08-sequences-and-series",
        "09-conic-sections",
        "10-probability-and-statistics",
        "11-matrices"
    ],
    "05-trigonometry": [
        "01-angles-and-radian-measure",
        "02-unit-circle",
        "03-trigonometric-functions",
        "04-graphs-of-trig-functions",
        "05-inverse-trig-functions",
        "06-trig-identities",
        "07-sum-and-difference-formulas",
        "08-double-and-half-angle-formulas",
        "09-law-of-sines-and-cosines",
        "10-polar-coordinates",
        "11-complex-numbers-in-trig-form"
    ],
    "06-pre-calculus": [
        "01-functions-and-graphs-review",
        "02-polynomial-and-rational-functions",
        "03-exponential-and-logarithmic-functions",
        "04-trigonometric-functions-review",
        "05-analytic-trigonometry",
        "06-vectors",
        "07-parametric-equations",
        "08-polar-coordinates-and-graphs",
        "09-conic-sections-advanced",
        "10-sequences-series-and-induction",
        "11-limits-introduction"
    ],
    "07-calculus-1": [
        "01-limits-and-continuity",
        "02-definition-of-the-derivative",
        "03-differentiation-rules",
        "04-applications-of-derivatives",
        "05-curve-sketching",
        "06-optimization",
        "07-related-rates",
        "08-antiderivatives-and-indefinite-integrals",
        "09-definite-integrals-and-ftc",
        "10-integration-by-substitution",
        "11-area-between-curves"
    ],
    "08-calculus-2": [
        "01-integration-techniques",
        "02-integration-by-parts",
        "03-trigonometric-integrals",
        "04-partial-fractions",
        "05-improper-integrals",
        "06-arc-length-and-surface-area",
        "07-volumes-of-revolution",
        "08-sequences",
        "09-infinite-series",
        "10-power-series",
        "11-taylor-and-maclaurin-series",
        "12-parametric-and-polar-calculus"
    ],
    "09-calculus-3": [
        "01-vectors-and-3d-space",
        "02-vector-valued-functions",
        "03-partial-derivatives",
        "04-directional-derivatives-and-gradients",
        "05-lagrange-multipliers",
        "06-double-integrals",
        "07-triple-integrals",
        "08-change-of-variables",
        "09-line-integrals",
        "10-surface-integrals",
        "11-greens-stokes-divergence-theorems"
    ],
    "10-discrete-math": [
        "01-logic-and-proofs",
        "02-sets-and-set-operations",
        "03-functions-and-relations",
        "04-mathematical-induction",
        "05-counting-and-combinatorics",
        "06-permutations-and-combinations",
        "07-graph-theory",
        "08-trees-and-spanning-trees",
        "09-boolean-algebra",
        "10-recurrence-relations",
        "11-number-theory",
        "12-formal-languages-and-automata"
    ],
    "11-linear-algebra": [
        "01-systems-of-linear-equations",
        "02-matrices-and-matrix-operations",
        "03-determinants",
        "04-vector-spaces",
        "05-subspaces",
        "06-linear-independence-and-basis",
        "07-linear-transformations",
        "08-eigenvalues-and-eigenvectors",
        "09-diagonalization",
        "10-inner-product-spaces",
        "11-orthogonality",
        "12-singular-value-decomposition"
    ],
    "12-ordinary-differential-equations": [
        "01-introduction-and-classification",
        "02-first-order-separable-equations",
        "03-first-order-linear-equations",
        "04-exact-equations",
        "05-second-order-linear-equations",
        "06-homogeneous-equations-with-constant-coefficients",
        "07-nonhomogeneous-equations",
        "08-variation-of-parameters",
        "09-laplace-transforms",
        "10-systems-of-differential-equations",
        "11-series-solutions",
        "12-numerical-methods"
    ]
};

// --- Config ---
const BASE_DIR = path.join(process.env.USERPROFILE || process.env.HOME, 'Desktop', 'math-glossary');
const MAX_CONCURRENT = 5; // Max parallel research tasks (be nice to the API)
const POLL_INTERVAL_MS = 15000; // 15 seconds between polls

function toTitle(slug) {
    return slug.replace(/^\d+-/, '').replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

async function generateGlossary(courseName, courseTitle, unitName, unitTitle, unitDir) {
    const glossaryFile = path.join(unitDir, 'glossary.md');

    // Skip if exists
    try {
        await fs.access(glossaryFile);
        console.log(`  SKIP: ${courseTitle} > ${unitTitle} (exists)`);
        return { status: 'skipped', course: courseTitle, unit: unitTitle };
    } catch { /* doesn't exist, proceed */ }

    const prompt = `Research and create a comprehensive glossary for the following mathematics unit. 
This is for a software developer learning mathematics from scratch.

Course: ${courseTitle}
Unit: ${unitTitle}

Create a Markdown document with:
1. # ${unitTitle} (${courseTitle})
2. A brief overview (2-3 sentences) of what this unit covers and why it matters for developers
3. ## Key Concepts — every important term defined with:
   - **Bold term name**
   - Clear definition (1-2 sentences)
   - Mathematical notation in LaTeX ($notation$) if applicable
   - A programming analogy or code example where relevant
4. ## Essential Formulas — the must-know formulas with LaTeX
5. ## Common Pitfalls — mistakes developers commonly make with this material
6. ## Programming Applications — specific ways this math is used in software development
7. ## Sources — cite the sources you used for accuracy

Make it thorough and reference-quality. A developer should be able to ctrl+F any term they encounter while studying.`;

    try {
        console.log(`  START: ${courseTitle} > ${unitTitle}`);

        const interaction = await client.interactions.create({
            input: prompt,
            agent: 'deep-research-pro-preview-12-2025',
            background: true
        });

        console.log(`    Research ID: ${interaction.id}`);

        // Poll until done
        while (true) {
            const result = await client.interactions.get(interaction.id);

            if (result.status === 'completed') {
                const text = result.outputs[result.outputs.length - 1].text;
                await fs.writeFile(glossaryFile, text, 'utf8');
                const lines = text.split('\n').length;
                console.log(`  DONE: ${courseTitle} > ${unitTitle} (${lines} lines)`);
                return { status: 'completed', course: courseTitle, unit: unitTitle, lines };
            } else if (result.status === 'failed') {
                console.error(`  FAIL: ${courseTitle} > ${unitTitle}: ${result.error}`);
                return { status: 'failed', course: courseTitle, unit: unitTitle, error: result.error };
            }

            await new Promise(resolve => setTimeout(resolve, POLL_INTERVAL_MS));
        }
    } catch (err) {
        console.error(`  ERROR: ${courseTitle} > ${unitTitle}: ${err.message}`);
        return { status: 'error', course: courseTitle, unit: unitTitle, error: err.message };
    }
}

async function main() {
    console.log('');
    console.log('========================================');
    console.log('  Math Glossary Generator');
    console.log('  Powered by Gemini Deep Research Pro');
    console.log('========================================');
    console.log('');

    // Create all directories
    let totalUnits = 0;
    const allTasks = [];

    for (const [courseName, units] of Object.entries(COURSES)) {
        const courseTitle = toTitle(courseName);
        const courseDir = path.join(BASE_DIR, courseName);
        await fs.mkdir(courseDir, { recursive: true });

        for (const unitName of units) {
            totalUnits++;
            const unitTitle = toTitle(unitName);
            const unitDir = path.join(courseDir, unitName);
            await fs.mkdir(unitDir, { recursive: true });
            allTasks.push({ courseName, courseTitle, unitName, unitTitle, unitDir });
        }

        console.log(`  Created: ${courseName}/ (${units.length} units)`);
    }

    console.log('');
    console.log(`Total: ${totalUnits} units across ${Object.keys(COURSES).length} courses`);
    console.log(`Max concurrent: ${MAX_CONCURRENT}`);
    console.log(`Output: ${BASE_DIR}`);
    console.log('');

    // Process in batches of MAX_CONCURRENT
    const results = [];
    for (let i = 0; i < allTasks.length; i += MAX_CONCURRENT) {
        const batch = allTasks.slice(i, i + MAX_CONCURRENT);
        console.log(`--- Batch ${Math.floor(i / MAX_CONCURRENT) + 1}/${Math.ceil(allTasks.length / MAX_CONCURRENT)} ---`);

        const batchResults = await Promise.all(
            batch.map(t => generateGlossary(t.courseName, t.courseTitle, t.unitName, t.unitTitle, t.unitDir))
        );
        results.push(...batchResults);
    }

    // Summary
    const completed = results.filter(r => r.status === 'completed').length;
    const skipped = results.filter(r => r.status === 'skipped').length;
    const failed = results.filter(r => r.status === 'failed' || r.status === 'error').length;

    console.log('');
    console.log('========================================');
    console.log(`  Complete! ${completed} generated, ${skipped} skipped, ${failed} failed`);
    console.log(`  Output: ${BASE_DIR}`);
    console.log('========================================');
}

main().catch(console.error);
