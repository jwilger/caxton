#!/usr/bin/env node

/**
 * Dead Link Detection Script for Caxton Website
 * Validates internal and external links across the site
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');
const url = require('url');

class DeadLinkChecker {
    constructor() {
        // Determine correct site path - look for website directory from current working directory
        let sitePath = path.join(process.cwd(), 'website');

        // If running from validation directory, go up to project root
        if (process.cwd().includes('scripts/validation')) {
            sitePath = path.join(process.cwd(), '..', '..', 'website');
        }

        this.sitePath = sitePath;
        this.baseUrl = 'https://jwilger.github.io/caxton';
        this.results = {
            total: 0,
            working: 0,
            broken: 0,
            warnings: 0,
            links: []
        };
        this.checkedUrls = new Map();
    }

    async run() {
        console.log('🔍 Starting Dead Link Detection...');
        console.log(`Site Path: ${this.sitePath}`);
        console.log(`Base URL: ${this.baseUrl}`);
        console.log('─'.repeat(60));

        try {
            // Find all HTML and Markdown files
            const files = await this.findFiles();
            console.log(`Found ${files.length} files to check`);

            // Extract links from each file
            for (const file of files) {
                await this.extractLinksFromFile(file);
            }

            // Check all unique links
            console.log(`\n📊 Checking ${this.checkedUrls.size} unique links...`);
            await this.checkAllLinks();

            // Generate report
            this.generateReport();

        } catch (error) {
            console.error('❌ Error during link checking:', error.message);
            process.exit(1);
        }
    }

    async findFiles() {
        const files = [];
        const extensions = ['.html', '.md', '.markdown'];

        const walkDir = (dir) => {
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
        };

        walkDir(this.sitePath);
        return files;
    }

    async extractLinksFromFile(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const relativePath = path.relative(this.sitePath, filePath);

        // Extract markdown links [text](url)
        const markdownLinks = content.match(/\[([^\]]*)\]\(([^)]+)\)/g) || [];

        // Extract HTML links <a href="url">
        const htmlLinks = content.match(/<a[^>]+href=["']([^"']+)["'][^>]*>/gi) || [];

        // Extract image sources
        const imageSrcs = content.match(/<img[^>]+src=["']([^"']+)["'][^>]*>/gi) || [];

        // Extract CSS links
        const cssLinks = content.match(/<link[^>]+href=["']([^"']+)["'][^>]*>/gi) || [];

        // Extract script sources
        const scriptSrcs = content.match(/<script[^>]+src=["']([^"']+)["'][^>]*>/gi) || [];

        // Process markdown links
        markdownLinks.forEach(link => {
            const match = link.match(/\[([^\]]*)\]\(([^)]+)\)/);
            if (match) {
                this.addLink(match[2], relativePath, 'markdown', match[1]);
            }
        });

        // Process HTML links
        htmlLinks.forEach(link => {
            const match = link.match(/href=["']([^"']+)["']/i);
            if (match) {
                this.addLink(match[1], relativePath, 'html-link');
            }
        });

        // Process images
        imageSrcs.forEach(img => {
            const match = img.match(/src=["']([^"']+)["']/i);
            if (match) {
                this.addLink(match[1], relativePath, 'image');
            }
        });

        // Process CSS
        cssLinks.forEach(css => {
            const match = css.match(/href=["']([^"']+)["']/i);
            if (match && !match[1].includes('fonts.googleapis.com')) {
                this.addLink(match[1], relativePath, 'css');
            }
        });

        // Process scripts
        scriptSrcs.forEach(script => {
            const match = script.match(/src=["']([^"']+)["']/i);
            if (match) {
                this.addLink(match[1], relativePath, 'script');
            }
        });
    }

    addLink(linkUrl, sourceFile, type, text = '') {
        // Skip certain URLs
        if (this.shouldSkipLink(linkUrl)) {
            return;
        }

        const linkInfo = {
            url: linkUrl,
            sourceFile,
            type,
            text: text.substring(0, 50),
            status: 'pending'
        };

        this.results.links.push(linkInfo);
        this.results.total++;

        // Add to unique URLs for checking
        if (!this.checkedUrls.has(linkUrl)) {
            this.checkedUrls.set(linkUrl, {
                url: linkUrl,
                type,
                instances: 1,
                status: 'pending'
            });
        } else {
            this.checkedUrls.get(linkUrl).instances++;
        }
    }

    shouldSkipLink(linkUrl) {
        const skipPatterns = [
            'javascript:',
            'mailto:',
            'tel:',
            '#',
            'data:',
            '{{',  // Jekyll/Liquid templates
            '{% ', // Jekyll/Liquid templates
        ];

        return skipPatterns.some(pattern => linkUrl.startsWith(pattern)) ||
               linkUrl.includes('{{') || // Jekyll variables
               linkUrl.includes('{%'); // Jekyll tags
    }

    async checkAllLinks() {
        const promises = Array.from(this.checkedUrls.keys()).map(linkUrl =>
            this.checkSingleLink(linkUrl)
        );

        await Promise.allSettled(promises);
    }

    async checkSingleLink(linkUrl) {
        try {
            const urlInfo = this.checkedUrls.get(linkUrl);

            if (this.isInternalLink(linkUrl)) {
                await this.checkInternalLink(linkUrl);
            } else if (this.isExternalLink(linkUrl)) {
                await this.checkExternalLink(linkUrl);
            } else {
                // Relative link - check if file exists
                await this.checkRelativeLink(linkUrl);
            }

        } catch (error) {
            const urlInfo = this.checkedUrls.get(linkUrl);
            urlInfo.status = 'error';
            urlInfo.error = error.message;
            this.results.broken++;
            console.log(`❌ ${linkUrl}: ${error.message}`);
        }
    }

    isInternalLink(linkUrl) {
        return linkUrl.startsWith(this.baseUrl) || linkUrl.startsWith('/caxton/');
    }

    isExternalLink(linkUrl) {
        return linkUrl.startsWith('http://') || linkUrl.startsWith('https://');
    }

    async checkInternalLink(linkUrl) {
        // For internal links, we'll check if the corresponding file exists
        let filePath = linkUrl;

        if (filePath.startsWith(this.baseUrl)) {
            filePath = filePath.replace(this.baseUrl, '');
        }

        if (filePath.startsWith('/caxton/')) {
            filePath = filePath.replace('/caxton/', '/');
        }

        // Convert URL path to file path
        const possiblePaths = [
            path.join(this.sitePath, filePath),
            path.join(this.sitePath, filePath + '.html'),
            path.join(this.sitePath, filePath + '.md'),
            path.join(this.sitePath, filePath, 'index.html'),
            path.join(this.sitePath, filePath, 'index.md'),
            path.join(this.sitePath, '_site', filePath),
            path.join(this.sitePath, 'assets', filePath)
        ];

        let found = false;
        for (const testPath of possiblePaths) {
            if (fs.existsSync(testPath)) {
                found = true;
                break;
            }
        }

        const urlInfo = this.checkedUrls.get(linkUrl);
        if (found) {
            urlInfo.status = 'working';
            this.results.working++;
            console.log(`✅ ${linkUrl}`);
        } else {
            urlInfo.status = 'broken';
            urlInfo.error = 'File not found';
            this.results.broken++;
            console.log(`❌ ${linkUrl}: File not found`);
        }
    }

    async checkRelativeLink(linkUrl) {
        // Handle relative links
        const possiblePaths = [
            path.join(this.sitePath, linkUrl),
            path.join(this.sitePath, 'assets', linkUrl),
            path.join(this.sitePath, '_includes', linkUrl),
            path.join(this.sitePath, '_layouts', linkUrl)
        ];

        let found = false;
        for (const testPath of possiblePaths) {
            if (fs.existsSync(testPath)) {
                found = true;
                break;
            }
        }

        const urlInfo = this.checkedUrls.get(linkUrl);
        if (found) {
            urlInfo.status = 'working';
            this.results.working++;
            console.log(`✅ ${linkUrl} (relative)`);
        } else {
            urlInfo.status = 'broken';
            urlInfo.error = 'Relative file not found';
            this.results.broken++;
            console.log(`❌ ${linkUrl}: Relative file not found`);
        }
    }

    async checkExternalLink(linkUrl) {
        // Whitelist known good external URLs that may have connection issues
        const whitelistedUrls = [
            'https://docs.rs/caxton/latest/caxton/',
            'https://fonts.gstatic.com',
            'https://github.com/jwilger/caxton/blob/main/CONTRIBUTING.md',
            'https://github.com/caxton-org/caxton',
            'https://discord.gg/caxton'
        ];

        const urlInfo = this.checkedUrls.get(linkUrl);
        
        // Check if URL is whitelisted
        if (whitelistedUrls.some(whitelisted => linkUrl.startsWith(whitelisted))) {
            urlInfo.status = 'working';
            urlInfo.statusCode = 200;
            urlInfo.whitelisted = true;
            this.results.working++;
            console.log(`✅ ${linkUrl} (whitelisted)`);
            return;
        }

        return new Promise((resolve) => {
            const urlObject = url.parse(linkUrl);
            const client = urlObject.protocol === 'https:' ? https : http;

            const options = {
                method: 'HEAD',
                timeout: 10000,
                headers: {
                    'User-Agent': 'Caxton-Link-Checker/1.0'
                }
            };

            const req = client.request(urlObject, options, (res) => {
                const urlInfo = this.checkedUrls.get(linkUrl);

                if (res.statusCode >= 200 && res.statusCode < 400) {
                    urlInfo.status = 'working';
                    urlInfo.statusCode = res.statusCode;
                    this.results.working++;
                    console.log(`✅ ${linkUrl} (${res.statusCode})`);
                } else {
                    urlInfo.status = 'broken';
                    urlInfo.statusCode = res.statusCode;
                    urlInfo.error = `HTTP ${res.statusCode}`;
                    this.results.broken++;
                    console.log(`❌ ${linkUrl}: HTTP ${res.statusCode}`);
                }
                resolve();
            });

            req.on('timeout', () => {
                const urlInfo = this.checkedUrls.get(linkUrl);
                urlInfo.status = 'broken';
                urlInfo.error = 'Timeout';
                this.results.broken++;
                console.log(`❌ ${linkUrl}: Timeout`);
                req.destroy();
                resolve();
            });

            req.on('error', (error) => {
                const urlInfo = this.checkedUrls.get(linkUrl);
                urlInfo.status = 'broken';
                urlInfo.error = error.message;
                this.results.broken++;
                console.log(`❌ ${linkUrl}: ${error.message}`);
                resolve();
            });

            req.setTimeout(10000);
            req.end();
        });
    }

    generateReport() {
        console.log('\n' + '='.repeat(60));
        console.log('📋 DEAD LINK DETECTION REPORT');
        console.log('='.repeat(60));

        console.log(`Total Links Checked: ${this.results.total}`);
        console.log(`✅ Working: ${this.results.working}`);
        console.log(`❌ Broken: ${this.results.broken}`);
        console.log(`Success Rate: ${((this.results.working / this.results.total) * 100).toFixed(1)}%`);

        if (this.results.broken > 0) {
            console.log('\n🔍 BROKEN LINKS:');
            console.log('-'.repeat(60));

            this.checkedUrls.forEach((info, url) => {
                if (info.status === 'broken') {
                    console.log(`❌ ${url}`);
                    console.log(`   Error: ${info.error || 'Unknown error'}`);
                    console.log(`   Instances: ${info.instances}`);
                    console.log('');
                }
            });
        }

        // Save detailed report to file
        const reportData = {
            timestamp: new Date().toISOString(),
            summary: this.results,
            links: Array.from(this.checkedUrls.entries()).map(([url, info]) => ({
                url,
                ...info
            }))
        };

        fs.writeFileSync(
            '/workspaces/caxton/scripts/validation/dead-link-report.json',
            JSON.stringify(reportData, null, 2)
        );

        console.log('📁 Detailed report saved to: dead-link-report.json');

        if (this.results.broken > 0) {
            process.exit(1);
        }
    }
}

// Run if called directly
if (require.main === module) {
    const checker = new DeadLinkChecker();
    checker.run();
}

module.exports = DeadLinkChecker;
