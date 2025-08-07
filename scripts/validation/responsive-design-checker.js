#!/usr/bin/env node

/**
 * Responsive Design Breakpoints Checker for Caxton Website
 * Validates CSS breakpoints, responsive behavior, and mobile-first design
 */

const fs = require('fs');
const path = require('path');

class ResponsiveDesignChecker {
    constructor() {
        // Determine correct site path - look for website directory from current working directory
        let sitePath = path.join(process.cwd(), 'website');

        // If running from validation directory, go up to project root
        if (process.cwd().includes('scripts/validation')) {
            sitePath = path.join(process.cwd(), '..', '..', 'website');
        }

        this.sitePath = sitePath;
        this.results = {
            breakpoints: { found: [], missing: [], issues: [] },
            viewport: { configured: false, issues: [] },
            media: { queries: 0, issues: [] },
            images: { responsive: 0, total: 0, issues: [] },
            typography: { responsive: false, issues: [] },
            navigation: { mobile: false, issues: [] },
            layout: { issues: [] },
            testing: { suggestions: [] }
        };

        // Common responsive breakpoints
        this.standardBreakpoints = {
            'mobile-small': '320px',
            'mobile': '480px',
            'tablet': '768px',
            'desktop': '1024px',
            'desktop-large': '1200px',
            'desktop-xl': '1440px'
        };
    }

    async run() {
        console.log('📱 Starting Responsive Design Check...');
        console.log(`Site Path: ${this.sitePath}`);
        console.log('─'.repeat(60));

        try {
            await this.checkViewportConfiguration();
            await this.analyzeBreakpoints();
            await this.checkMediaQueries();
            await this.validateResponsiveImages();
            await this.checkResponsiveTypography();
            await this.validateMobileNavigation();
            await this.analyzeLayoutPatterns();
            await this.generateTestingSuggestions();
            this.generateReport();
        } catch (error) {
            console.error('❌ Error during responsive design check:', error.message);
            process.exit(1);
        }
    }

