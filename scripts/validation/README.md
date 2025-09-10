# Caxton Website Validation Suite

Comprehensive technical validation scripts for the Caxton multi-agent
orchestration platform website.

## Overview

This validation suite provides automated testing and quality assurance for the
Caxton Jekyll-based website. It includes seven specialized validators that check
different aspects of website functionality, accessibility, and deployment
readiness.

## Validators Included

### 1. 🔗 Dead Link Checker (`dead-link-checker.js`)

- **Purpose**: Validates internal and external links across the site
- **Checks**:
  - Link accessibility and response codes
  - Broken internal references
  - Missing assets and images
  - Invalid anchor targets
- **Critical**: Yes

### 2. 📄 HTML/CSS Validator (`html-css-validator.js`)

- **Purpose**: Validates HTML structure, CSS syntax, and accessibility
- **Checks**:
  - HTML5 compliance and semantic structure
  - CSS syntax errors and browser compatibility
  - ARIA attributes and accessibility features
  - Form label associations
  - Heading hierarchy
- **Critical**: Yes

### 3. 📜 JavaScript Error Checker (`js-error-checker.js`)

- **Purpose**: Detects JavaScript syntax errors and runtime issues
- **Checks**:
  - Syntax validation and static analysis
  - Console usage and debugging statements
  - Performance anti-patterns
  - Memory leak potential
  - Security vulnerabilities (eval, innerHTML)
- **Critical**: Yes

### 4. 🎨 Code Syntax Highlighter (`code-syntax-highlighter.js`)

- **Purpose**: Validates code block syntax highlighting
- **Checks**:
  - Language specification accuracy
  - Rouge highlighter compatibility
  - Code example validity
  - Security issues in code samples
- **Critical**: No

### 5. 🏷️ SEO Meta Validator (`seo-meta-validator.js`)

- **Purpose**: Validates SEO elements and metadata
- **Checks**:
  - Title and description tags
  - Open Graph and Twitter Card tags
  - Image alt text coverage
  - Heading structure for SEO
  - Structured data validation
- **Critical**: No

### 6. 🔨 Build/Deployment Checker (`build-deployment-checker.js`)

- **Purpose**: Validates Jekyll build and GitHub Pages deployment
- **Checks**:
  - Jekyll configuration compatibility
  - GitHub Actions workflow validation
  - Dependency management (Gemfile)
  - Asset optimization
  - Build process testing
- **Critical**: Yes

### 7. 📱 Responsive Design Checker (`responsive-design-checker.js`)

- **Purpose**: Validates responsive design implementation
- **Checks**:
  - Viewport configuration
  - Media query breakpoints
  - Responsive image implementation
  - Mobile navigation patterns
  - Touch target accessibility
- **Critical**: No

## Installation & Usage

### Quick Start

```bash
# Navigate to validation directory
cd scripts/validation

# Run all validations
npm run validate

# Or run the master script directly
node run-all-validations.js
```

### Individual Validators

```bash
# Run specific validation
npm run validate:links        # Dead link checking
npm run validate:html-css     # HTML/CSS validation
npm run validate:js           # JavaScript error checking
npm run validate:syntax       # Code syntax highlighting
npm run validate:seo          # SEO meta tag validation
npm run validate:build        # Build/deployment checking
npm run validate:responsive   # Responsive design validation
```

### Command Line Options

```bash
# Run only specific check
node run-all-validations.js --only seoMeta

# Skip specific check
node run-all-validations.js --skip buildDeploy

# Show help
node run-all-validations.js --help

# Show version
node run-all-validations.js --version
```

## Output & Reports

### Console Output

Each validator provides real-time console output with:

- ✅ Success indicators
- ❌ Error messages
- ⚠️ Warning notifications
- 📊 Progress and statistics

### JSON Reports

Detailed JSON reports are generated for each validator:

- `dead-link-report.json`
- `html-css-report.json`
- `js-error-report.json`
- `code-syntax-report.json`
- `seo-meta-report.json`
- `build-deployment-report.json`
- `responsive-design-report.json`
- `master-validation-report.json` (combined summary)

### Exit Codes

- `0`: All validations passed
- `1`: Critical validation failures detected
- `2`: Non-critical issues found

## Understanding Results

### Critical vs Non-Critical Issues

**Critical Issues** (must be fixed before deployment):

- Dead links that break user experience
- HTML/CSS syntax errors causing browser issues
- JavaScript errors preventing functionality
- Build/deployment configuration problems

**Non-Critical Issues** (recommended fixes):

- Missing SEO optimization
- Code syntax highlighting problems
- Responsive design improvements
- Performance optimizations

### Success Metrics

The validation suite tracks:

- **Success Rate**: Percentage of checks passed
- **Response Time**: Individual validator performance
- **Coverage**: Areas of the website validated
- **Trend Analysis**: Issue resolution over time

## Integration

### CI/CD Pipeline

Add to your GitHub Actions workflow:

```yaml
- name: Validate Website
  run: |
    cd scripts/validation
    node run-all-validations.js
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
cd scripts/validation && npm run validate
```

### Development Workflow

1. Run validations locally before committing
2. Address critical issues immediately
3. Plan non-critical improvements
4. Monitor validation results in CI/CD

## Configuration

### Customizing Validators

Each validator can be configured by modifying constants at the top of the
respective files:

```javascript
// Example: Configure dead link checker timeouts
const TIMEOUT_MS = 10000; // 10 seconds

// Example: Configure breakpoints in responsive checker
const STANDARD_BREAKPOINTS = {
    'tablet': '768px',
    'desktop': '1024px'
};
```

### Adding Custom Checks

1. Create new validator class extending base patterns
2. Add to `run-all-validations.js` validators array
3. Update package.json scripts
4. Document in this README

## Troubleshooting

### Common Issues

**Jekyll not found**:

```bash
gem install jekyll bundler
```

**Permission errors**:

```bash
chmod +x *.js
```

**Node.js version**: Requires Node.js 14.0.0 or higher

**Large repository timeouts**: Increase timeout values in validator
configuration

### Debug Mode

Add verbose logging by setting environment variable:

```bash
DEBUG=true node run-all-validations.js
```

## Development

### Adding New Validators

1. Follow the pattern established in existing validators
2. Implement async `run()` method
3. Generate JSON report with consistent structure
4. Add appropriate console logging
5. Handle errors gracefully
6. Update master runner configuration

### Testing Validators

```bash
# Test on sample files
mkdir test-site
cd test-site
# ... create test content
cd ../scripts/validation
node dead-link-checker.js
```

## Maintenance

### Regular Updates

- Keep Jekyll and dependencies updated
- Review validator logic for new web standards
- Update breakpoints and responsive design patterns
- Monitor for new accessibility guidelines

### Performance Optimization

- Cache results where appropriate
- Implement parallel processing for large sites
- Optimize file system operations
- Consider incremental validation for CI

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add or improve validators
4. Test thoroughly with various website configurations
5. Update documentation
6. Submit pull request

## License

MIT License - see LICENSE file for details.

## Support

- GitHub Issues: [Create Issue](https://github.com/jwilger/caxton/issues)
- Documentation: [Caxton Website](https://jwilger.github.io/caxton)
- Community: [GitHub Discussions](https://github.com/jwilger/caxton/discussions)

______________________________________________________________________

*This validation suite is part of the Caxton multi-agent orchestration platform
project.*
