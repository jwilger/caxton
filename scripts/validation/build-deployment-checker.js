#!/usr/bin/env node

/**
 * Build and Deployment Issues Checker for Caxton Website
 * Validates Jekyll build process, GitHub Pages compatibility, and deployment configuration
 */

const fs = require('fs');
const path = require('path');
const { execSync, exec } = require('child_process');

class BuildDeploymentChecker {
    constructor() {
        // Determine correct paths - look for website directory from current working directory
        let sitePath = path.join(process.cwd(), 'website');
        let rootPath = process.cwd();

        // If running from validation directory, go up to project root
        if (process.cwd().includes('scripts/validation')) {
            sitePath = path.join(process.cwd(), '..', '..', 'website');
            rootPath = path.join(process.cwd(), '..', '..');
        }

        this.sitePath = sitePath;
        this.rootPath = rootPath;
        this.results = {
            jekyll: { configured: false, issues: [] },
            github: { configured: false, issues: [] },
            build: { successful: false, issues: [] },
            dependencies: { issues: [] },
            assets: { issues: [] },
            config: { issues: [] },
            performance: { issues: [] }
        };
    }

    async run() {
        console.log('🔍 Starting Build/Deployment Issues Check...');
        console.log(`Site Path: ${this.sitePath}`);
        console.log(`Root Path: ${this.rootPath}`);
        console.log('─'.repeat(60));

        try {
            await this.checkJekyllConfiguration();
            await this.checkGitHubPagesCompatibility();
            await this.checkDependencies();
            await this.validateAssets();
            await this.testBuildProcess();
            await this.checkDeploymentConfiguration();
            await this.performanceChecks();
            this.generateReport();
        } catch (error) {
            console.error('❌ Error during build/deployment check:', error.message);
            process.exit(1);
        }
    }

    async checkJekyllConfiguration() {
        console.log('\n🔧 Checking Jekyll Configuration...');

        const configPath = path.join(this.sitePath, '_config.yml');

        if (!fs.existsSync(configPath)) {
            this.results.jekyll.issues.push('Missing _config.yml file');
            console.log('  ❌ No _config.yml found');
            return;
        }

        console.log('  ✅ _config.yml found');
        this.results.jekyll.configured = true;

        const config = fs.readFileSync(configPath, 'utf8');
        await this.validateJekyllConfig(config);
    }

    async validateJekyllConfig(config) {
        // Check essential configuration
        const essentialSettings = [
            { key: 'title', required: true },
            { key: 'description', required: true },
            { key: 'url', required: true },
            { key: 'baseurl', required: false },
            { key: 'markdown', required: false, recommended: 'kramdown' },
            { key: 'highlighter', required: false, recommended: 'rouge' }
        ];

        essentialSettings.forEach(setting => {
            const hasKey = config.includes(`${setting.key}:`);

            if (setting.required && !hasKey) {
                this.results.jekyll.issues.push(`Missing required setting: ${setting.key}`);
                console.log(`  ❌ Missing: ${setting.key}`);
            } else if (hasKey) {
                console.log(`  ✅ Found: ${setting.key}`);

                if (setting.recommended) {
                    const valueMatch = config.match(new RegExp(`${setting.key}:\\s*(.+)`));
                    if (valueMatch && !valueMatch[1].includes(setting.recommended)) {
                        this.results.jekyll.issues.push(`Consider using ${setting.recommended} for ${setting.key}`);
                        console.log(`  ⚠️  Recommended: ${setting.key}: ${setting.recommended}`);
                    }
                }
            }
        });

        // Check for GitHub Pages compatibility
        if (config.includes('plugins:')) {
            const pluginSection = this.extractConfigSection(config, 'plugins');
            this.validatePlugins(pluginSection);
        }

        // Check for safe mode compatibility
        if (config.includes('safe: false')) {
            this.results.jekyll.issues.push('safe: false may not work on GitHub Pages');
            console.log('  ⚠️  safe: false detected');
        }

        // Check for custom gems that might not be supported
        this.checkUnsupportedFeatures(config);
    }

