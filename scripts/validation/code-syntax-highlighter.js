#!/usr/bin/env node

/**
 * Code Block Syntax Highlighting Validator for Caxton Website
 * Validates syntax highlighting, code block structure, and language detection
 */

const fs = require('fs');
const path = require('path');

class CodeSyntaxHighlighter {
    constructor() {
        // Determine correct site path - look for website directory from current working directory
        let sitePath = path.join(process.cwd(), 'website');

        // If running from validation directory, go up to project root
        if (process.cwd().includes('scripts/validation')) {
            sitePath = path.join(process.cwd(), '..', '..', 'website');
        }

        this.sitePath = sitePath;
        this.results = {
            codeBlocks: { total: 0, highlighted: 0, unhighlighted: 0, invalid: 0 },
            languages: {},
            issues: [],
            files: []
        };

        // Supported languages by Rouge (Jekyll's default highlighter)
        this.supportedLanguages = [
            'bash', 'sh', 'shell',
            'rust', 'rs',
            'javascript', 'js',
            'typescript', 'ts',
            'html', 'xml',
            'css', 'scss', 'sass',
            'json',
            'yaml', 'yml',
            'markdown', 'md',
            'python', 'py',
            'java',
            'c', 'cpp', 'c++',
            'go',
            'ruby', 'rb',
            'php',
            'sql',
            'dockerfile',
            'toml',
            'ini',
            'plaintext', 'text'
        ];
    }

    async run() {
        console.log('🎨 Starting Code Syntax Highlighting Validation...');
        console.log(`Site Path: ${this.sitePath}`);
        console.log('─'.repeat(60));

        try {
            await this.findAndAnalyzeCodeBlocks();
            await this.validateSyntaxHighlighting();
            await this.checkHighlighterConfiguration();
            this.generateReport();
        } catch (error) {
            console.error('❌ Error during code highlighting validation:', error.message);
            process.exit(1);
        }
    }

    async findAndAnalyzeCodeBlocks() {
        console.log('\n🔍 Finding and Analyzing Code Blocks...');

        const files = this.findFiles(['.html', '.md', '.markdown']);
        this.results.files = files.map(f => path.relative(this.sitePath, f));

        for (const file of files) {
            const relativePath = path.relative(this.sitePath, file);
            console.log(`Analyzing: ${relativePath}`);

            const codeBlocks = await this.extractCodeBlocks(file);

            if (codeBlocks.length > 0) {
                console.log(`  Found ${codeBlocks.length} code blocks`);
                this.analyzeCodeBlocks(codeBlocks, relativePath);
            } else {
                console.log(`  No code blocks found`);
            }
        }
    }

