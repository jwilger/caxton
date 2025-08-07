#!/usr/bin/env node

/**
 * SEO Meta Tags and Elements Validator for Caxton Website
 * Validates meta tags, structured data, alt text, and SEO elements
 */

const fs = require('fs');
const path = require('path');

class SeoMetaValidator {
    constructor() {
        // Determine correct site path - look for website directory from current working directory
        let sitePath = path.join(process.cwd(), 'website');

        // If running from validation directory, go up to project root
        if (process.cwd().includes('scripts/validation')) {
            sitePath = path.join(process.cwd(), '..', '..', 'website');
        }

        this.sitePath = sitePath;
        this.results = {
            meta: { total: 0, passed: 0, failed: 0, issues: [] },
            images: { total: 0, withAlt: 0, withoutAlt: 0, issues: [] },
            headings: { files: 0, issues: [] },
            links: { internal: 0, external: 0, issues: [] },
            structured: { found: 0, issues: [] },
            performance: { issues: [] }
        };
    }

    async run() {
        console.log('🔍 Starting SEO Meta Tags Validation...');
        console.log(`Site Path: ${this.sitePath}`);
        console.log('─'.repeat(60));

        try {
            await this.validateMetaTags();
            await this.validateImageAltText();
            await this.validateHeadingStructure();
            await this.validateLinks();
            await this.validateStructuredData();
            await this.validatePerformanceElements();
            this.generateReport();
        } catch (error) {
            console.error('❌ Error during SEO validation:', error.message);
            process.exit(1);
        }
    }