    extractConfigSection(config, section) {
        const regex = new RegExp(`${section}:\\s*\\n((\\s+-.+\\n)*|(\\s+[^\\n]+\\n)*)`, 'g');
        const match = config.match(regex);
        return match ? match[0] : '';
    }

    validatePlugins(pluginSection) {
        const supportedPlugins = [
            'jekyll-coffeescript',
            'jekyll-default-layout',
            'jekyll-gist',
            'jekyll-github-metadata',
            'jekyll-paginate',
            'jekyll-relative-links',
            'jekyll-optional-front-matter',
            'jekyll-readme-index',
            'jekyll-redirect-from',
            'jekyll-sass-converter',
            'jekyll-sitemap',
            'jekyll-swiss',
            'jekyll-theme-architect',
            'jekyll-theme-cayman',
            'jekyll-theme-dinky',
            'jekyll-theme-hacker',
            'jekyll-theme-leap-day',
            'jekyll-theme-merlot',
            'jekyll-theme-midnight',
            'jekyll-theme-minimal',
            'jekyll-theme-modernist',
            'jekyll-theme-primer',
            'jekyll-theme-slate',
            'jekyll-theme-tactile',
            'jekyll-theme-time-machine',
            'jekyll-titles-from-headings',
            'jemoji',
            'kramdown',
            'liquid',
            'rouge',
            'safe_yaml'
        ];

        const pluginMatches = pluginSection.match(/- ([\w-]+)/g) || [];
        pluginMatches.forEach(match => {
            const plugin = match.replace('- ', '');
            if (!supportedPlugins.includes(plugin)) {
                this.results.jekyll.issues.push(`Unsupported plugin for GitHub Pages: ${plugin}`);
                console.log(`  ⚠️  Unsupported plugin: ${plugin}`);
            } else {
                console.log(`  ✅ Supported plugin: ${plugin}`);
            }
        });
    }

