#!/usr/bin/env node

/**
 * Master Validation Script for Caxton Website
 * Runs all validation checks and generates a comprehensive report
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Import all validation modules
const DeadLinkChecker = require('./dead-link-checker');
const HtmlCssValidator = require('./html-css-validator');
const JavaScriptErrorChecker = require('./js-error-checker');
const CodeSyntaxHighlighter = require('./code-syntax-highlighter');
const SeoMetaValidator = require('./seo-meta-validator');
const BuildDeploymentChecker = require('./build-deployment-checker');
const ResponsiveDesignChecker = require('./responsive-design-checker');

class MasterValidationRunner {
    constructor() {
        this.scriptsPath = path.join(process.cwd(), 'scripts', 'validation');
        this.results = {
            summary: {
                totalChecks: 7,
                passed: 0,
                failed: 0,
                warnings: 0,
                startTime: null,
                endTime: null,
                duration: null
            },
            checks: {}
        };

        this.validators = [
            {
                name: 'Dead Link Detection',
                key: 'deadLinks',
                class: DeadLinkChecker,
                critical: true,
                description: 'Validates internal and external links'
            },
            {
                name: 'HTML/CSS Validation',
                key: 'htmlCss',
                class: HtmlCssValidator,
                critical: true,
                description: 'Validates HTML structure and CSS syntax'
            },
            {
                name: 'JavaScript Error Detection',
                key: 'javascript',
                class: JavaScriptErrorChecker,
                critical: true,
                description: 'Detects JavaScript syntax and runtime errors'
            },
            {
                name: 'Code Syntax Highlighting',
                key: 'codeSyntax',
                class: CodeSyntaxHighlighter,
                critical: false,
                description: 'Validates code block syntax highlighting'
            },
            {
                name: 'SEO Meta Tags',
                key: 'seoMeta',
                class: SeoMetaValidator,
                critical: false,
                description: 'Validates SEO elements and meta tags'
            },
            {
                name: 'Build/Deployment',
                key: 'buildDeploy',
                class: BuildDeploymentChecker,
                critical: true,
                description: 'Checks build and deployment configuration'
            },
            {
                name: 'Responsive Design',
                key: 'responsive',
                class: ResponsiveDesignChecker,
                critical: false,
                description: 'Validates responsive design breakpoints'
            }
        ];
    }

    async run() {
        console.log('🚀 CAXTON WEBSITE VALIDATION SUITE');
        console.log('='.repeat(60));
        console.log('Running comprehensive technical validation...');
        console.log(`Start Time: ${new Date().toLocaleString()}`);
        console.log('='.repeat(60));

        this.results.summary.startTime = new Date().toISOString();

        try {
            // Run all validators
            for (const validator of this.validators) {
                await this.runValidator(validator);
            }

            // Generate master report
            this.generateMasterReport();

        } catch (error) {
            console.error('❌ Fatal error during validation:', error.message);
            this.results.summary.failed++;
            process.exit(1);
        } finally {
            this.results.summary.endTime = new Date().toISOString();
            this.results.summary.duration = this.calculateDuration();
        }
    }

    async runValidator(validator) {
        console.log(`\n${'▶'.repeat(3)} Running ${validator.name}...`);
        console.log(`Description: ${validator.description}`);
        console.log('-'.repeat(40));

        const startTime = Date.now();

        try {
            // Create validator instance and run
            const validatorInstance = new validator.class();
            await validatorInstance.run();

            // If we get here without throwing, it passed
            this.results.summary.passed++;
            this.results.checks[validator.key] = {
                name: validator.name,
                status: 'passed',
                critical: validator.critical,
                duration: Date.now() - startTime,
                error: null
            };

            console.log(`✅ ${validator.name} completed successfully`);

        } catch (error) {
            // Check if this was a validation failure (exit code 1) vs actual error
            const isValidationFailure = error.code === 1;

            if (isValidationFailure) {
                if (validator.critical) {
                    this.results.summary.failed++;
                } else {
                    this.results.summary.warnings++;
                }
            } else {
                this.results.summary.failed++;
            }

            this.results.checks[validator.key] = {
                name: validator.name,
                status: isValidationFailure ? 'validation_failed' : 'error',
                critical: validator.critical,
                duration: Date.now() - startTime,
                error: error.message
            };

            console.log(`❌ ${validator.name} ${isValidationFailure ? 'found issues' : 'encountered error'}: ${error.message}`);

            // Don't exit on non-critical validation failures
            if (!validator.critical && isValidationFailure) {
                console.log(`⚠️  Non-critical check failed, continuing...`);
            } else if (validator.critical && isValidationFailure) {
                console.log(`🚨 Critical check failed!`);
            }
        }

        console.log(`⏱️  Duration: ${((Date.now() - startTime) / 1000).toFixed(2)}s`);
    }

    calculateDuration() {
        const start = new Date(this.results.summary.startTime);
        const end = new Date(this.results.summary.endTime);
        return Math.round((end - start) / 1000);
    }

    generateMasterReport() {
        console.log('\n' + '='.repeat(60));
        console.log('📊 MASTER VALIDATION REPORT');
        console.log('='.repeat(60));

        // Overall Summary
        console.log('\n📋 Overall Summary:');
        console.log(`  Total Checks: ${this.results.summary.totalChecks}`);
        console.log(`  ✅ Passed: ${this.results.summary.passed}`);
        console.log(`  ❌ Failed: ${this.results.summary.failed}`);
        console.log(`  ⚠️  Warnings: ${this.results.summary.warnings}`);
        console.log(`  ⏱️  Total Duration: ${this.results.summary.duration}s`);

        // Success Rate
        const totalRun = this.results.summary.passed + this.results.summary.failed + this.results.summary.warnings;
        if (totalRun > 0) {
            const successRate = ((this.results.summary.passed / totalRun) * 100).toFixed(1);
            console.log(`  📈 Success Rate: ${successRate}%`);
        }

        // Detailed Results
        console.log('\n📝 Detailed Results:');
        console.log('-'.repeat(60));

        Object.entries(this.results.checks).forEach(([key, result]) => {
            const icon = result.status === 'passed' ? '✅' :
                        result.status === 'validation_failed' ? (result.critical ? '❌' : '⚠️ ') : '💥';
            const critical = result.critical ? '[CRITICAL]' : '[NON-CRITICAL]';
            const duration = `${(result.duration / 1000).toFixed(2)}s`;

            console.log(`${icon} ${result.name} ${critical} (${duration})`);

            if (result.error) {
                console.log(`    Error: ${result.error}`);
            }
        });

        // Critical Issues Summary
        const criticalFailures = Object.values(this.results.checks)
            .filter(check => check.critical && check.status !== 'passed');

        if (criticalFailures.length > 0) {
            console.log('\n🚨 CRITICAL ISSUES:');
            console.log('-'.repeat(60));
            criticalFailures.forEach(failure => {
                console.log(`❌ ${failure.name}: ${failure.error || 'Validation failed'}`);
            });
        }

        // File-based Reports Summary
        console.log('\n📁 Individual Reports Generated:');
        console.log('-'.repeat(60));

        const reportFiles = [
            'dead-link-report.json',
            'html-css-report.json',
            'js-error-report.json',
            'code-syntax-report.json',
            'seo-meta-report.json',
            'build-deployment-report.json',
            'responsive-design-report.json'
        ];

        reportFiles.forEach(file => {
            const filePath = path.join(this.scriptsPath, file);
            if (fs.existsSync(filePath)) {
                const stats = fs.statSync(filePath);
                const sizeKB = Math.round(stats.size / 1024);
                console.log(`📄 ${file} (${sizeKB}KB)`);
            } else {
                console.log(`❌ ${file} (not generated)`);
            }
        });

        // Recommendations
        this.generateRecommendations();

        // Save master report
        this.saveMasterReport();

        // Final Assessment
        this.provideFinalAssessment();
    }

    generateRecommendations() {
        console.log('\n💡 PRIORITY RECOMMENDATIONS:');
        console.log('-'.repeat(60));

        const recommendations = [];

        // Critical issues first
        const criticalFailures = Object.values(this.results.checks)
            .filter(check => check.critical && check.status !== 'passed');

        if (criticalFailures.length > 0) {
            recommendations.push('🚨 CRITICAL: Fix all critical validation failures before deployment');
        }

        // Specific recommendations based on failures
        Object.entries(this.results.checks).forEach(([key, result]) => {
            if (result.status !== 'passed') {
                switch (key) {
                    case 'deadLinks':
                        recommendations.push('• Fix broken links to improve user experience and SEO');
                        break;
                    case 'htmlCss':
                        recommendations.push('• Resolve HTML/CSS validation errors for better browser compatibility');
                        break;
                    case 'javascript':
                        recommendations.push('• Fix JavaScript errors to ensure proper functionality');
                        break;
                    case 'buildDeploy':
                        recommendations.push('• Resolve build/deployment issues to ensure reliable deployments');
                        break;
                    case 'seoMeta':
                        recommendations.push('• Improve SEO meta tags for better search engine visibility');
                        break;
                    case 'codeSyntax':
                        recommendations.push('• Fix code syntax highlighting for better documentation');
                        break;
                    case 'responsive':
                        recommendations.push('• Improve responsive design for better mobile experience');
                        break;
                }
            }
        });

        // General recommendations
        if (recommendations.length === 0) {
            recommendations.push('✨ All validations passed! Consider these enhancements:');
            recommendations.push('• Implement performance optimizations');
            recommendations.push('• Add more comprehensive error handling');
            recommendations.push('• Consider accessibility improvements');
            recommendations.push('• Set up automated testing pipeline');
        }

        recommendations.forEach(rec => console.log(rec));
    }

    saveMasterReport() {
        const reportData = {
            timestamp: new Date().toISOString(),
            summary: this.results.summary,
            checks: this.results.checks,
            website: 'Caxton Multi-Agent Orchestration Platform',
            validation_suite_version: '1.0.0'
        };

        const reportPath = path.join(this.scriptsPath, 'master-validation-report.json');
        fs.writeFileSync(reportPath, JSON.stringify(reportData, null, 2));

        console.log(`\n📁 Master report saved to: ${reportPath}`);
    }

    provideFinalAssessment() {
        const criticalFailures = Object.values(this.results.checks)
            .filter(check => check.critical && check.status !== 'passed').length;

        const totalIssues = this.results.summary.failed + this.results.summary.warnings;

        console.log('\n' + '🎯 FINAL ASSESSMENT'.padStart(40));
        console.log('='.repeat(60));

        if (criticalFailures === 0 && totalIssues === 0) {
            console.log('🎉 EXCELLENT: All validations passed!');
            console.log('   Your website is ready for production deployment.');
            console.log('   Consider implementing continuous validation in your CI/CD pipeline.');
        } else if (criticalFailures === 0) {
            console.log('✅ GOOD: No critical issues found.');
            console.log(`   ${totalIssues} non-critical issue(s) should be addressed for optimal quality.`);
            console.log('   Website is deployable but improvements recommended.');
        } else if (criticalFailures <= 2) {
            console.log('⚠️  ATTENTION REQUIRED: Critical issues detected.');
            console.log(`   ${criticalFailures} critical issue(s) must be resolved before deployment.`);
            console.log('   Address critical issues first, then work on non-critical items.');
        } else {
            console.log('🚨 SIGNIFICANT ISSUES: Multiple critical failures detected.');
            console.log(`   ${criticalFailures} critical issue(s) require immediate attention.`);
            console.log('   Deployment not recommended until issues are resolved.');
        }

        console.log('\nFor detailed information, check individual JSON reports in the validation directory.');
        console.log(`Validation completed at: ${new Date().toLocaleString()}`);

        // Exit with appropriate code
        if (criticalFailures > 0) {
            process.exit(1);
        } else if (totalIssues > 0) {
            process.exit(2); // Non-critical issues
        } else {
            process.exit(0); // All good
        }
    }

    // Alternative method to run validations as separate processes (if needed)
    async runAsProcess(scriptName) {
        return new Promise((resolve, reject) => {
            const scriptPath = path.join(this.scriptsPath, scriptName);

            exec(`node "${scriptPath}"`, { cwd: this.scriptsPath }, (error, stdout, stderr) => {
                if (error) {
                    reject(error);
                } else {
                    resolve({ stdout, stderr });
                }
            });
        });
    }
}

// CLI usage information
function showUsage() {
    console.log(`
Usage: node run-all-validations.js [options]

Options:
  --help, -h     Show this help message
  --version, -v  Show version information
  --only <check> Run only specific validation check
  --skip <check> Skip specific validation check

Available checks:
  - deadLinks      Dead link detection
  - htmlCss        HTML/CSS validation
  - javascript     JavaScript error detection
  - codeSyntax     Code syntax highlighting
  - seoMeta        SEO meta tags validation
  - buildDeploy    Build/deployment checks
  - responsive     Responsive design validation

Examples:
  node run-all-validations.js
  node run-all-validations.js --only seoMeta
  node run-all-validations.js --skip buildDeploy
    `);
}

// Main execution
if (require.main === module) {
    const args = process.argv.slice(2);

    if (args.includes('--help') || args.includes('-h')) {
        showUsage();
        process.exit(0);
    }

    if (args.includes('--version') || args.includes('-v')) {
        console.log('Caxton Website Validation Suite v1.0.0');
        process.exit(0);
    }

    const runner = new MasterValidationRunner();

    // Handle --only and --skip options (basic implementation)
    if (args.includes('--only')) {
        const checkIndex = args.indexOf('--only') + 1;
        if (checkIndex < args.length) {
            const checkName = args[checkIndex];
            runner.validators = runner.validators.filter(v => v.key === checkName);
            runner.results.summary.totalChecks = runner.validators.length;
        }
    }

    if (args.includes('--skip')) {
        const checkIndex = args.indexOf('--skip') + 1;
        if (checkIndex < args.length) {
            const checkName = args[checkIndex];
            runner.validators = runner.validators.filter(v => v.key !== checkName);
            runner.results.summary.totalChecks = runner.validators.length;
        }
    }

    runner.run();
}

module.exports = MasterValidationRunner;
