#!/usr/bin/env node

/**
 * JavaScript Error Detection Script for Caxton Website
 * Validates JavaScript syntax, detects console errors, and checks for runtime issues
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class JavaScriptErrorChecker {
    constructor() {
        // Determine correct site path - look for website directory from current working directory
        let sitePath = path.join(process.cwd(), 'website');

        // If running from validation directory, go up to project root
        if (process.cwd().includes('scripts/validation')) {
            sitePath = path.join(process.cwd(), '..', '..', 'website');
        }

        this.sitePath = sitePath;
        this.results = {
            syntax: { total: 0, valid: 0, errors: 0, issues: [] },
            runtime: { total: 0, passed: 0, failed: 0, issues: [] },
            console: { warnings: 0, errors: 0, logs: [] },
            performance: { total: 0, passed: 0, issues: [] }
        };
    }

    async run() {
        console.log('🔍 Starting JavaScript Error Detection...');
        console.log(`Site Path: ${this.sitePath}`);
        console.log('─'.repeat(60));

        try {
            await this.checkJavaScriptSyntax();
            await this.analyzeRuntimeBehavior();
            await this.checkConsoleUsage();
            await this.performanceAnalysis();
            this.generateReport();
        } catch (error) {
            console.error('❌ Error during JavaScript validation:', error.message);
            process.exit(1);
        }
    }

    async checkJavaScriptSyntax() {
        console.log('\n📜 Checking JavaScript Syntax...');

        const jsFiles = this.findFiles(['.js']);
        this.results.syntax.total = jsFiles.length;

        for (const file of jsFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Checking syntax: ${relativePath}`);

            try {
                const issues = await this.validateJavaScriptFile(file);

                if (issues.errors.length === 0) {
                    this.results.syntax.valid++;
                    console.log(`  ✅ Valid JavaScript syntax`);
                } else {
                    this.results.syntax.errors++;
                    console.log(`  ❌ ${issues.errors.length} syntax errors`);
                }

                this.results.syntax.issues.push({
                    file: relativePath,
                    errors: issues.errors,
                    warnings: issues.warnings
                });

            } catch (error) {
                this.results.syntax.errors++;
                this.results.syntax.issues.push({
                    file: relativePath,
                    errors: [error.message],
                    warnings: []
                });
                console.log(`  ❌ Syntax check failed: ${error.message}`);
            }
        }
    }

    async validateJavaScriptFile(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const issues = { errors: [], warnings: [] };

        // Basic syntax validation using Node.js
        try {
            // Try to parse the JavaScript (this won't catch runtime errors)
            new Function(content);
        } catch (syntaxError) {
            issues.errors.push(`Syntax error: ${syntaxError.message}`);
        }

        // Static analysis checks
        this.performStaticAnalysis(content, issues, filePath);

        return issues;
    }

    performStaticAnalysis(content, issues, filePath) {
        const lines = content.split('\n');

        lines.forEach((line, index) => {
            const lineNum = index + 1;

            // Check for common issues

            // Undefined variables (basic check)
            if (line.match(/\b[a-zA-Z_$][a-zA-Z0-9_$]*\s*=.*undefined\b/)) {
                issues.warnings.push(`Line ${lineNum}: Assignment to undefined`);
            }

            // Missing semicolons (strict check)
            const trimmed = line.trim();
            if (trimmed &&
                !trimmed.startsWith('//') &&
                !trimmed.startsWith('/*') &&
                !trimmed.endsWith(';') &&
                !trimmed.endsWith('{') &&
                !trimmed.endsWith('}') &&
                !trimmed.endsWith(',') &&
                trimmed.match(/^\s*(var|let|const|return|throw|break|continue)\s/)) {
                issues.warnings.push(`Line ${lineNum}: Missing semicolon`);
            }

            // console.* calls (should be removed in production)
            if (line.match(/console\.(log|warn|error|debug|info)\s*\(/)) {
                issues.warnings.push(`Line ${lineNum}: Console statement found (consider removing for production)`);
            }

            // eval() usage (security risk)
            if (line.includes('eval(')) {
                issues.errors.push(`Line ${lineNum}: eval() usage detected (security risk)`);
            }

            // alert() usage (poor UX)
            if (line.includes('alert(')) {
                issues.warnings.push(`Line ${lineNum}: alert() usage (consider better UX)`);
            }

            // == vs === (type coercion issues)
            if (line.match(/[^=!]==\s*[^=]/) || line.match(/[^=!]!=\s*[^=]/)) {
                issues.warnings.push(`Line ${lineNum}: Use === or !== instead of == or !=`);
            }

            // var instead of let/const
            if (line.match(/^\s*var\s+/)) {
                issues.warnings.push(`Line ${lineNum}: Consider using let or const instead of var`);
            }

            // Missing error handling for async operations
            if (line.includes('await') && !content.includes('try')) {
                issues.warnings.push(`Line ${lineNum}: Async operation without error handling`);
            }

            // Potential XSS vulnerabilities
            if (line.includes('innerHTML') && !line.includes('textContent')) {
                issues.warnings.push(`Line ${lineNum}: innerHTML usage - ensure input is sanitized`);
            }
        });

        // Check for proper function declarations
        this.checkFunctionDeclarations(content, issues);

        // Check for proper event handling
        this.checkEventHandling(content, issues);

        // Check for memory leaks potential
        this.checkMemoryLeaks(content, issues);
    }

    checkFunctionDeclarations(content, issues) {
        // Check for function hoisting issues
        const functionDeclarations = content.match(/function\s+\w+\s*\([^)]*\)\s*{/g) || [];
        const functionExpressions = content.match(/const\s+\w+\s*=\s*\([^)]*\)\s*=>/g) || [];

        if (functionDeclarations.length > 0 && functionExpressions.length > 0) {
            issues.warnings.push('Mixed function declarations and expressions - consider consistency');
        }
    }

    checkEventHandling(content, issues) {
        // Check for proper event listener cleanup
        if (content.includes('addEventListener') && !content.includes('removeEventListener')) {
            issues.warnings.push('Event listeners added but no cleanup found - potential memory leak');
        }

        // Check for passive event listeners
        if (content.match(/addEventListener\s*\(\s*['"`](scroll|wheel|touchstart|touchmove)['"`]/)) {
            if (!content.includes('passive')) {
                issues.warnings.push('Consider using passive event listeners for better performance');
            }
        }
    }

    checkMemoryLeaks(content, issues) {
        // Check for potential memory leaks
        if (content.includes('setInterval') && !content.includes('clearInterval')) {
            issues.warnings.push('setInterval without clearInterval - potential memory leak');
        }

        if (content.includes('setTimeout') && !content.includes('clearTimeout')) {
            issues.warnings.push('setTimeout without cleanup - ensure proper cleanup if needed');
        }

        // Check for DOM references that might not be cleaned up
        if (content.includes('document.getElementById') || content.includes('querySelector')) {
            if (content.includes('addEventListener')) {
                issues.warnings.push('DOM element references with event listeners - ensure proper cleanup');
            }
        }
    }

    async analyzeRuntimeBehavior() {
        console.log('\n🔄 Analyzing Runtime Behavior...');

        const jsFiles = this.findFiles(['.js']);
        this.results.runtime.total = jsFiles.length;

        for (const file of jsFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Analyzing runtime: ${relativePath}`);

            try {
                const issues = await this.analyzeRuntimeFile(file);

                if (issues.length === 0) {
                    this.results.runtime.passed++;
                    console.log(`  ✅ No runtime issues detected`);
                } else {
                    this.results.runtime.failed++;
                    console.log(`  ❌ ${issues.length} potential runtime issues`);

                    this.results.runtime.issues.push({
                        file: relativePath,
                        issues: issues
                    });
                }

            } catch (error) {
                this.results.runtime.failed++;
                console.log(`  ❌ Runtime analysis failed: ${error.message}`);
            }
        }
    }

    async analyzeRuntimeFile(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const issues = [];

        // Check for DOM readiness
        if (content.includes('document.querySelector') || content.includes('document.getElementById')) {
            if (!content.includes('DOMContentLoaded') &&
                !content.includes('document.readyState') &&
                !content.includes('window.addEventListener(\'load\'')) {
                issues.push('DOM manipulation without checking readiness state');
            }
        }

        // Check for error boundaries
        if (content.includes('try') && !content.includes('catch')) {
            issues.push('Try statement without catch block');
        }

        // Check for async/await error handling
        const asyncFunctions = content.match(/async\s+function|\(\s*\)\s*=>\s*{|=\s*async\s*\(/g) || [];
        if (asyncFunctions.length > 0) {
            if (!content.includes('try') && !content.includes('catch')) {
                issues.push('Async functions without error handling');
            }
        }

        // Check for proper promise handling
        if (content.includes('.then(') && !content.includes('.catch(')) {
            issues.push('Promise chain without error handling');
        }

        // Check for potential null/undefined access
        const potentialNullAccess = content.match(/\w+\.\w+/g) || [];
        if (potentialNullAccess.length > 0 && !content.includes('?.')) {
            issues.push('Property access without null checks - consider optional chaining');
        }

        return issues;
    }

    async checkConsoleUsage() {
        console.log('\n📝 Checking Console Usage...');

        const jsFiles = this.findFiles(['.js']);

        for (const file of jsFiles) {
            const content = fs.readFileSync(file, 'utf8');
            const relativePath = path.relative(this.sitePath, file);

            // Find console statements
            const consoleMatches = content.match(/console\.(log|warn|error|debug|info|trace)\s*\([^)]*\)/g) || [];

            consoleMatches.forEach(match => {
                const type = match.match(/console\.(\w+)/)[1];
                const entry = {
                    file: relativePath,
                    type: type,
                    statement: match.substring(0, 100)
                };

                this.results.console.logs.push(entry);

                if (type === 'error') {
                    this.results.console.errors++;
                } else if (type === 'warn') {
                    this.results.console.warnings++;
                }
            });
        }

        console.log(`Found ${this.results.console.logs.length} console statements`);
        console.log(`  ⚠️  ${this.results.console.warnings} warnings`);
        console.log(`  ❌ ${this.results.console.errors} errors`);
    }

    async performanceAnalysis() {
        console.log('\n⚡ Performance Analysis...');

        const jsFiles = this.findFiles(['.js']);
        this.results.performance.total = jsFiles.length;

        for (const file of jsFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Analyzing performance: ${relativePath}`);

            try {
                const issues = await this.analyzePerformance(file);

                if (issues.length === 0) {
                    this.results.performance.passed++;
                    console.log(`  ✅ No performance issues detected`);
                } else {
                    console.log(`  ⚠️  ${issues.length} performance considerations`);

                    this.results.performance.issues.push({
                        file: relativePath,
                        issues: issues
                    });
                }

            } catch (error) {
                console.log(`  ❌ Performance analysis failed: ${error.message}`);
            }
        }
    }

    async analyzePerformance(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const issues = [];

        // Check file size
        const stats = fs.statSync(filePath);
        if (stats.size > 100000) { // 100KB
            issues.push(`Large file size: ${Math.round(stats.size / 1024)}KB - consider code splitting`);
        }

        // Check for expensive operations in loops
        if (content.includes('for') || content.includes('while')) {
            if (content.includes('document.querySelector') || content.includes('getElementById')) {
                issues.push('DOM queries inside loops - consider caching elements');
            }
        }

        // Check for inefficient event handling
        if (content.includes('mousemove') || content.includes('scroll')) {
            if (!content.includes('throttle') && !content.includes('debounce')) {
                issues.push('High-frequency events without throttling/debouncing');
            }
        }

        // Check for memory-intensive operations
        if (content.includes('setInterval')) {
            issues.push('setInterval usage - ensure proper cleanup and consider requestAnimationFrame');
        }

        // Check for blocking operations
        if (content.includes('alert') || content.includes('confirm') || content.includes('prompt')) {
            issues.push('Blocking UI operations detected');
        }

        // Check for proper image loading
        if (content.includes('new Image()')) {
            issues.push('Image preloading detected - ensure proper error handling');
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
        console.log('📋 JAVASCRIPT ERROR DETECTION REPORT');
        console.log('='.repeat(60));

        // Syntax Report
        console.log('\n📜 JavaScript Syntax:');
        console.log(`  Total files: ${this.results.syntax.total}`);
        console.log(`  ✅ Valid: ${this.results.syntax.valid}`);
        console.log(`  ❌ With errors: ${this.results.syntax.errors}`);

        // Runtime Report
        console.log('\n🔄 Runtime Analysis:');
        console.log(`  Total files: ${this.results.runtime.total}`);
        console.log(`  ✅ Passed: ${this.results.runtime.passed}`);
        console.log(`  ❌ Failed: ${this.results.runtime.failed}`);

        // Console Usage Report
        console.log('\n📝 Console Usage:');
        console.log(`  Total statements: ${this.results.console.logs.length}`);
        console.log(`  ⚠️  Warnings: ${this.results.console.warnings}`);
        console.log(`  ❌ Errors: ${this.results.console.errors}`);

        // Performance Report
        console.log('\n⚡ Performance:');
        console.log(`  Total files: ${this.results.performance.total}`);
        console.log(`  ✅ Passed: ${this.results.performance.passed}`);
        console.log(`  ⚠️  With issues: ${this.results.performance.issues.length}`);

        // Show detailed issues
        if (this.results.syntax.errors > 0 ||
            this.results.runtime.failed > 0 ||
            this.results.performance.issues.length > 0) {

            console.log('\n🔍 DETAILED ISSUES:');
            console.log('-'.repeat(60));

            // Syntax Issues
            this.results.syntax.issues.forEach(item => {
                if (item.errors.length > 0 || item.warnings.length > 0) {
                    console.log(`\n📜 ${item.file}:`);
                    item.errors.forEach(error => console.log(`  ❌ ${error}`));
                    item.warnings.forEach(warning => console.log(`  ⚠️  ${warning}`));
                }
            });

            // Runtime Issues
            this.results.runtime.issues.forEach(item => {
                if (item.issues.length > 0) {
                    console.log(`\n🔄 ${item.file}:`);
                    item.issues.forEach(issue => console.log(`  ❌ ${issue}`));
                }
            });

            // Performance Issues
            this.results.performance.issues.forEach(item => {
                if (item.issues.length > 0) {
                    console.log(`\n⚡ ${item.file}:`);
                    item.issues.forEach(issue => console.log(`  ⚠️  ${issue}`));
                }
            });
        }

        // Show console usage details
        if (this.results.console.logs.length > 0) {
            console.log('\n📝 CONSOLE STATEMENTS:');
            console.log('-'.repeat(60));

            const groupedLogs = {};
            this.results.console.logs.forEach(log => {
                if (!groupedLogs[log.file]) {
                    groupedLogs[log.file] = [];
                }
                groupedLogs[log.file].push(log);
            });

            Object.keys(groupedLogs).forEach(file => {
                console.log(`\n${file}:`);
                groupedLogs[file].forEach(log => {
                    const icon = log.type === 'error' ? '❌' :
                               log.type === 'warn' ? '⚠️ ' : 'ℹ️ ';
                    console.log(`  ${icon} ${log.statement}`);
                });
            });
        }

        // Save detailed report
        const reportData = {
            timestamp: new Date().toISOString(),
            results: this.results
        };

        fs.writeFileSync(
            '/workspaces/caxton/scripts/validation/js-error-report.json',
            JSON.stringify(reportData, null, 2)
        );

        console.log('\n📁 Detailed report saved to: js-error-report.json');

        const hasErrors = this.results.syntax.errors > 0 || this.results.runtime.failed > 0;

        if (hasErrors) {
            process.exit(1);
        }
    }
}

// Run if called directly
if (require.main === module) {
    const checker = new JavaScriptErrorChecker();
    checker.run();
}

module.exports = JavaScriptErrorChecker;