    async validateMetaTags() {
        console.log('\n🏷️  Validating Meta Tags...');

        const htmlFiles = this.findFiles(['.html']);
        const mdFiles = this.findFiles(['.md', '.markdown']);
        const allFiles = [...htmlFiles, ...mdFiles];

        this.results.meta.total = allFiles.length;

        for (const file of allFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Checking meta tags: ${relativePath}`);

            try {
                const issues = await this.validateFileMetaTags(file);

                if (issues.length === 0) {
                    this.results.meta.passed++;
                    console.log(`  ✅ Meta tags valid`);
                } else {
                    this.results.meta.failed++;
                    console.log(`  ❌ ${issues.length} meta tag issues`);

                    this.results.meta.issues.push({
                        file: relativePath,
                        issues: issues
                    });
                }

            } catch (error) {
                this.results.meta.failed++;
                console.log(`  ❌ Meta validation failed: ${error.message}`);
            }
        }
    }

    async validateFileMetaTags(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const issues = [];
        const isMarkdown = filePath.endsWith('.md') || filePath.endsWith('.markdown');

        if (isMarkdown) {
            // Check Jekyll front matter
            const frontMatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
            if (frontMatterMatch) {
                const frontMatter = frontMatterMatch[1];
                this.validateFrontMatter(frontMatter, issues);
            } else {
                issues.push('Markdown file missing Jekyll front matter');
            }
        } else {
            // Check HTML meta tags
            this.validateHtmlMetaTags(content, issues);
        }

        return issues;
    }

    validateFrontMatter(frontMatter, issues) {
        // Essential front matter fields
        const requiredFields = ['title', 'description'];
        const recommendedFields = ['layout', 'permalink'];

        requiredFields.forEach(field => {
            if (!frontMatter.includes(`${field}:`)) {
                issues.push(`Missing required front matter field: ${field}`);
            }
        });

        // Check title length
        const titleMatch = frontMatter.match(/title:\s*["']?([^"'\n]+)["']?/);
        if (titleMatch) {
            const title = titleMatch[1].trim();
            if (title.length < 10) {
                issues.push(`Title too short: ${title.length} characters (recommend 10-60)`);
            } else if (title.length > 60) {
                issues.push(`Title too long: ${title.length} characters (recommend 10-60)`);
            }
        }

        // Check description length
        const descMatch = frontMatter.match(/description:\s*["']?([^"'\n]+)["']?/);
        if (descMatch) {
            const desc = descMatch[1].trim();
            if (desc.length < 50) {
                issues.push(`Description too short: ${desc.length} characters (recommend 50-160)`);
            } else if (desc.length > 160) {
                issues.push(`Description too long: ${desc.length} characters (recommend 50-160)`);
            }
        }

        // Check for SEO-specific fields
        const seoFields = ['keywords', 'author', 'canonical_url', 'robots'];
        seoFields.forEach(field => {
            if (frontMatter.includes(`${field}:`)) {
                // Field present, could validate content
            }
        });
    }

    validateHtmlMetaTags(content, issues) {
        // Check for essential meta tags
        const essentialTags = [
            { pattern: /<meta[^>]*charset/i, name: 'charset' },
            { pattern: /<meta[^>]*name=["']viewport["']/i, name: 'viewport' },
            { pattern: /<title>/i, name: 'title' },
            { pattern: /<meta[^>]*name=["']description["']/i, name: 'description' }
        ];

        essentialTags.forEach(tag => {
            if (!content.match(tag.pattern)) {
                issues.push(`Missing essential meta tag: ${tag.name}`);
            }
        });

        // Validate title tag
        const titleMatch = content.match(/<title>(.*?)<\/title>/i);
        if (titleMatch) {
            const title = titleMatch[1].trim();
            if (title.length < 10) {
                issues.push(`Title too short: ${title.length} characters`);
            } else if (title.length > 60) {
                issues.push(`Title too long: ${title.length} characters`);
            }
        }

        // Validate description meta tag
        const descMatch = content.match(/<meta[^>]*name=["']description["'][^>]*content=["']([^"']*)["']/i);
        if (descMatch) {
            const desc = descMatch[1].trim();
            if (desc.length < 50) {
                issues.push(`Description too short: ${desc.length} characters`);
            } else if (desc.length > 160) {
                issues.push(`Description too long: ${desc.length} characters`);
            }
        }

        // Check Open Graph tags
        this.validateOpenGraphTags(content, issues);

        // Check Twitter Card tags
        this.validateTwitterCardTags(content, issues);

        // Check other SEO tags
        this.validateOtherSeoTags(content, issues);
    }

    validateOpenGraphTags(content, issues) {
        const ogTags = [
            { pattern: /<meta[^>]*property=["']og:title["']/i, name: 'og:title' },
            { pattern: /<meta[^>]*property=["']og:description["']/i, name: 'og:description' },
            { pattern: /<meta[^>]*property=["']og:url["']/i, name: 'og:url' },
            { pattern: /<meta[^>]*property=["']og:type["']/i, name: 'og:type' }
        ];

        let ogTagsFound = 0;
        ogTags.forEach(tag => {
            if (content.match(tag.pattern)) {
                ogTagsFound++;
            }
        });

        if (ogTagsFound > 0 && ogTagsFound < 4) {
            issues.push(`Incomplete Open Graph tags: ${ogTagsFound}/4 found`);
        } else if (ogTagsFound === 4) {
            // All OG tags found, validate image
            if (!content.match(/<meta[^>]*property=["']og:image["']/i)) {
                issues.push('Open Graph tags found but missing og:image');
            }
        }
    }

    validateTwitterCardTags(content, issues) {
        const twitterTags = [
            { pattern: /<meta[^>]*name=["']twitter:card["']/i, name: 'twitter:card' },
            { pattern: /<meta[^>]*name=["']twitter:title["']/i, name: 'twitter:title' },
            { pattern: /<meta[^>]*name=["']twitter:description["']/i, name: 'twitter:description' }
        ];

        let twitterTagsFound = 0;
        twitterTags.forEach(tag => {
            if (content.match(tag.pattern)) {
                twitterTagsFound++;
            }
        });

        if (twitterTagsFound > 0 && twitterTagsFound < 3) {
            issues.push(`Incomplete Twitter Card tags: ${twitterTagsFound}/3 found`);
        }
    }

    validateOtherSeoTags(content, issues) {
        // Check for canonical URL
        if (!content.match(/<link[^>]*rel=["']canonical["']/i)) {
            issues.push('Missing canonical URL');
        }

        // Check for robots meta tag
        if (content.match(/<meta[^>]*name=["']robots["'][^>]*content=["'][^"']*noindex[^"']*["']/i)) {
            issues.push('Page set to noindex - verify this is intentional');
        }

        // Check for duplicate meta tags
        const metaDescriptions = content.match(/<meta[^>]*name=["']description["']/gi);
        if (metaDescriptions && metaDescriptions.length > 1) {
            issues.push('Multiple description meta tags found');
        }
    }

    async validateImageAltText() {
        console.log('\n🖼️  Validating Image Alt Text...');

        const htmlFiles = this.findFiles(['.html']);
        const mdFiles = this.findFiles(['.md', '.markdown']);
        const allFiles = [...htmlFiles, ...mdFiles];

        for (const file of allFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Checking images: ${relativePath}`);

            const content = fs.readFileSync(file, 'utf8');
            const images = this.extractImages(content);

            this.results.images.total += images.length;

            images.forEach(image => {
                if (image.hasAlt) {
                    this.results.images.withAlt++;
                    if (image.altText.trim() === '') {
                        // Empty alt is okay for decorative images
                        if (!image.isDecorative) {
                            this.results.images.issues.push({
                                file: relativePath,
                                issue: 'Empty alt text (consider adding descriptive text)',
                                image: image.src
                            });
                        }
                    }
                } else {
                    this.results.images.withoutAlt++;
                    this.results.images.issues.push({
                        file: relativePath,
                        issue: 'Missing alt attribute',
                        image: image.src
                    });
                }
            });

            if (images.length > 0) {
                console.log(`  Found ${images.length} images, ${images.filter(i => i.hasAlt).length} with alt text`);
            }
        }
    }

    extractImages(content) {
        const images = [];

        // HTML images
        const htmlImages = content.match(/<img[^>]*>/gi) || [];
        htmlImages.forEach(imgTag => {
            const srcMatch = imgTag.match(/src=["']([^"']*)["']/i);
            const altMatch = imgTag.match(/alt=["']([^"']*)["']/i);
            const roleMatch = imgTag.match(/role=["']presentation["']/i);
            const ariaHidden = imgTag.match(/aria-hidden=["']true["']/i);

            if (srcMatch) {
                images.push({
                    src: srcMatch[1],
                    hasAlt: !!altMatch,
                    altText: altMatch ? altMatch[1] : '',
                    isDecorative: !!roleMatch || !!ariaHidden,
                    type: 'html'
                });
            }
        });

        // Markdown images
        const markdownImages = content.match(/!\[([^\]]*)\]\(([^)]+)\)/g) || [];
        markdownImages.forEach(mdImg => {
            const match = mdImg.match(/!\[([^\]]*)\]\(([^)]+)\)/);
            if (match) {
                images.push({
                    src: match[2],
                    hasAlt: true,
                    altText: match[1],
                    isDecorative: false,
                    type: 'markdown'
                });
            }
        });

        return images;
    }

    async validateHeadingStructure() {
        console.log('\n📑 Validating Heading Structure...');

        const htmlFiles = this.findFiles(['.html']);
        const mdFiles = this.findFiles(['.md', '.markdown']);
        const allFiles = [...htmlFiles, ...mdFiles];

        this.results.headings.files = allFiles.length;

        for (const file of allFiles) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Checking headings: ${relativePath}`);

            const content = fs.readFileSync(file, 'utf8');
            const issues = this.validateHeadingHierarchy(content);

            if (issues.length > 0) {
                this.results.headings.issues.push({
                    file: relativePath,
                    issues: issues
                });
                console.log(`  ❌ ${issues.length} heading issues`);
            } else {
                console.log(`  ✅ Heading structure valid`);
            }
        }
    }

    validateHeadingHierarchy(content) {
        const issues = [];
        let headings = [];

        // Extract HTML headings
        const htmlHeadings = content.match(/<h([1-6])[^>]*>(.*?)<\/h[1-6]>/gi) || [];
        htmlHeadings.forEach(heading => {
            const match = heading.match(/<h([1-6])[^>]*>(.*?)<\/h[1-6]>/i);
            if (match) {
                headings.push({
                    level: parseInt(match[1]),
                    text: match[2].replace(/<[^>]*>/g, '').trim(),
                    type: 'html'
                });
            }
        });

        // Extract Markdown headings
        const mdHeadings = content.match(/^#{1,6}\s+.+$/gm) || [];
        mdHeadings.forEach(heading => {
            const match = heading.match(/^(#{1,6})\s+(.+)$/);
            if (match) {
                headings.push({
                    level: match[1].length,
                    text: match[2].trim(),
                    type: 'markdown'
                });
            }
        });

        if (headings.length === 0) {
            issues.push('No headings found');
            return issues;
        }

        // Check for H1
        const h1Count = headings.filter(h => h.level === 1).length;
        if (h1Count === 0) {
            issues.push('No H1 heading found');
        } else if (h1Count > 1) {
            issues.push(`Multiple H1 headings found: ${h1Count}`);
        }

        // Check hierarchy
        let previousLevel = 0;
        headings.forEach((heading, index) => {
            if (heading.level > previousLevel + 1) {
                issues.push(`Heading hierarchy skip: H${previousLevel} to H${heading.level} ("${heading.text.substring(0, 30)}...")`);
            }
            previousLevel = heading.level;
        });

        // Check for empty headings
        headings.forEach(heading => {
            if (heading.text.length === 0) {
                issues.push('Empty heading found');
            }
        });

        return issues;
    }

    async validateLinks() {
        console.log('\n🔗 Validating Links...');

        const htmlFiles = this.findFiles(['.html']);
        const mdFiles = this.findFiles(['.md', '.markdown']);
        const allFiles = [...htmlFiles, ...mdFiles];

        for (const file of allFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            const links = this.extractLinks(content);

            links.forEach(link => {
                if (link.isExternal) {
                    this.results.links.external++;

                    // Check if external links have proper attributes
                    if (!link.hasTargetBlank) {
                        this.results.links.issues.push({
                            file: relativePath,
                            issue: 'External link missing target="_blank"',
                            url: link.href
                        });
                    }

                    if (!link.hasNoOpener) {
                        this.results.links.issues.push({
                            file: relativePath,
                            issue: 'External link missing rel="noopener"',
                            url: link.href
                        });
                    }
                } else {
                    this.results.links.internal++;
                }

                // Check for meaningful link text
                if (link.text && (
                    link.text.toLowerCase() === 'click here' ||
                    link.text.toLowerCase() === 'read more' ||
                    link.text.toLowerCase() === 'here'
                )) {
                    this.results.links.issues.push({
                        file: relativePath,
                        issue: 'Non-descriptive link text',
                        url: link.href,
                        text: link.text
                    });
                }
            });
        }

        console.log(`Found ${this.results.links.internal} internal and ${this.results.links.external} external links`);
    }

    extractLinks(content) {
        const links = [];

        // HTML links
        const htmlLinks = content.match(/<a[^>]*href=["']([^"']*)["'][^>]*>(.*?)<\/a>/gi) || [];
        htmlLinks.forEach(linkTag => {
            const hrefMatch = linkTag.match(/href=["']([^"']*)["']/i);
            const textMatch = linkTag.match(/>([^<]*)</);
            const targetMatch = linkTag.match(/target=["']_blank["']/i);
            const relMatch = linkTag.match(/rel=["']([^"']*)["']/i);

            if (hrefMatch) {
                const href = hrefMatch[1];
                const isExternal = href.startsWith('http://') || href.startsWith('https://');

                links.push({
                    href: href,
                    text: textMatch ? textMatch[1].trim() : '',
                    isExternal: isExternal,
                    hasTargetBlank: !!targetMatch,
                    hasNoOpener: relMatch ? relMatch[1].includes('noopener') : false,
                    type: 'html'
                });
            }
        });

        // Markdown links
        const markdownLinks = content.match(/\[([^\]]*)\]\(([^)]+)\)/g) || [];
        markdownLinks.forEach(mdLink => {
            const match = mdLink.match(/\[([^\]]*)\]\(([^)]+)\)/);
            if (match) {
                const href = match[2];
                const isExternal = href.startsWith('http://') || href.startsWith('https://');

                links.push({
                    href: href,
                    text: match[1],
                    isExternal: isExternal,
                    hasTargetBlank: false, // Markdown doesn't have these attributes
                    hasNoOpener: false,
                    type: 'markdown'
                });
            }
        });

        return links;
    }

    async validateStructuredData() {
        console.log('\n🏗️  Validating Structured Data...');

        const htmlFiles = this.findFiles(['.html']);

        for (const file of htmlFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            // Check for JSON-LD
            const jsonLdMatches = content.match(/<script[^>]*type=["']application\/ld\+json["'][^>]*>(.*?)<\/script>/gis) || [];
            jsonLdMatches.forEach(match => {
                this.results.structured.found++;
                const jsonMatch = match.match(/<script[^>]*type=["']application\/ld\+json["'][^>]*>(.*?)<\/script>/is);
                if (jsonMatch) {
                    try {
                        JSON.parse(jsonMatch[1]);
                    } catch (error) {
                        this.results.structured.issues.push({
                            file: relativePath,
                            issue: `Invalid JSON-LD: ${error.message}`
                        });
                    }
                }
            });

            // Check for microdata
            const microdataElements = content.match(/<[^>]*itemscope[^>]*>/gi) || [];
            if (microdataElements.length > 0) {
                this.results.structured.found += microdataElements.length;
            }

            // Check for RDFa
            const rdfaElements = content.match(/<[^>]*property=["'][^"']*["'][^>]*>/gi) || [];
            if (rdfaElements.length > 0) {
                this.results.structured.found += rdfaElements.length;
            }
        }

        console.log(`Found ${this.results.structured.found} structured data elements`);
    }

    async validatePerformanceElements() {
        console.log('\n⚡ Validating Performance Elements...');

        const htmlFiles = this.findFiles(['.html']);

        for (const file of htmlFiles) {
            const relativePath = path.relative(this.sitePath, file);
            const content = fs.readFileSync(file, 'utf8');

            // Check for lazy loading
            const images = content.match(/<img[^>]*>/gi) || [];
            images.forEach(img => {
                if (!img.includes('loading=') && !img.includes('lazy')) {
                    this.results.performance.issues.push({
                        file: relativePath,
                        issue: 'Image without lazy loading attribute',
                        element: img.substring(0, 50) + '...'
                    });
                }
            });

            // Check for preconnect to external resources
            const externalFonts = content.match(/fonts\.googleapis\.com|fonts\.gstatic\.com/g);
            if (externalFonts && !content.includes('preconnect')) {
                this.results.performance.issues.push({
                    file: relativePath,
                    issue: 'External fonts without preconnect'
                });
            }

            // Check for multiple CSS files (should be minified/combined)
            const cssLinks = content.match(/<link[^>]*rel=["']stylesheet["'][^>]*>/gi) || [];
            if (cssLinks.length > 5) {
                this.results.performance.issues.push({
                    file: relativePath,
                    issue: `Many CSS files (${cssLinks.length}) - consider combining`,
                    count: cssLinks.length
                });
            }
        }
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
        console.log('📋 SEO META TAGS VALIDATION REPORT');
        console.log('='.repeat(60));

        // Meta Tags Summary
        console.log('\n🏷️  Meta Tags:');
        console.log(`  Total files: ${this.results.meta.total}`);
        console.log(`  ✅ Passed: ${this.results.meta.passed}`);
        console.log(`  ❌ Failed: ${this.results.meta.failed}`);

        if (this.results.meta.total > 0) {
            const passRate = ((this.results.meta.passed / this.results.meta.total) * 100).toFixed(1);
            console.log(`  📈 Pass rate: ${passRate}%`);
        }

        // Images Summary
        console.log('\n🖼️  Images:');
        console.log(`  Total images: ${this.results.images.total}`);
        console.log(`  ✅ With alt text: ${this.results.images.withAlt}`);
        console.log(`  ❌ Missing alt text: ${this.results.images.withoutAlt}`);

        if (this.results.images.total > 0) {
            const altRate = ((this.results.images.withAlt / this.results.images.total) * 100).toFixed(1);
            console.log(`  📈 Alt text coverage: ${altRate}%`);
        }

        // Links Summary
        console.log('\n🔗 Links:');
        console.log(`  Internal links: ${this.results.links.internal}`);
        console.log(`  External links: ${this.results.links.external}`);
        console.log(`  Link issues: ${this.results.links.issues.length}`);

        // Structured Data Summary
        console.log('\n🏗️  Structured Data:');
        console.log(`  Elements found: ${this.results.structured.found}`);
        console.log(`  Issues: ${this.results.structured.issues.length}`);

        // Performance Summary
        console.log('\n⚡ Performance:');
        console.log(`  Issues found: ${this.results.performance.issues.length}`);

        // Detailed Issues
        const allIssues = [
            ...this.results.meta.issues,
            ...this.results.images.issues.map(i => ({file: i.file, issues: [i.issue]})),
            ...this.results.headings.issues,
            ...this.results.links.issues.map(i => ({file: i.file, issues: [i.issue]})),
            ...this.results.structured.issues.map(i => ({file: i.file, issues: [i.issue]})),
            ...this.results.performance.issues.map(i => ({file: i.file, issues: [i.issue]}))
        ];

        if (allIssues.length > 0) {
            console.log('\n🔍 DETAILED ISSUES:');
            console.log('-'.repeat(60));

            const groupedIssues = {};
            allIssues.forEach(item => {
                if (!groupedIssues[item.file]) {
                    groupedIssues[item.file] = [];
                }
                if (Array.isArray(item.issues)) {
                    groupedIssues[item.file].push(...item.issues);
                } else {
                    groupedIssues[item.file].push(item.issues);
                }
            });

            Object.keys(groupedIssues).forEach(file => {
                console.log(`\n📄 ${file}:`);
                groupedIssues[file].forEach(issue => {
                    console.log(`  ❌ ${issue}`);
                });
            });
        }

        // SEO Recommendations
        console.log('\n💡 SEO RECOMMENDATIONS:');
        console.log('-'.repeat(60));

        const recommendations = [];

        if (this.results.meta.failed > 0) {
            recommendations.push('• Ensure all pages have title and description meta tags');
        }

        if (this.results.images.withoutAlt > 0) {
            recommendations.push('• Add alt text to all images for accessibility and SEO');
        }

        if (this.results.links.issues.length > 0) {
            recommendations.push('• Use descriptive link text and proper attributes for external links');
        }

        if (this.results.structured.found === 0) {
            recommendations.push('• Consider adding structured data (JSON-LD) for better search results');
        }

        recommendations.push('• Optimize page loading speed with lazy loading and resource preconnection');
        recommendations.push('• Use proper heading hierarchy (H1-H6) for content structure');
        recommendations.push('• Implement canonical URLs to prevent duplicate content issues');

        recommendations.forEach(rec => console.log(rec));

        // Save detailed report
        const reportData = {
            timestamp: new Date().toISOString(),
            summary: {
                metaTags: this.results.meta,
                images: this.results.images,
                headings: this.results.headings,
                links: this.results.links,
                structuredData: this.results.structured,
                performance: this.results.performance
            },
            recommendations: recommendations
        };

        fs.writeFileSync(
            '/workspaces/caxton/scripts/validation/seo-meta-report.json',
            JSON.stringify(reportData, null, 2)
        );

        console.log('\n📁 Detailed report saved to: seo-meta-report.json');

        const hasErrors = this.results.meta.failed > 0 ||
                         this.results.images.withoutAlt > 0 ||
                         this.results.links.issues.length > 0 ||
                         this.results.structured.issues.length > 0;

        if (hasErrors) {
            process.exit(1);
        }
    }
}

// Run if called directly
if (require.main === module) {
    const validator = new SeoMetaValidator();
    validator.run();
}

module.exports = SeoMetaValidator;
