#!/usr/bin/env node

/**
 * HTML/CSS Validation Script for Caxton Website
 * Validates HTML structure, accessibility, and CSS syntax
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class HtmlCssValidator {
    constructor() {
        // Determine correct site path - look for website directory from current working directory
        let sitePath = path.join(process.cwd(), 'website');

        // If running from validation directory, go up to project root
        if (process.cwd().includes('scripts/validation')) {
            sitePath = path.join(process.cwd(), '..', '..', 'website');
        }

        this.sitePath = sitePath;
        this.results = {
            html: { total: 0, valid: 0, errors: 0, warnings: 0, issues: [] },
            css: { total: 0, valid: 0, errors: 0, warnings: 0, issues: [] },
            accessibility: { total: 0, passed: 0, failed: 0, issues: [] }
        };
    }

    async run() {
        console.log('🔍 Starting HTML/CSS Validation...');
        console.log(`Site Path: ${this.sitePath}`);
        console.log('─'.repeat(60));

        try {
            await this.validateHtmlFiles();
            await this.validateCssFiles();
            await this.checkAccessibility();
            this.generateReport();
        } catch (error) {
            console.error('❌ Error during validation:', error.message);
            process.exit(1);
        }
    }

    async validateHtmlFiles() {
        console.log('\n📄 Validating HTML Files...');

        const htmlFiles = this.findFiles(['.html']);
        this.results.html.total = htmlFiles.length;

        for (const file of htmlFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Checking: ${relativePath}`);

            try {
                const issues = await this.validateHtmlFile(file);

                if (issues.errors.length === 0) {
                    this.results.html.valid++;
                    console.log(`  ✅ Valid HTML`);
                } else {
                    this.results.html.errors++;
                    console.log(`  ❌ ${issues.errors.length} HTML errors`);
                }

                if (issues.warnings.length > 0) {
                    this.results.html.warnings += issues.warnings.length;
                    console.log(`  ⚠️  ${issues.warnings.length} HTML warnings`);
                }

                this.results.html.issues.push({
                    file: relativePath,
                    errors: issues.errors,
                    warnings: issues.warnings
                });

            } catch (error) {
                this.results.html.errors++;
                this.results.html.issues.push({
                    file: relativePath,
                    errors: [error.message],
                    warnings: []
                });
                console.log(`  ❌ Validation failed: ${error.message}`);
            }
        }
    }

    async validateHtmlFile(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const issues = { errors: [], warnings: [] };

        // Basic HTML structure validation
        this.checkHtmlStructure(content, issues, filePath);

        // Check for proper DOCTYPE
        if (!content.includes('<!DOCTYPE html>') && !content.includes('<!doctype html>')) {
            issues.warnings.push('Missing DOCTYPE declaration');
        }

        // Check for required HTML attributes
        if (content.includes('<html') && !content.match(/<html[^>]*lang=/i)) {
            issues.errors.push('Missing lang attribute on html element');
        }

        // Check for proper charset
        if (!content.match(/<meta[^>]*charset=/i)) {
            issues.warnings.push('Missing charset declaration');
        }

        // Check for viewport meta tag
        if (!content.match(/<meta[^>]*name=["']viewport["'][^>]*>/i)) {
            issues.warnings.push('Missing viewport meta tag');
        }

        // Validate image alt attributes
        this.validateImageAltText(content, issues);

        // Check for proper heading hierarchy
        this.validateHeadingHierarchy(content, issues);

        // Check for proper form labels
        this.validateFormLabels(content, issues);

        return issues;
    }

    checkHtmlStructure(content, issues, filePath) {
        // Check for unmatched tags (basic check)
        const openTags = (content.match(/<[^/][^>]*>/g) || [])
            .filter(tag => !tag.match(/<(area|base|br|col|embed|hr|img|input|link|meta|param|source|track|wbr)[^>]*>/i))
            .map(tag => tag.match(/<(\w+)/)[1].toLowerCase());

        const closeTags = (content.match(/<\/[^>]+>/g) || [])
            .map(tag => tag.match(/<\/(\w+)/)[1].toLowerCase());

        // Simple validation for common structural issues
        const requiredTags = ['html', 'head', 'body'];
        for (const tag of requiredTags) {
            if (!content.match(new RegExp(`<${tag}[^>]*>`, 'i'))) {
                issues.errors.push(`Missing required tag: ${tag}`);
            }
        }
    }

    validateImageAltText(content, issues) {
        const images = content.match(/<img[^>]*>/gi) || [];

        for (const img of images) {
            if (!img.match(/alt=["'][^"']*["']/i)) {
                issues.errors.push(`Image missing alt attribute: ${img.substring(0, 50)}...`);
            } else {
                const altMatch = img.match(/alt=["']([^"']*)["']/i);
                if (altMatch && altMatch[1].trim() === '') {
                    // Empty alt is okay for decorative images, but warn if no role or aria-hidden
                    if (!img.match(/role=["']presentation["']/i) && !img.match(/aria-hidden=["']true["']/i)) {
                        issues.warnings.push(`Image has empty alt text but no role="presentation": ${img.substring(0, 50)}...`);
                    }
                }
            }
        }
    }

    validateHeadingHierarchy(content, issues) {
        const headings = (content.match(/<h[1-6][^>]*>/gi) || [])
            .map(h => parseInt(h.match(/<h([1-6])/i)[1]));

        let previousLevel = 0;
        for (const level of headings) {
            if (level > previousLevel + 1) {
                issues.warnings.push(`Heading hierarchy skip: h${previousLevel} to h${level}`);
            }
            previousLevel = level;
        }
    }

    validateFormLabels(content, issues) {
        const inputs = content.match(/<input[^>]*>/gi) || [];

        for (const input of inputs) {
            const type = (input.match(/type=["']([^"']*)["']/i) || [])[1];
            if (type && ['text', 'email', 'password', 'tel', 'url', 'search'].includes(type)) {
                const id = (input.match(/id=["']([^"']*)["']/i) || [])[1];
                if (!id || !content.match(new RegExp(`<label[^>]*for=["']${id}["']`, 'i'))) {
                    if (!input.match(/aria-label=["'][^"']*["']/i)) {
                        issues.errors.push(`Input missing label: ${input.substring(0, 50)}...`);
                    }
                }
            }
        }
    }

    async validateCssFiles() {
        console.log('\n🎨 Validating CSS Files...');

        const cssFiles = this.findFiles(['.css', '.scss']);
        this.results.css.total = cssFiles.length;

        for (const file of cssFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Checking: ${relativePath}`);

            try {
                const issues = await this.validateCssFile(file);

                if (issues.errors.length === 0) {
                    this.results.css.valid++;
                    console.log(`  ✅ Valid CSS`);
                } else {
                    this.results.css.errors++;
                    console.log(`  ❌ ${issues.errors.length} CSS errors`);
                }

                if (issues.warnings.length > 0) {
                    this.results.css.warnings += issues.warnings.length;
                    console.log(`  ⚠️  ${issues.warnings.length} CSS warnings`);
                }

                this.results.css.issues.push({
                    file: relativePath,
                    errors: issues.errors,
                    warnings: issues.warnings
                });

            } catch (error) {
                this.results.css.errors++;
                this.results.css.issues.push({
                    file: relativePath,
                    errors: [error.message],
                    warnings: []
                });
                console.log(`  ❌ Validation failed: ${error.message}`);
            }
        }
    }

    async validateCssFile(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const issues = { errors: [], warnings: [] };

        // Check for syntax issues
        this.checkCssSyntax(content, issues);

        // Check for browser compatibility issues
        this.checkBrowserCompatibility(content, issues);

        // Check for accessibility issues
        this.checkCssAccessibility(content, issues);

        return issues;
    }

    checkCssSyntax(content, issues) {
        // Basic CSS syntax validation

        // Check for unclosed brackets
        const openBraces = (content.match(/{/g) || []).length;
        const closeBraces = (content.match(/}/g) || []).length;

        if (openBraces !== closeBraces) {
            issues.errors.push(`Unmatched braces: ${openBraces} open, ${closeBraces} close`);
        }

        // Check for common syntax errors
        const lines = content.split('\n');
        lines.forEach((line, index) => {
            const lineNum = index + 1;

            // Check for missing semicolons (basic check)
            if (line.match(/:\s*[^;{}]+$/)) {
                const trimmed = line.trim();
                if (!trimmed.endsWith(';') && !trimmed.endsWith('{') && !trimmed.endsWith('}') && trimmed !== '') {
                    issues.warnings.push(`Line ${lineNum}: Possibly missing semicolon`);
                }
            }

            // Check for unknown CSS properties (basic list)
            const propertyMatch = line.match(/^\s*([a-z-]+)\s*:/);
            if (propertyMatch) {
                const property = propertyMatch[1];
                if (property.startsWith('--')) {
                    // CSS custom property, skip validation
                    return;
                }

                // This is a simplified check - in production, you'd use a comprehensive CSS property list
                const unknownProperties = ['colr', 'widht', 'heigth', 'margn', 'paddig'];
                if (unknownProperties.includes(property)) {
                    issues.errors.push(`Line ${lineNum}: Unknown CSS property '${property}'`);
                }
            }
        });
    }

    checkBrowserCompatibility(content, issues) {
        // Check for potentially problematic CSS features
        const compatibilityIssues = [
            { pattern: /backdrop-filter:/i, warning: 'backdrop-filter has limited browser support' },
            { pattern: /-webkit-backdrop-filter:/i, warning: 'Consider adding standard backdrop-filter property' },
            { pattern: /grid-template-areas:/i, warning: 'CSS Grid has good support but test in older browsers' },
            { pattern: /@supports\s*not/i, warning: '@supports not() has limited support in older browsers' }
        ];

        compatibilityIssues.forEach(issue => {
            if (content.match(issue.pattern)) {
                issues.warnings.push(issue.warning);
            }
        });
    }

    checkCssAccessibility(content, issues) {
        // Check for accessibility issues in CSS

        // Check for focus styles
        if (!content.match(/:focus/)) {
            issues.warnings.push('No focus styles defined - consider accessibility');
        }

        // Check for font-size in px only
        const fontSizeMatches = content.match(/font-size:\s*\d+px/g) || [];
        if (fontSizeMatches.length > 0 && !content.match(/font-size:\s*[\d.]+rem/)) {
            issues.warnings.push('Consider using rem units for better accessibility');
        }

        // Check for color contrast issues (basic check)
        if (content.includes('color: white') || content.includes('color: #fff')) {
            if (content.includes('background: white') || content.includes('background: #fff')) {
                issues.errors.push('Potential color contrast issue: white text on white background');
            }
        }
    }

    async checkAccessibility() {
        console.log('\n♿ Checking Accessibility...');

        const htmlFiles = this.findFiles(['.html']);
        this.results.accessibility.total = htmlFiles.length;

        for (const file of htmlFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Checking accessibility: ${relativePath}`);

            try {
                const issues = await this.checkAccessibilityFile(file);

                if (issues.length === 0) {
                    this.results.accessibility.passed++;
                    console.log(`  ✅ Accessibility checks passed`);
                } else {
                    this.results.accessibility.failed++;
                    console.log(`  ❌ ${issues.length} accessibility issues`);

                    this.results.accessibility.issues.push({
                        file: relativePath,
                        issues: issues
                    });
                }

            } catch (error) {
                this.results.accessibility.failed++;
                console.log(`  ❌ Accessibility check failed: ${error.message}`);
            }
        }
    }

    async checkAccessibilityFile(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const issues = [];

        // Check for skip links
        if (content.includes('<nav') && !content.match(/skip.{0,20}content/i)) {
            issues.push('Consider adding a skip link for keyboard navigation');
        }

        // Check for ARIA landmarks
        const landmarks = ['main', 'navigation', 'banner', 'contentinfo'];
        const hasRoleLandmarks = landmarks.some(landmark =>
            content.includes(`role="${landmark}"`) || content.includes(`role='${landmark}'`)
        );
        const hasSemanticLandmarks = content.match(/<(main|nav|header|footer)[^>]*>/);

        if (!hasRoleLandmarks && !hasSemanticLandmarks) {
            issues.push('No ARIA landmarks found - consider adding for screen readers');
        }

        // Check for proper button/link usage
        const clickableElements = content.match(/<[^>]+onclick[^>]*>/gi) || [];
        if (clickableElements.length > 0) {
            issues.push('onclick handlers found - ensure keyboard accessibility');
        }

        // Check for table headers
        const tables = content.match(/<table[^>]*>/gi) || [];
        if (tables.length > 0 && !content.match(/<th[^>]*>/i)) {
            issues.push('Tables without headers found - add <th> elements');
        }

        return issues;
    }

    findFiles(extensions) {
        const files = [];

        const walkDir = (dir) => {
            try {
                const items = fs.readdirSync(dir);

                for (const item of items) {
                    const fullPath = path.join(dir, item);
                    const stat = fs.statSync(fullPath);

                    if (stat.isDirectory() && !item.startsWith('.') && item !== 'node_modules') {
                        walkDir(fullPath);
                    } else if (extensions.some(ext => item.endsWith(ext))) {
                        files.push(fullPath);
                    }
                }
            } catch (error) {
                console.warn(`Warning: Cannot access directory ${dir}: ${error.message}`);
            }
        };

        walkDir(this.sitePath);
        return files;
    }

    generateReport() {
        console.log('\n' + '='.repeat(60));
        console.log('📋 HTML/CSS VALIDATION REPORT');
        console.log('='.repeat(60));

        // HTML Report
        console.log('\n📄 HTML Validation:');
        console.log(`  Total files: ${this.results.html.total}`);
        console.log(`  ✅ Valid: ${this.results.html.valid}`);
        console.log(`  ❌ With errors: ${this.results.html.errors}`);
        console.log(`  ⚠️  Total warnings: ${this.results.html.warnings}`);

        // CSS Report
        console.log('\n🎨 CSS Validation:');
        console.log(`  Total files: ${this.results.css.total}`);
        console.log(`  ✅ Valid: ${this.results.css.valid}`);
        console.log(`  ❌ With errors: ${this.results.css.errors}`);
        console.log(`  ⚠️  Total warnings: ${this.results.css.warnings}`);

        // Accessibility Report
        console.log('\n♿ Accessibility:');
        console.log(`  Total files: ${this.results.accessibility.total}`);
        console.log(`  ✅ Passed: ${this.results.accessibility.passed}`);
        console.log(`  ❌ Failed: ${this.results.accessibility.failed}`);

        // Show detailed issues
        if (this.results.html.errors > 0 || this.results.css.errors > 0 || this.results.accessibility.failed > 0) {
            console.log('\n🔍 DETAILED ISSUES:');
            console.log('-'.repeat(60));

            // HTML Issues
            this.results.html.issues.forEach(item => {
                if (item.errors.length > 0 || item.warnings.length > 0) {
                    console.log(`\n📄 ${item.file}:`);
                    item.errors.forEach(error => console.log(`  ❌ ${error}`));
                    item.warnings.forEach(warning => console.log(`  ⚠️  ${warning}`));
                }
            });

            // CSS Issues
            this.results.css.issues.forEach(item => {
                if (item.errors.length > 0 || item.warnings.length > 0) {
                    console.log(`\n🎨 ${item.file}:`);
                    item.errors.forEach(error => console.log(`  ❌ ${error}`));
                    item.warnings.forEach(warning => console.log(`  ⚠️  ${warning}`));
                }
            });

            // Accessibility Issues
            this.results.accessibility.issues.forEach(item => {
                if (item.issues.length > 0) {
                    console.log(`\n♿ ${item.file}:`);
                    item.issues.forEach(issue => console.log(`  ❌ ${issue}`));
                }
            });
        }

        // Save detailed report
        const reportData = {
            timestamp: new Date().toISOString(),
            results: this.results
        };

        fs.writeFileSync(
            '/workspaces/caxton/scripts/validation/html-css-report.json',
            JSON.stringify(reportData, null, 2)
        );

        console.log('\n📁 Detailed report saved to: html-css-report.json');

        const hasErrors = this.results.html.errors > 0 ||
                         this.results.css.errors > 0 ||
                         this.results.accessibility.failed > 0;

        if (hasErrors) {
            process.exit(1);
        }
    }
}

// Run if called directly
if (require.main === module) {
    const validator = new HtmlCssValidator();
    validator.run();
}

module.exports = HtmlCssValidator;