    checkUnsupportedFeatures(config) {
        const unsupportedFeatures = [
            { pattern: /custom_plugins/, message: 'Custom plugins not supported on GitHub Pages' },
            { pattern: /gems:/, message: 'Use plugins: instead of gems: for GitHub Pages' },
            { pattern: /^(?!#).*\.rb$/, message: 'Ruby plugins not supported on GitHub Pages' }
        ];

        unsupportedFeatures.forEach(feature => {
            if (config.match(feature.pattern)) {
                this.results.jekyll.issues.push(feature.message);
                console.log(`  ⚠️  ${feature.message}`);
            }
        });
    }

    async checkGitHubPagesCompatibility() {
        console.log('\n🐙 Checking GitHub Pages Compatibility...');

        // Check if this is a GitHub repository
        const gitPath = path.join(this.rootPath, '.git');
        if (!fs.existsSync(gitPath)) {
            this.results.github.issues.push('Not a Git repository');
            console.log('  ❌ Not a Git repository');
            return;
        }

        // Check for GitHub Pages workflow
        const workflowsPath = path.join(this.rootPath, '.github', 'workflows');
        if (fs.existsSync(workflowsPath)) {
            const workflowFiles = fs.readdirSync(workflowsPath);
            const jekyllWorkflow = workflowFiles.find(file =>
                file.includes('jekyll') || file.includes('pages') || file.includes('deploy')
            );

            if (jekyllWorkflow) {
                console.log(`  ✅ GitHub Actions workflow found: ${jekyllWorkflow}`);
                this.results.github.configured = true;
                await this.validateGitHubWorkflow(path.join(workflowsPath, jekyllWorkflow));
            } else {
                this.results.github.issues.push('No GitHub Pages deployment workflow found');
                console.log('  ⚠️  No GitHub Pages workflow detected');
            }
        }

        // Check repository structure
        this.checkRepositoryStructure();
    }

    async validateGitHubWorkflow(workflowPath) {
        const workflow = fs.readFileSync(workflowPath, 'utf8');

        // Check for required workflow components
        const requiredComponents = [
            { pattern: /actions\/checkout/, name: 'checkout action' },
            { pattern: /actions\/configure-pages/, name: 'configure-pages action' },
            { pattern: /actions\/upload-pages-artifact/, name: 'upload-pages-artifact action' },
            { pattern: /actions\/deploy-pages/, name: 'deploy-pages action' }
        ];

        requiredComponents.forEach(component => {
            if (workflow.match(component.pattern)) {
                console.log(`    ✅ ${component.name} configured`);
            } else {
                this.results.github.issues.push(`Missing workflow component: ${component.name}`);
                console.log(`    ⚠️  Missing: ${component.name}`);
            }
        });

        // Check for proper permissions
        if (!workflow.includes('pages: write')) {
            this.results.github.issues.push('Workflow missing pages: write permission');
            console.log('    ⚠️  Missing pages: write permission');
        }

        if (!workflow.includes('id-token: write')) {
            this.results.github.issues.push('Workflow missing id-token: write permission');
            console.log('    ⚠️  Missing id-token: write permission');
        }
    }

    checkRepositoryStructure() {
        // Check for proper source directory structure
        const expectedDirs = ['_layouts', '_includes', '_sass', 'assets'];
        expectedDirs.forEach(dir => {
            const dirPath = path.join(this.sitePath, dir);
            if (fs.existsSync(dirPath)) {
                console.log(`  ✅ Found: ${dir}/`);
            } else {
                console.log(`  ℹ️  Optional: ${dir}/ not found`);
            }
        });

        // Check for CNAME file (if using custom domain)
        const cnamePath = path.join(this.sitePath, 'CNAME');
        if (fs.existsSync(cnamePath)) {
            const cname = fs.readFileSync(cnamePath, 'utf8').trim();
            console.log(`  ✅ Custom domain configured: ${cname}`);
        }
    }

    async checkDependencies() {
        console.log('\n📦 Checking Dependencies...');

        const gemfilePath = path.join(this.sitePath, 'Gemfile');
        if (!fs.existsSync(gemfilePath)) {
            this.results.dependencies.issues.push('No Gemfile found');
            console.log('  ❌ No Gemfile found');
            return;
        }

        console.log('  ✅ Gemfile found');
        const gemfile = fs.readFileSync(gemfilePath, 'utf8');

        // Check for essential gems
        const essentialGems = [
            { name: 'jekyll', pattern: /gem\s+["']jekyll["']/ },
            { name: 'github-pages', pattern: /gem\s+["']github-pages["']/ }
        ];

        let hasJekyll = false;
        let hasGitHubPages = false;

        essentialGems.forEach(gem => {
            if (gemfile.match(gem.pattern)) {
                console.log(`  ✅ ${gem.name} dependency found`);
                if (gem.name === 'jekyll') hasJekyll = true;
                if (gem.name === 'github-pages') hasGitHubPages = true;
            }
        });

        if (!hasJekyll && !hasGitHubPages) {
            this.results.dependencies.issues.push('Missing Jekyll or github-pages gem');
            console.log('  ❌ Missing Jekyll dependency');
        }

        // Check for Gemfile.lock
        const gemfileLockPath = path.join(this.sitePath, 'Gemfile.lock');
        if (fs.existsSync(gemfileLockPath)) {
            console.log('  ✅ Gemfile.lock found');
        } else {
            this.results.dependencies.issues.push('Gemfile.lock missing - run bundle install');
            console.log('  ⚠️  Gemfile.lock missing');
        }

        // Try to check bundle status (if bundler is available)
        try {
            execSync('which bundle', { cwd: this.sitePath, stdio: 'pipe' });
            console.log('  ✅ Bundler available');

            try {
                execSync('bundle check', { cwd: this.sitePath, stdio: 'pipe' });
                console.log('  ✅ Bundle dependencies satisfied');
            } catch (error) {
                this.results.dependencies.issues.push('Bundle dependencies not satisfied - run bundle install');
                console.log('  ⚠️  Bundle dependencies not satisfied');
            }
        } catch (error) {
            console.log('  ℹ️  Bundler not available - cannot check bundle status');
        }
    }

    async validateAssets() {
        console.log('\n🎨 Validating Assets...');

        const assetsPath = path.join(this.sitePath, 'assets');
        if (!fs.existsSync(assetsPath)) {
            this.results.assets.issues.push('No assets directory found');
            console.log('  ⚠️  No assets/ directory');
            return;
        }

        console.log('  ✅ Assets directory found');

        // Check asset structure
        const assetDirs = ['css', 'js', 'img', 'images'];
        assetDirs.forEach(dir => {
            const dirPath = path.join(assetsPath, dir);
            if (fs.existsSync(dirPath)) {
                const files = fs.readdirSync(dirPath);
                console.log(`  ✅ ${dir}/: ${files.length} files`);

                // Check for large files
                files.forEach(file => {
                    const filePath = path.join(dirPath, file);
                    const stats = fs.statSync(filePath);
                    const sizeKB = Math.round(stats.size / 1024);

                    if (sizeKB > 500 && (file.endsWith('.jpg') || file.endsWith('.png') || file.endsWith('.gif'))) {
                        this.results.assets.issues.push(`Large image file: ${dir}/${file} (${sizeKB}KB)`);
                        console.log(`    ⚠️  Large file: ${file} (${sizeKB}KB)`);
                    } else if (sizeKB > 100 && (file.endsWith('.css') || file.endsWith('.js'))) {
                        this.results.assets.issues.push(`Large CSS/JS file: ${dir}/${file} (${sizeKB}KB)`);
                        console.log(`    ⚠️  Large file: ${file} (${sizeKB}KB)`);
                    }
                });
            }
        });

        // Check for SCSS compilation
        const sassPath = path.join(this.sitePath, '_sass');
        const cssPath = path.join(assetsPath, 'css');

        if (fs.existsSync(sassPath) && fs.existsSync(cssPath)) {
            console.log('  ✅ SCSS structure detected');

            // Check for main.scss or similar
            const cssFiles = fs.readdirSync(cssPath);
            const scssFiles = cssFiles.filter(f => f.endsWith('.scss'));

            if (scssFiles.length === 0) {
                this.results.assets.issues.push('SCSS directory found but no .scss files in assets/css/');
                console.log('    ⚠️  No .scss files in assets/css/');
            } else {
                console.log(`    ✅ Found ${scssFiles.length} SCSS files`);
            }
        }
    }

    async testBuildProcess() {
        console.log('\n🔨 Testing Build Process...');

        // Check if Jekyll is available
        try {
            const jekyllVersion = execSync('jekyll --version', { cwd: this.sitePath, stdio: 'pipe', timeout: 10000 });
            console.log(`  ✅ Jekyll available: ${jekyllVersion.toString().trim()}`);
        } catch (error) {
            console.log('  ⚠️  Jekyll not available in PATH');

            // Try with bundle exec
            try {
                const bundleJekyll = execSync('bundle exec jekyll --version', { cwd: this.sitePath, stdio: 'pipe', timeout: 10000 });
                console.log(`  ✅ Jekyll via Bundle: ${bundleJekyll.toString().trim()}`);
            } catch (bundleError) {
                this.results.build.issues.push('Jekyll not available (try: gem install jekyll)');
                console.log('  ❌ Jekyll not available via bundle either');
                return;
            }
        }

        // Attempt a dry-run build
        try {
            console.log('  🔄 Attempting dry-run build...');

            const buildCommand = fs.existsSync(path.join(this.sitePath, 'Gemfile'))
                ? 'bundle exec jekyll build --dry-run --trace'
                : 'jekyll build --dry-run --trace';

            const buildOutput = execSync(buildCommand, {
                cwd: this.sitePath,
                stdio: 'pipe',
                timeout: 30000,
                encoding: 'utf8'
            });

            console.log('  ✅ Dry-run build successful');
            this.results.build.successful = true;

            // Parse build output for warnings
            this.parseBuildOutput(buildOutput);

        } catch (error) {
            this.results.build.successful = false;
            this.results.build.issues.push(`Build failed: ${error.message}`);
            console.log(`  ❌ Build failed: ${error.message}`);

            // Try to extract useful error information
            if (error.stdout) {
                const errorLines = error.stdout.toString().split('\n')
                    .filter(line => line.includes('ERROR') || line.includes('Error'))
                    .slice(0, 3); // Limit to first 3 errors

                errorLines.forEach(line => {
                    console.log(`    ${line.trim()}`);
                });
            }
        }
    }

    parseBuildOutput(output) {
        const lines = output.split('\n');

        lines.forEach(line => {
            if (line.includes('WARN')) {
                this.results.build.issues.push(`Build warning: ${line.trim()}`);
                console.log(`    ⚠️  ${line.trim()}`);
            }

            if (line.includes('Deprecation')) {
                this.results.build.issues.push(`Deprecation: ${line.trim()}`);
                console.log(`    ⚠️  ${line.trim()}`);
            }
        });
    }

    async checkDeploymentConfiguration() {
        console.log('\n🚀 Checking Deployment Configuration...');

        // Check for GitHub repository settings (if possible)
        const configPath = path.join(this.sitePath, '_config.yml');
        if (fs.existsSync(configPath)) {
            const config = fs.readFileSync(configPath, 'utf8');

            // Check URL configuration
            const urlMatch = config.match(/url:\s*["']?([^"'\n]+)["']?/);
            const baseurlMatch = config.match(/baseurl:\s*["']?([^"'\n]+)["']?/);

            if (urlMatch) {
                console.log(`  ✅ Site URL configured: ${urlMatch[1]}`);
            } else {
                this.results.config.issues.push('Site URL not configured');
                console.log('  ⚠️  Site URL not configured');
            }

            if (baseurlMatch) {
                console.log(`  ✅ Base URL configured: ${baseurlMatch[1]}`);
            }
        }

        // Check for proper 404 page
        const notFoundPages = ['404.html', '404.md'];
        let has404 = false;

        notFoundPages.forEach(page => {
            if (fs.existsSync(path.join(this.sitePath, page))) {
                console.log(`  ✅ Custom 404 page: ${page}`);
                has404 = true;
            }
        });

        if (!has404) {
            this.results.config.issues.push('No custom 404 page found');
            console.log('  ⚠️  No custom 404 page');
        }

        // Check for robots.txt
        const robotsPath = path.join(this.sitePath, 'robots.txt');
        if (fs.existsSync(robotsPath)) {
            console.log('  ✅ robots.txt found');
        } else {
            console.log('  ℹ️  No robots.txt (optional)');
        }

        // Check for sitemap generation
        if (fs.existsSync(configPath)) {
            const config = fs.readFileSync(configPath, 'utf8');
            if (config.includes('jekyll-sitemap')) {
                console.log('  ✅ Sitemap plugin configured');
            } else {
                console.log('  ℹ️  No sitemap plugin (consider adding jekyll-sitemap)');
            }
        }
    }

    async performanceChecks() {
        console.log('\n⚡ Performance Checks...');

        // Check for asset optimization
        const assetsPath = path.join(this.sitePath, 'assets');
        if (fs.existsSync(assetsPath)) {
            // Check for minification
            const cssPath = path.join(assetsPath, 'css');
            if (fs.existsSync(cssPath)) {
                const cssFiles = fs.readdirSync(cssPath);
                const minifiedCss = cssFiles.filter(f => f.includes('.min.'));

                if (minifiedCss.length === 0 && cssFiles.length > 0) {
                    this.results.performance.issues.push('No minified CSS files detected');
                    console.log('  ⚠️  No minified CSS detected');
                }
            }

            const jsPath = path.join(assetsPath, 'js');
            if (fs.existsSync(jsPath)) {
                const jsFiles = fs.readdirSync(jsPath);
                const minifiedJs = jsFiles.filter(f => f.includes('.min.'));

                if (minifiedJs.length === 0 && jsFiles.length > 0) {
                    this.results.performance.issues.push('No minified JavaScript files detected');
                    console.log('  ⚠️  No minified JS detected');
                }
            }
        }

        // Check Jekyll configuration for performance
        const configPath = path.join(this.sitePath, '_config.yml');
        if (fs.existsSync(configPath)) {
            const config = fs.readFileSync(configPath, 'utf8');

            // Check for compression
            if (config.includes('compress_html')) {
                console.log('  ✅ HTML compression configured');
            } else {
                this.results.performance.issues.push('HTML compression not configured');
                console.log('  ⚠️  No HTML compression');
            }

            // Check SASS compression
            if (config.includes('style: compressed')) {
                console.log('  ✅ SASS compression configured');
            } else {
                this.results.performance.issues.push('SASS compression not configured');
                console.log('  ⚠️  No SASS compression');
            }
        }
    }

    generateReport() {
        console.log('\n' + '='.repeat(60));
        console.log('📋 BUILD/DEPLOYMENT ISSUES REPORT');
        console.log('='.repeat(60));

        // Configuration Summary
        console.log('\n🔧 Jekyll Configuration:');
        console.log(`  ✅ Configured: ${this.results.jekyll.configured ? 'Yes' : 'No'}`);
        console.log(`  ❌ Issues: ${this.results.jekyll.issues.length}`);

        console.log('\n🐙 GitHub Pages:');
        console.log(`  ✅ Configured: ${this.results.github.configured ? 'Yes' : 'No'}`);
        console.log(`  ❌ Issues: ${this.results.github.issues.length}`);

        console.log('\n🔨 Build Process:');
        console.log(`  ✅ Successful: ${this.results.build.successful ? 'Yes' : 'No'}`);
        console.log(`  ❌ Issues: ${this.results.build.issues.length}`);

        // Dependencies Summary
        console.log('\n📦 Dependencies:');
        console.log(`  ❌ Issues: ${this.results.dependencies.issues.length}`);

        // Assets Summary
        console.log('\n🎨 Assets:');
        console.log(`  ❌ Issues: ${this.results.assets.issues.length}`);

        // Performance Summary
        console.log('\n⚡ Performance:');
        console.log(`  ❌ Issues: ${this.results.performance.issues.length}`);

        // All Issues
        const allIssues = [
            ...this.results.jekyll.issues.map(i => ({ type: 'Jekyll', issue: i })),
            ...this.results.github.issues.map(i => ({ type: 'GitHub', issue: i })),
            ...this.results.build.issues.map(i => ({ type: 'Build', issue: i })),
            ...this.results.dependencies.issues.map(i => ({ type: 'Dependencies', issue: i })),
            ...this.results.assets.issues.map(i => ({ type: 'Assets', issue: i })),
            ...this.results.config.issues.map(i => ({ type: 'Config', issue: i })),
            ...this.results.performance.issues.map(i => ({ type: 'Performance', issue: i }))
        ];

        if (allIssues.length > 0) {
            console.log('\n🔍 ALL ISSUES:');
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

        // Build/Deploy Recommendations
        console.log('\n💡 BUILD/DEPLOYMENT RECOMMENDATIONS:');
        console.log('-'.repeat(60));

        const recommendations = [
            '• Ensure Jekyll and all dependencies are properly installed',
            '• Test builds locally before pushing to repository',
            '• Use GitHub Actions workflow for consistent deployments',
            '• Configure asset compression and minification',
            '• Implement proper error handling and 404 pages',
            '• Monitor build logs for warnings and deprecations',
            '• Keep Jekyll and plugins updated to latest stable versions',
            '• Use bundle exec for consistent gem versions',
            '• Consider using Jekyll environments for different configs'
        ];

        recommendations.forEach(rec => console.log(rec));

        // Save detailed report
        const reportData = {
            timestamp: new Date().toISOString(),
            results: this.results,
            recommendations: recommendations
        };

        fs.writeFileSync(
            '/workspaces/caxton/scripts/validation/build-deployment-report.json',
            JSON.stringify(reportData, null, 2)
        );

        console.log('\n📁 Detailed report saved to: build-deployment-report.json');

        const hasCriticalErrors = !this.results.build.successful ||
                                !this.results.jekyll.configured ||
                                this.results.dependencies.issues.length > 0;

        if (hasCriticalErrors) {
            console.log('\n🚨 CRITICAL ISSUES DETECTED - Build/deployment may fail');
            process.exit(1);
        }
    }
}

// Run if called directly
if (require.main === module) {
    const checker = new BuildDeploymentChecker();
    checker.run();
}

module.exports = BuildDeploymentChecker;
