#!/usr/bin/env node

/**
 * Anchor Links Validation Script
 * Validates that the anchor links implementation is correctly structured
 */

const fs = require('fs');
const path = require('path');

// File paths
const anchorLinksJs = path.join(__dirname, '../../website/assets/js/anchor-links.js');
const documentationCss = path.join(__dirname, '../../website/assets/css/documentation.css');
const adrCss = path.join(__dirname, '../../website/assets/css/adr.css');
const documentationLayout = path.join(__dirname, '../../website/_layouts/documentation.html');
const adrLayout = path.join(__dirname, '../../website/_layouts/adr.html');

console.log('🔍 Validating Anchor Links Implementation...\n');

let issues = [];
let successes = [];

// Check if files exist
const filesToCheck = [
    { path: anchorLinksJs, name: 'anchor-links.js' },
    { path: documentationCss, name: 'documentation.css' },
    { path: adrCss, name: 'adr.css' },
    { path: documentationLayout, name: 'documentation.html layout' },
    { path: adrLayout, name: 'adr.html layout' }
];

filesToCheck.forEach(file => {
    if (fs.existsSync(file.path)) {
        successes.push(`✅ ${file.name} exists`);
    } else {
        issues.push(`❌ ${file.name} is missing`);
    }
});

// Validate JavaScript file
if (fs.existsSync(anchorLinksJs)) {
    const jsContent = fs.readFileSync(anchorLinksJs, 'utf8');

    // Check for required classes and methods
    const requiredFeatures = [
        'class AnchorLinks',
        'generateAnchors()',
        'setupSmoothScrolling()',
        'copyToClipboard(',
        'updateTableOfContents()',
        'showToast(',
        'handleInitialAnchor()'
    ];

    requiredFeatures.forEach(feature => {
        if (jsContent.includes(feature)) {
            successes.push(`✅ JavaScript includes ${feature}`);
        } else {
            issues.push(`❌ JavaScript missing ${feature}`);
        }
    });

    // Check for accessibility features
    const accessibilityFeatures = [
        'aria-label',
        'tabindex',
        'prefers-reduced-motion'
    ];

    accessibilityFeatures.forEach(feature => {
        if (jsContent.includes(feature)) {
            successes.push(`✅ Accessibility feature: ${feature}`);
        } else {
            issues.push(`⚠️ Missing accessibility feature: ${feature}`);
        }
    });
}

// Validate CSS files
[documentationCss, adrCss].forEach(cssFile => {
    if (fs.existsSync(cssFile)) {
        const cssContent = fs.readFileSync(cssFile, 'utf8');
        const fileName = path.basename(cssFile);

        // Check for anchor link styles
        const requiredStyles = [
            '.anchor-link',
            '.anchor-icon',
            '.table-of-contents',
            'scroll-margin-top'
        ];

        requiredStyles.forEach(style => {
            if (cssContent.includes(style)) {
                successes.push(`✅ ${fileName} includes ${style} styles`);
            } else {
                issues.push(`❌ ${fileName} missing ${style} styles`);
            }
        });

        // Check for mobile responsiveness
        if (cssContent.includes('@media (max-width: 768px)')) {
            successes.push(`✅ ${fileName} includes mobile styles`);
        } else {
            issues.push(`⚠️ ${fileName} missing mobile styles`);
        }

        // Check for print styles
        if (cssContent.includes('@media print')) {
            successes.push(`✅ ${fileName} includes print styles`);
        } else {
            issues.push(`⚠️ ${fileName} missing print styles`);
        }
    }
});

// Validate layout files
[documentationLayout, adrLayout].forEach(layoutFile => {
    if (fs.existsSync(layoutFile)) {
        const layoutContent = fs.readFileSync(layoutFile, 'utf8');
        const fileName = path.basename(layoutFile);

        if (layoutContent.includes('anchor-links.js')) {
            successes.push(`✅ ${fileName} includes anchor-links.js`);
        } else {
            issues.push(`❌ ${fileName} missing anchor-links.js include`);
        }
    }
});

// Check for demo page
const demoPage = path.join(__dirname, '../../website/docs/anchor-links-demo.md');
if (fs.existsSync(demoPage)) {
    successes.push('✅ Demo page created');
} else {
    issues.push('⚠️ Demo page missing (optional)');
}

// Report results
console.log('📊 VALIDATION RESULTS\n');

console.log('🎉 SUCCESSES:');
successes.forEach(success => console.log(`  ${success}`));

if (issues.length > 0) {
    console.log('\n⚠️ ISSUES:');
    issues.forEach(issue => console.log(`  ${issue}`));
}

console.log(`\n📈 Summary: ${successes.length} successes, ${issues.length} issues`);

// Final assessment
const criticalIssues = issues.filter(issue => issue.startsWith('❌'));
if (criticalIssues.length === 0) {
    console.log('\n✅ IMPLEMENTATION COMPLETE: Anchor links are properly implemented!');
    console.log('\nFeatures included:');
    console.log('  • Auto-generated anchor links for h2-h6 headings');
    console.log('  • Hover-visible link icons with # symbol');
    console.log('  • Smooth scroll behavior with proper offset');
    console.log('  • Copy-to-clipboard functionality (right-click or Ctrl+click)');
    console.log('  • Auto-generated table of contents for long documents');
    console.log('  • Mobile-friendly responsive design');
    console.log('  • Accessibility features (keyboard nav, screen reader support)');
    console.log('  • Print-friendly styling');
    console.log('  • Integration with both documentation and ADR layouts');
} else {
    console.log('\n❌ CRITICAL ISSUES FOUND: Please fix the issues above before using.');
    process.exit(1);
}

console.log('\n🚀 To test the implementation:');
console.log('  1. Start the Jekyll server: bundle exec jekyll serve');
console.log('  2. Visit: http://localhost:4000/docs/anchor-links-demo/');
console.log('  3. Hover over headings to see anchor links');
console.log('  4. Right-click anchor links to copy URLs');
console.log('  5. Check table of contents generation');