    async extractCodeBlocks(filePath) {
        const content = fs.readFileSync(filePath, 'utf8');
        const codeBlocks = [];

        // Extract Markdown code blocks (fenced with ```)
        const markdownBlocks = content.match(/```(\w+)?\n([\s\S]*?)```/g) || [];
        markdownBlocks.forEach((block, index) => {
            const match = block.match(/```(\w+)?\n([\s\S]*?)```/);
            const language = match[1] || 'unknown';
            const code = match[2];

            codeBlocks.push({
                type: 'markdown',
                language: language,
                code: code,
                raw: block,
                index: index + 1
            });
        });

        // Extract HTML code blocks (<pre><code>)
        const htmlBlocks = content.match(/<pre[^>]*><code[^>]*class=["']?language-(\w+)["']?[^>]*>([\s\S]*?)<\/code><\/pre>/gi) || [];
        htmlBlocks.forEach((block, index) => {
            const match = block.match(/<pre[^>]*><code[^>]*class=["']?language-(\w+)["']?[^>]*>([\s\S]*?)<\/code><\/pre>/i);
            const language = match[1] || 'unknown';
            const code = match[2];

            codeBlocks.push({
                type: 'html',
                language: language,
                code: code,
                raw: block,
                index: index + 1 + markdownBlocks.length
            });
        });

        // Extract Jekyll highlight blocks ({% highlight %})
        const jekyllBlocks = content.match(/{%\s*highlight\s+(\w+)\s*%}([\s\S]*?){%\s*endhighlight\s*%}/g) || [];
        jekyllBlocks.forEach((block, index) => {
            const match = block.match(/{%\s*highlight\s+(\w+)\s*%}([\s\S]*?){%\s*endhighlight\s*%}/);
            const language = match[1] || 'unknown';
            const code = match[2];

            codeBlocks.push({
                type: 'jekyll',
                language: language,
                code: code,
                raw: block,
                index: index + 1 + markdownBlocks.length + htmlBlocks.length
            });
        });

        // Extract inline code
        const inlineCode = content.match(/`([^`]+)`/g) || [];
        inlineCode.forEach((code, index) => {
            if (code.length > 10) { // Only consider longer inline code
                codeBlocks.push({
                    type: 'inline',
                    language: 'unknown',
                    code: code.slice(1, -1),
                    raw: code,
                    index: index + 1 + markdownBlocks.length + htmlBlocks.length + jekyllBlocks.length
                });
            }
        });

        return codeBlocks;
    }

    analyzeCodeBlocks(codeBlocks, filePath) {
        codeBlocks.forEach(block => {
            this.results.codeBlocks.total++;

            // Count by language
            if (!this.results.languages[block.language]) {
                this.results.languages[block.language] = 0;
            }
            this.results.languages[block.language]++;

            // Validate language support
            const isValidLanguage = this.supportedLanguages.includes(block.language.toLowerCase()) ||
                                  block.language === 'unknown';

            if (!isValidLanguage) {
                this.results.codeBlocks.invalid++;
                this.results.issues.push({
                    file: filePath,
                    type: 'invalid_language',
                    language: block.language,
                    message: `Unsupported language: ${block.language}`,
                    block: block.index
                });
            }

            // Check if language is specified
            if (block.language === 'unknown' && block.type !== 'inline') {
                this.results.codeBlocks.unhighlighted++;
                this.results.issues.push({
                    file: filePath,
                    type: 'no_language',
                    message: 'Code block without language specification',
                    block: block.index
                });
            } else if (block.language !== 'unknown') {
                this.results.codeBlocks.highlighted++;
            }

            // Validate code content
            this.validateCodeContent(block, filePath);
        });
    }

    validateCodeContent(block, filePath) {
        const code = block.code.trim();

        // Check for empty code blocks
        if (code.length === 0) {
            this.results.issues.push({
                file: filePath,
                type: 'empty_code',
                language: block.language,
                message: 'Empty code block',
                block: block.index
            });
            return;
        }

        // Validate based on language
        switch (block.language.toLowerCase()) {
            case 'rust':
            case 'rs':
                this.validateRustCode(code, block, filePath);
                break;
            case 'javascript':
            case 'js':
                this.validateJavaScriptCode(code, block, filePath);
                break;
            case 'bash':
            case 'sh':
            case 'shell':
                this.validateBashCode(code, block, filePath);
                break;
            case 'html':
                this.validateHtmlCode(code, block, filePath);
                break;
            case 'css':
            case 'scss':
                this.validateCssCode(code, block, filePath);
                break;
            case 'json':
                this.validateJsonCode(code, block, filePath);
                break;
            case 'yaml':
            case 'yml':
                this.validateYamlCode(code, block, filePath);
                break;
        }

        // General validations
        this.validateGeneralCodePatterns(code, block, filePath);
    }

    validateRustCode(code, block, filePath) {
        // Check for common Rust syntax
        const rustPatterns = [
            { pattern: /fn\s+\w+/, message: 'Rust function declaration' },
            { pattern: /use\s+\w+/, message: 'Rust use statement' },
            { pattern: /struct\s+\w+/, message: 'Rust struct declaration' },
            { pattern: /impl\s+/, message: 'Rust implementation block' },
        ];

        let hasRustSyntax = rustPatterns.some(p => p.pattern.test(code));

        if (!hasRustSyntax && code.includes('println!')) {
            hasRustSyntax = true;
        }

        if (!hasRustSyntax && block.language.toLowerCase() === 'rust') {
            this.results.issues.push({
                file: filePath,
                type: 'language_mismatch',
                language: block.language,
                message: 'Code marked as Rust but doesn\'t contain Rust syntax',
                block: block.index
            });
        }
    }

    validateJavaScriptCode(code, block, filePath) {
        // Check for common JavaScript patterns
        const jsPatterns = [
            /function\s+\w+/,
            /const\s+\w+\s*=/,
            /let\s+\w+\s*=/,
            /var\s+\w+\s*=/,
            /=>\s*{/,
            /console\./,
            /document\./,
            /window\./
        ];

        const hasJsSyntax = jsPatterns.some(pattern => pattern.test(code));

        if (!hasJsSyntax && block.language.toLowerCase() === 'javascript') {
            this.results.issues.push({
                file: filePath,
                type: 'language_mismatch',
                language: block.language,
                message: 'Code marked as JavaScript but doesn\'t contain JavaScript syntax',
                block: block.index
            });
        }
    }

    validateBashCode(code, block, filePath) {
        // Check for bash-specific patterns
        const bashPatterns = [
            /^#!/,  // Shebang
            /\$\w+/, // Variables
            /echo\s+/,
            /cd\s+/,
            /ls\s+/,
            /mkdir\s+/,
            /sudo\s+/
        ];

        const hasBashSyntax = bashPatterns.some(pattern => pattern.test(code));

        if (!hasBashSyntax && ['bash', 'sh', 'shell'].includes(block.language.toLowerCase())) {
            this.results.issues.push({
                file: filePath,
                type: 'language_mismatch',
                language: block.language,
                message: 'Code marked as Bash/Shell but doesn\'t contain shell syntax',
                block: block.index
            });
        }
    }

    validateHtmlCode(code, block, filePath) {
        // Check for HTML tags
        const hasHtmlTags = /<\/?[a-zA-Z][^>]*>/;

        if (!hasHtmlTags.test(code)) {
            this.results.issues.push({
                file: filePath,
                type: 'language_mismatch',
                language: block.language,
                message: 'Code marked as HTML but doesn\'t contain HTML tags',
                block: block.index
            });
        }
    }

    validateCssCode(code, block, filePath) {
        // Check for CSS syntax
        const cssPatterns = [
            /\{[\s\S]*\}/,  // CSS rules
            /[a-zA-Z-]+:\s*[^;]+;/,  // CSS properties
            /\.[a-zA-Z-_]+/,  // Class selectors
            /#[a-zA-Z-_]+/   // ID selectors
        ];

        const hasCssSyntax = cssPatterns.some(pattern => pattern.test(code));

        if (!hasCssSyntax && ['css', 'scss'].includes(block.language.toLowerCase())) {
            this.results.issues.push({
                file: filePath,
                type: 'language_mismatch',
                language: block.language,
                message: 'Code marked as CSS but doesn\'t contain CSS syntax',
                block: block.index
            });
        }
    }

    validateJsonCode(code, block, filePath) {
        try {
            JSON.parse(code);
        } catch (error) {
            this.results.issues.push({
                file: filePath,
                type: 'syntax_error',
                language: block.language,
                message: `Invalid JSON: ${error.message}`,
                block: block.index
            });
        }
    }

    validateYamlCode(code, block, filePath) {
        // Basic YAML structure validation
        const lines = code.split('\n');
        let indentLevel = 0;

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            if (line.trim() === '') continue;

            const currentIndent = line.length - line.trimLeft().length;

            // Check for consistent indentation (YAML is sensitive)
            if (currentIndent % 2 !== 0) {
                this.results.issues.push({
                    file: filePath,
                    type: 'syntax_warning',
                    language: block.language,
                    message: `YAML line ${i + 1}: Inconsistent indentation (should be multiples of 2)`,
                    block: block.index
                });
            }
        }
    }

    validateGeneralCodePatterns(code, block, filePath) {
        // Check for placeholder text
        const placeholders = ['TODO', 'FIXME', 'XXX', 'HACK'];
        placeholders.forEach(placeholder => {
            if (code.includes(placeholder)) {
                this.results.issues.push({
                    file: filePath,
                    type: 'placeholder',
                    language: block.language,
                    message: `Code contains placeholder: ${placeholder}`,
                    block: block.index
                });
            }
        });

        // Check for very long lines
        const lines = code.split('\n');
        lines.forEach((line, index) => {
            if (line.length > 120) {
                this.results.issues.push({
                    file: filePath,
                    type: 'long_line',
                    language: block.language,
                    message: `Line ${index + 1} is ${line.length} characters (consider wrapping)`,
                    block: block.index
                });
            }
        });

        // Check for sensitive information patterns
        const sensitivePatterns = [
            { pattern: /password\s*=\s*["'][^"']+["']/i, type: 'password' },
            { pattern: /api_key\s*=\s*["'][^"']+["']/i, type: 'api_key' },
            { pattern: /secret\s*=\s*["'][^"']+["']/i, type: 'secret' },
            { pattern: /token\s*=\s*["'][^"']+["']/i, type: 'token' }
        ];

        sensitivePatterns.forEach(({ pattern, type }) => {
            if (pattern.test(code)) {
                this.results.issues.push({
                    file: filePath,
                    type: 'security',
                    language: block.language,
                    message: `Possible sensitive information: ${type}`,
                    block: block.index
                });
            }
        });
    }

    async validateSyntaxHighlighting() {
        console.log('\n🎨 Validating Syntax Highlighting Configuration...');

        // Check Jekyll configuration
        const configPath = path.join(this.sitePath, '_config.yml');
        if (fs.existsSync(configPath)) {
            const config = fs.readFileSync(configPath, 'utf8');

            if (config.includes('highlighter: rouge')) {
                console.log('  ✅ Rouge highlighter configured');
            } else if (config.includes('highlighter:')) {
                console.log('  ⚠️  Non-standard highlighter detected');
            } else {
                console.log('  ⚠️  No highlighter specified in _config.yml');
            }
        } else {
            console.log('  ❌ No _config.yml found');
        }

        // Check for syntax highlighting CSS
        const cssFiles = this.findFiles(['.css', '.scss']);
        let hasSyntaxCss = false;

        for (const cssFile of cssFiles) {
            const content = fs.readFileSync(cssFile, 'utf8');
            if (content.includes('.highlight') ||
                content.includes('.syntax-') ||
                content.includes('code[class*="language-"]')) {
                hasSyntaxCss = true;
                break;
            }
        }

        if (hasSyntaxCss) {
            console.log('  ✅ Syntax highlighting CSS found');
        } else {
            console.log('  ⚠️  No syntax highlighting CSS detected');
            this.results.issues.push({
                file: 'CSS files',
                type: 'missing_styles',
                message: 'No syntax highlighting CSS classes found'
            });
        }
    }

    async checkHighlighterConfiguration() {
        console.log('\n⚙️  Checking Highlighter Configuration...');

        // Check Gemfile for syntax highlighting gems
        const gemfilePath = path.join(this.sitePath, 'Gemfile');
        if (fs.existsSync(gemfilePath)) {
            const gemfile = fs.readFileSync(gemfilePath, 'utf8');

            if (gemfile.includes('rouge') || gemfile.includes('github-pages')) {
                console.log('  ✅ Rouge gem dependency found');
            } else {
                console.log('  ⚠️  No Rouge gem found in Gemfile');
            }
        }

        // Check for code highlighting JavaScript
        const jsFiles = this.findFiles(['.js']);
        let hasHighlightJs = false;

        for (const jsFile of jsFiles) {
            const content = fs.readFileSync(jsFile, 'utf8');
            if (content.includes('highlight') ||
                content.includes('syntax') ||
                content.includes('prism') ||
                content.includes('hljs')) {
                hasHighlightJs = true;
                console.log(`  ✅ Client-side highlighting found in ${path.basename(jsFile)}`);
                break;
            }
        }

        if (!hasHighlightJs) {
            console.log('  ℹ️  No client-side syntax highlighting JavaScript detected (using server-side only)');
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
        console.log('📋 CODE SYNTAX HIGHLIGHTING REPORT');
        console.log('='.repeat(60));

        // Summary
        console.log('\n📊 Summary:');
        console.log(`  Total code blocks: ${this.results.codeBlocks.total}`);
        console.log(`  ✅ With highlighting: ${this.results.codeBlocks.highlighted}`);
        console.log(`  ❌ Without highlighting: ${this.results.codeBlocks.unhighlighted}`);
        console.log(`  ⚠️  Invalid language: ${this.results.codeBlocks.invalid}`);

        if (this.results.codeBlocks.total > 0) {
            const highlightedPercentage = ((this.results.codeBlocks.highlighted / this.results.codeBlocks.total) * 100).toFixed(1);
            console.log(`  📈 Highlighting coverage: ${highlightedPercentage}%`);
        }

        // Language breakdown
        console.log('\n🔤 Language Distribution:');
        const sortedLanguages = Object.entries(this.results.languages)
            .sort((a, b) => b[1] - a[1]);

        sortedLanguages.forEach(([lang, count]) => {
            const supported = this.supportedLanguages.includes(lang.toLowerCase()) ? '✅' : '❌';
            console.log(`  ${supported} ${lang}: ${count} blocks`);
        });

        // Issues
        if (this.results.issues.length > 0) {
            console.log('\n🔍 ISSUES FOUND:');
            console.log('-'.repeat(60));

            const groupedIssues = {};
            this.results.issues.forEach(issue => {
                if (!groupedIssues[issue.file]) {
                    groupedIssues[issue.file] = [];
                }
                groupedIssues[issue.file].push(issue);
            });

            Object.keys(groupedIssues).forEach(file => {
                console.log(`\n📄 ${file}:`);
                groupedIssues[file].forEach(issue => {
                    const icon = issue.type === 'syntax_error' ? '❌' :
                               issue.type === 'security' ? '🔒' :
                               issue.type === 'language_mismatch' ? '⚠️ ' : 'ℹ️ ';

                    const blockInfo = issue.block ? ` (block ${issue.block})` : '';
                    console.log(`  ${icon} ${issue.message}${blockInfo}`);
                });
            });
        }

        // Recommendations
        console.log('\n💡 RECOMMENDATIONS:');
        console.log('-'.repeat(60));

        if (this.results.codeBlocks.unhighlighted > 0) {
            console.log('• Add language specifications to code blocks without highlighting');
        }

        if (this.results.codeBlocks.invalid > 0) {
            console.log('• Use supported language identifiers for better highlighting');
        }

        console.log('• Consider using Rouge-compatible language identifiers');
        console.log('• Test syntax highlighting in development environment');
        console.log('• Keep code examples up-to-date and relevant');

        // Save detailed report
        const reportData = {
            timestamp: new Date().toISOString(),
            summary: this.results.codeBlocks,
            languages: this.results.languages,
            supportedLanguages: this.supportedLanguages,
            issues: this.results.issues,
            filesAnalyzed: this.results.files
        };

        fs.writeFileSync(
            '/workspaces/caxton/scripts/validation/code-syntax-report.json',
            JSON.stringify(reportData, null, 2)
        );

        console.log('\n📁 Detailed report saved to: code-syntax-report.json');

        const hasErrors = this.results.codeBlocks.invalid > 0 ||
                         this.results.issues.some(issue => issue.type === 'syntax_error');

        if (hasErrors) {
            process.exit(1);
        }
    }
}

// Run if called directly
if (require.main === module) {
    const highlighter = new CodeSyntaxHighlighter();
    highlighter.run();
}

module.exports = CodeSyntaxHighlighter;