    async checkViewportConfiguration() {
        console.log('\n📐 Checking Viewport Configuration...');

        const htmlFiles = this.findFiles(['.html']);
        let viewportConfigured = false;

        for (const file of htmlFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            // Check for viewport meta tag
            const viewportMatch = content.match(/<meta[^>]*name=["']viewport["'][^>]*>/i);
            if (viewportMatch) {
                viewportConfigured = true;
                console.log(`  ✅ Viewport configured in ${relativePath}`);

                // Validate viewport content
                const contentMatch = viewportMatch[0].match(/content=["']([^"']*)["']/i);
                if (contentMatch) {
                    this.validateViewportContent(contentMatch[1], relativePath);
                }
                break;
            }
        }

        if (!viewportConfigured) {
            this.results.viewport.issues.push('No viewport meta tag found');
            console.log('  ❌ No viewport meta tag found');
        } else {
            this.results.viewport.configured = true;
        }
    }

    validateViewportContent(content, file) {
        const requiredParts = ['width=device-width', 'initial-scale=1'];
        const recommendedParts = ['shrink-to-fit=no'];

        requiredParts.forEach(part => {
            if (!content.includes(part)) {
                this.results.viewport.issues.push(`Missing viewport setting: ${part} in ${file}`);
                console.log(`    ⚠️  Missing: ${part}`);
            } else {
                console.log(`    ✅ Found: ${part}`);
            }
        });

        // Check for problematic settings
        if (content.includes('user-scalable=no')) {
            this.results.viewport.issues.push('user-scalable=no may harm accessibility');
            console.log('    ⚠️  user-scalable=no detected (accessibility concern)');
        }

        if (content.includes('maximum-scale=1')) {
            this.results.viewport.issues.push('maximum-scale=1 prevents zoom (accessibility issue)');
            console.log('    ⚠️  maximum-scale=1 detected (accessibility concern)');
        }
    }

    async analyzeBreakpoints() {
        console.log('\n📏 Analyzing CSS Breakpoints...');

        const cssFiles = this.findFiles(['.css', '.scss']);
        const foundBreakpoints = new Set();
        let totalQueries = 0;

        for (const file of cssFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`  Analyzing: ${relativePath}`);

            const content = fs.readFileSync(file, 'utf8');
            const queries = this.extractMediaQueries(content);

            totalQueries += queries.length;

            queries.forEach(query => {
                foundBreakpoints.add(query.breakpoint);
                console.log(`    📱 ${query.type}: ${query.breakpoint} (${query.condition})`);
            });
        }

        this.results.media.queries = totalQueries;
        this.results.breakpoints.found = Array.from(foundBreakpoints);

        // Check for mobile-first approach
        this.checkMobileFirst(cssFiles);

        // Identify missing common breakpoints
        this.identifyMissingBreakpoints(foundBreakpoints);
    }

    extractMediaQueries(content) {
        const queries = [];

        // Match media queries
        const mediaQueryRegex = /@media\s*([^{]+)\{/gi;
        let match;

        while ((match = mediaQueryRegex.exec(content)) !== null) {
            const condition = match[1].trim();
            const breakpoint = this.extractBreakpointValue(condition);

            queries.push({
                condition: condition,
                breakpoint: breakpoint,
                type: this.categorizeMediaQuery(condition)
            });
        }

        return queries;
    }

    extractBreakpointValue(condition) {
        // Extract pixel values from media queries
        const pixelMatch = condition.match(/(\d+)px/);
        if (pixelMatch) {
            return pixelMatch[1] + 'px';
        }

        // Extract em values
        const emMatch = condition.match(/(\d+(?:\.\d+)?)em/);
        if (emMatch) {
            return emMatch[1] + 'em';
        }

        return 'custom';
    }

    categorizeMediaQuery(condition) {
        if (condition.includes('min-width')) {
            return 'min-width';
        } else if (condition.includes('max-width')) {
            return 'max-width';
        } else if (condition.includes('orientation')) {
            return 'orientation';
        } else if (condition.includes('hover')) {
            return 'hover';
        } else {
            return 'other';
        }
    }

    checkMobileFirst(cssFiles) {
        let mobileFirstCount = 0;
        let desktopFirstCount = 0;

        for (const file of cssFiles) {
            const content = fs.readFileSync(file, 'utf8');
            const minWidthQueries = (content.match(/@media[^{]*min-width/gi) || []).length;
            const maxWidthQueries = (content.match(/@media[^{]*max-width/gi) || []).length;

            mobileFirstCount += minWidthQueries;
            desktopFirstCount += maxWidthQueries;
        }

        if (mobileFirstCount > desktopFirstCount) {
            console.log('  ✅ Mobile-first approach detected');
        } else if (desktopFirstCount > mobileFirstCount) {
            this.results.breakpoints.issues.push('Desktop-first approach detected (consider mobile-first)');
            console.log('  ⚠️  Desktop-first approach detected');
        } else {
            console.log('  ℹ️  Mixed breakpoint approach');
        }
    }

    identifyMissingBreakpoints(foundBreakpoints) {
        const commonPixelBreakpoints = ['480px', '768px', '1024px', '1200px'];

        commonPixelBreakpoints.forEach(bp => {
            if (!foundBreakpoints.has(bp)) {
                this.results.breakpoints.missing.push(bp);
            }
        });

        if (this.results.breakpoints.missing.length > 0) {
            console.log(`  ℹ️  Common breakpoints not found: ${this.results.breakpoints.missing.join(', ')}`);
        }
    }

    async checkMediaQueries() {
        console.log('\n🎯 Checking Media Query Best Practices...');

        const cssFiles = this.findFiles(['.css', '.scss']);

        for (const file of cssFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            // Check for print styles
            if (content.includes('@media print')) {
                console.log(`  ✅ Print styles found in ${relativePath}`);
            }

            // Check for prefers-reduced-motion
            if (content.includes('prefers-reduced-motion')) {
                console.log(`  ✅ Reduced motion support in ${relativePath}`);
            } else {
                this.results.media.issues.push('No prefers-reduced-motion support found');
            }

            // Check for hover support
            if (content.includes('@media (hover:')) {
                console.log(`  ✅ Hover media queries in ${relativePath}`);
            }

            // Check for dark/light mode support
            if (content.includes('prefers-color-scheme')) {
                console.log(`  ✅ Color scheme preference support in ${relativePath}`);
            }

            // Validate media query syntax
            this.validateMediaQuerySyntax(content, relativePath);
        }
    }

    validateMediaQuerySyntax(content, file) {
        // Check for common syntax errors
        const issues = [];

        // Check for missing units
        const noUnitMatch = content.match(/@media[^{]*\d+\s*\)/g);
        if (noUnitMatch) {
            issues.push('Media query values missing units');
        }

        // Check for overly specific queries
        const complexQueries = content.match(/@media[^{]{100,}/g);
        if (complexQueries) {
            issues.push('Overly complex media queries found');
        }

        issues.forEach(issue => {
            this.results.media.issues.push(`${file}: ${issue}`);
            console.log(`    ⚠️  ${issue} in ${file}`);
        });
    }

    async validateResponsiveImages() {
        console.log('\n🖼️  Validating Responsive Images...');

        const htmlFiles = this.findFiles(['.html']);
        const mdFiles = this.findFiles(['.md', '.markdown']);
        const allFiles = [...htmlFiles, ...mdFiles];

        for (const file of allFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            const images = this.extractAllImages(content);
            this.results.images.total += images.length;

            images.forEach(image => {
                if (image.isResponsive) {
                    this.results.images.responsive++;
                    console.log(`    ✅ Responsive: ${image.src}`);
                } else {
                    this.results.images.issues.push({
                        file: relativePath,
                        image: image.src,
                        issue: 'Image not responsive'
                    });
                    console.log(`    ⚠️  Fixed size: ${image.src}`);
                }
            });

            if (images.length > 0) {
                console.log(`  ${relativePath}: ${images.length} images, ${images.filter(i => i.isResponsive).length} responsive`);
            }
        }
    }

    extractAllImages(content) {
        const images = [];

        // HTML images
        const htmlImages = content.match(/<img[^>]*>/gi) || [];
        htmlImages.forEach(imgTag => {
            const src = (imgTag.match(/src=["']([^"']*)["']/i) || [])[1];
            if (src) {
                images.push({
                    src: src,
                    isResponsive: this.checkImageResponsiveness(imgTag),
                    type: 'html'
                });
            }
        });

        // CSS background images (basic check)
        const cssBackgrounds = content.match(/background-image:\s*url\([^)]+\)/gi) || [];
        cssBackgrounds.forEach(bg => {
            const url = (bg.match(/url\(["']?([^"')]+)["']?\)/i) || [])[1];
            if (url) {
                images.push({
                    src: url,
                    isResponsive: false, // CSS backgrounds need media queries for responsiveness
                    type: 'css-background'
                });
            }
        });

        return images;
    }

    checkImageResponsiveness(imgTag) {
        // Check for responsive attributes
        if (imgTag.includes('max-width') && imgTag.includes('100%')) {
            return true;
        }

        if (imgTag.includes('width="100%"')) {
            return true;
        }

        if (imgTag.includes('class=') &&
            (imgTag.includes('responsive') ||
             imgTag.includes('img-fluid') ||
             imgTag.includes('img-responsive'))) {
            return true;
        }

        // Check for srcset attribute (responsive images)
        if (imgTag.includes('srcset=')) {
            return true;
        }

        return false;
    }

    async checkResponsiveTypography() {
        console.log('\n🔤 Checking Responsive Typography...');

        const cssFiles = this.findFiles(['.css', '.scss']);
        let hasResponsiveText = false;

        for (const file of cssFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            // Check for responsive font sizes
            if (content.includes('clamp(') ||
                content.includes('calc(') && content.includes('vw') ||
                content.match(/font-size:\s*\d+(?:\.\d+)?vw/)) {
                hasResponsiveText = true;
                console.log(`  ✅ Responsive font sizes in ${relativePath}`);
            }

            // Check for font-size in media queries
            const fontSizeInMedia = content.match(/@media[^}]*font-size[^}]*}/gi);
            if (fontSizeInMedia) {
                hasResponsiveText = true;
                console.log(`  ✅ Font size breakpoints in ${relativePath}`);
            }

            // Check for rem/em usage
            const hasRelativeUnits = content.match(/font-size:\s*\d+(?:\.\d+)?(?:rem|em)/g);
            if (hasRelativeUnits) {
                console.log(`  ✅ Relative font units in ${relativePath}`);
            }
        }

        this.results.typography.responsive = hasResponsiveText;

        if (!hasResponsiveText) {
            this.results.typography.issues.push('No responsive typography detected');
            console.log('  ⚠️  No responsive typography detected');
        }
    }

    async validateMobileNavigation() {
        console.log('\n🍔 Validating Mobile Navigation...');

        const htmlFiles = this.findFiles(['.html']);
        const jsFiles = this.findFiles(['.js']);

        // Check HTML for mobile navigation patterns
        for (const file of htmlFiles) {
            const content = fs.readFileSync(file, 'utf8');

            // Check for hamburger menu elements
            const mobileNavPatterns = [
                /(hamburger|menu-toggle|mobile-menu)/i,
                /aria-expanded/i,
                /class=["'][^"']*nav[^"']*toggle/i
            ];

            const hasMobileNav = mobileNavPatterns.some(pattern => content.match(pattern));
            if (hasMobileNav) {
                this.results.navigation.mobile = true;
                console.log(`  ✅ Mobile navigation detected in ${path.relative(this.sitePath, file)}`);
            }
        }

        // Check JavaScript for mobile navigation functionality
        for (const file of jsFiles) {
            const content = fs.readFileSync(file, 'utf8');

            if (content.includes('toggle') &&
                (content.includes('menu') || content.includes('nav'))) {
                console.log(`  ✅ Mobile navigation JS in ${path.relative(this.sitePath, file)}`);
            }
        }

        if (!this.results.navigation.mobile) {
            this.results.navigation.issues.push('No mobile navigation pattern detected');
            console.log('  ⚠️  No mobile navigation detected');
        }
    }

    async analyzeLayoutPatterns() {
        console.log('\n📐 Analyzing Layout Patterns...');

        const cssFiles = this.findFiles(['.css', '.scss']);

        for (const file of cssFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            // Check for modern layout techniques
            if (content.includes('display: grid')) {
                console.log(`  ✅ CSS Grid in ${relativePath}`);
            }

            if (content.includes('display: flex')) {
                console.log(`  ✅ Flexbox in ${relativePath}`);
            }

            // Check for container queries (future-facing)
            if (content.includes('@container')) {
                console.log(`  ✅ Container queries in ${relativePath}`);
            }

            // Check for problematic patterns
            if (content.includes('float:') &&
                !content.includes('clear:')) {
                this.results.layout.issues.push(`${relativePath}: Float without clear may cause layout issues`);
                console.log(`    ⚠️  Float without clear in ${relativePath}`);
            }

            // Check for fixed widths
            const fixedWidths = content.match(/width:\s*\d+px/g);
            if (fixedWidths && fixedWidths.length > 3) {
                this.results.layout.issues.push(`${relativePath}: Many fixed pixel widths (consider percentages/flexbox)`);
                console.log(`    ⚠️  Many fixed widths in ${relativePath}`);
            }
        }
    }

    async generateTestingSuggestions() {
        console.log('\n🧪 Generating Testing Suggestions...');

        const suggestions = [
            'Test on multiple devices: iPhone, Android, iPad, desktop',
            'Use browser developer tools to simulate different screen sizes',
            'Test with browser zoom at 200% and 300%',
            'Verify touch targets are at least 44px × 44px',
            'Check horizontal scrolling doesn\'t occur',
            'Test navigation usability on mobile devices',
            'Validate form inputs work properly on mobile',
            'Check image loading and sizing on different screens',
            'Test typography readability on small screens',
            'Verify performance on slower mobile connections'
        ];

        this.results.testing.suggestions = suggestions;

        suggestions.forEach(suggestion => {
            console.log(`  📝 ${suggestion}`);
        });
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
        console.log('📋 RESPONSIVE DESIGN REPORT');
        console.log('='.repeat(60));

        // Viewport Summary
        console.log('\n📐 Viewport Configuration:');
        console.log(`  ✅ Configured: ${this.results.viewport.configured ? 'Yes' : 'No'}`);
        console.log(`  ❌ Issues: ${this.results.viewport.issues.length}`);

        // Breakpoints Summary
        console.log('\n📏 Breakpoints:');
        console.log(`  ✅ Found: ${this.results.breakpoints.found.length} (${this.results.breakpoints.found.join(', ')})`);
        console.log(`  ❌ Missing common: ${this.results.breakpoints.missing.length}`);
        console.log(`  ❌ Issues: ${this.results.breakpoints.issues.length}`);

        // Media Queries Summary
        console.log('\n🎯 Media Queries:');
        console.log(`  Total queries: ${this.results.media.queries}`);
        console.log(`  ❌ Issues: ${this.results.media.issues.length}`);

        // Images Summary
        console.log('\n🖼️  Images:');
        console.log(`  Total images: ${this.results.images.total}`);
        console.log(`  ✅ Responsive: ${this.results.images.responsive}`);
        console.log(`  ❌ Issues: ${this.results.images.issues.length}`);

        if (this.results.images.total > 0) {
            const responsiveRate = ((this.results.images.responsive / this.results.images.total) * 100).toFixed(1);
            console.log(`  📈 Responsive rate: ${responsiveRate}%`);
        }

        // Typography Summary
        console.log('\n🔤 Typography:');
        console.log(`  ✅ Responsive: ${this.results.typography.responsive ? 'Yes' : 'No'}`);
        console.log(`  ❌ Issues: ${this.results.typography.issues.length}`);

        // Navigation Summary
        console.log('\n🍔 Mobile Navigation:');
        console.log(`  ✅ Detected: ${this.results.navigation.mobile ? 'Yes' : 'No'}`);
        console.log(`  ❌ Issues: ${this.results.navigation.issues.length}`);

        // Layout Summary
        console.log('\n📐 Layout:');
        console.log(`  ❌ Issues: ${this.results.layout.issues.length}`);

        // Detailed Issues
        const allIssues = [
            ...this.results.viewport.issues.map(i => ({ type: 'Viewport', issue: i })),
            ...this.results.breakpoints.issues.map(i => ({ type: 'Breakpoints', issue: i })),
            ...this.results.media.issues.map(i => ({ type: 'Media Queries', issue: i })),
            ...this.results.images.issues.map(i => ({ type: 'Images', issue: `${i.file}: ${i.issue} (${i.image})` })),
            ...this.results.typography.issues.map(i => ({ type: 'Typography', issue: i })),
            ...this.results.navigation.issues.map(i => ({ type: 'Navigation', issue: i })),
            ...this.results.layout.issues.map(i => ({ type: 'Layout', issue: i }))
        ];

        if (allIssues.length > 0) {
            console.log('\n🔍 DETAILED ISSUES:');
            console.log('-'.repeat(60));

            const groupedIssues = {};
            allIssues.forEach(item => {
                if (!groupedIssues[item.type]) {
                    groupedIssues[item.type] = [];
                }
                groupedIssues[item.type].push(item.issue);
            });

            Object.keys(groupedIssues).forEach(type => {
                console.log(`\n${type}:`);
                groupedIssues[type].forEach(issue => {
                    console.log(`  ❌ ${issue}`);
                });
            });
        }

        // Testing Suggestions
        console.log('\n🧪 TESTING SUGGESTIONS:');
        console.log('-'.repeat(60));
        this.results.testing.suggestions.forEach(suggestion => {
            console.log(`• ${suggestion}`);
        });

        // Responsive Recommendations
        console.log('\n💡 RESPONSIVE DESIGN RECOMMENDATIONS:');
        console.log('-'.repeat(60));

        const recommendations = [
            '• Use mobile-first responsive design approach',
            '• Implement flexible grid systems (CSS Grid, Flexbox)',
            '• Use relative units (rem, em, %) instead of fixed pixels',
            '• Optimize images with srcset for different screen densities',
            '• Test touch interactions and minimum target sizes',
            '• Consider progressive enhancement for JavaScript features',
            '• Implement proper focus management for keyboard navigation',
            '• Use semantic HTML for better screen reader support',
            '• Test with reduced motion preferences enabled',
            '• Optimize performance for slower mobile connections'
        ];

        recommendations.forEach(rec => console.log(rec));

        // Save detailed report
        const reportData = {
            timestamp: new Date().toISOString(),
            results: this.results,
            testingSuggestions: this.results.testing.suggestions,
            recommendations: recommendations
        };

        fs.writeFileSync(
            '/workspaces/caxton/scripts/validation/responsive-design-report.json',
            JSON.stringify(reportData, null, 2)
        );

        console.log('\n📁 Detailed report saved to: responsive-design-report.json');

        const hasErrors = !this.results.viewport.configured ||
                         this.results.images.issues.length > this.results.images.total * 0.5 ||
                         !this.results.navigation.mobile;

        if (hasErrors) {
            console.log('\n🚨 RESPONSIVE DESIGN ISSUES DETECTED');
            process.exit(1);
        }
    }
}

// Run if called directly
if (require.main === module) {
    const checker = new ResponsiveDesignChecker();
    checker.run();
}

module.exports = ResponsiveDesignChecker;
