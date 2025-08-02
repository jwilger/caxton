# Jekyll Migration Plan: Static HTML to Jekyll

**Migration Planner Agent - Comprehensive Migration Strategy**

## Executive Summary

This plan outlines the migration from the current static HTML site to Jekyll while maintaining:
- ✅ Zero downtime
- ✅ All current functionality 
- ✅ URL structure preservation
- ✅ Design consistency
- ✅ ADR carousel functionality

## Current State Analysis

### Static Site Structure
```
docs/site/
├── index.html                 # Main landing page (4,000+ lines)
├── adr-template.html          # ADR page template  
├── css/style.css              # Catppuccin Mocha theme (1,100+ lines)
└── js/
    ├── caxton.js              # Main site functionality
    └── adr-carousel.js        # ADR carousel module
```

### Existing Jekyll Setup  
```
jekyll-migration/
├── _config.yml                # Comprehensive Jekyll configuration
├── Gemfile                    # GitHub Pages compatible dependencies
├── _layouts/                  # Empty - needs population
├── _includes/                 # Empty - needs population  
├── _sass/                     # Empty - needs SCSS migration
└── assets/                    # Empty - needs asset migration
```

### ADR Collection
```
docs/adr/
├── 0001-observability-first-architecture.md
├── 0002-webassembly-for-agent-isolation.md
├── 0003-fipa-messaging-protocol.md
├── 0004-minimal-core-philosophy.md
└── 0005-mcp-for-external-tools.md
```

## Migration Strategy

### Phase 1: Template Conversion (Priority: High)

#### 1.1 Create Jekyll Layouts

**Main Layout (`_layouts/default.html`)**
- Extract common HTML structure from `index.html`
- Convert to Liquid template with `{{ content }}` placeholder
- Preserve all meta tags, scripts, and styling

**ADR Layout (`_layouts/adr.html`)**  
- Convert `adr-template.html` to Jekyll layout
- Add collection navigation
- Maintain carousel script integration

**Home Layout (`_layouts/home.html`)**
- Specialized layout for homepage
- Includes hero section, features grid, etc.
- Inherits from default layout

#### 1.2 Create Jekyll Includes

**Navigation (`_includes/navigation.html`)**
- Extract navbar from static HTML
- Make responsive to current page
- Support for site.navigation config

**Footer (`_includes/footer.html`)**
- Extract footer HTML
- Make configurable via `_config.yml`

**Head (`_includes/head.html`)**
- Meta tags, stylesheets, fonts
- SEO optimization with jekyll-seo-tag

### Phase 2: Asset Migration (Priority: Medium)

#### 2.1 SCSS Migration
```
_sass/
├── _variables.scss            # Catppuccin color variables
├── _base.scss                 # Global styles, typography  
├── _components.scss           # Buttons, cards, navigation
├── _layout.scss               # Grid, sections, responsive
├── _features.scss             # Feature-specific styles
└── _adr.scss                  # ADR carousel and layout
```

#### 2.2 JavaScript Migration
```
assets/js/
├── caxton.js                  # View tabs, smooth scrolling
└── adr-carousel.js            # ADR carousel (unchanged)
```

#### 2.3 Image Migration  
```
assets/img/
└── logo.svg                   # Caxton logo
```

### Phase 3: ADR Collection Setup (Priority: High)

#### 3.1 Collection Configuration
Already configured in `_config.yml`:
```yaml
collections:
  adrs:
    output: true
    permalink: /adr/:name/
    sort_by: date
```

#### 3.2 ADR Migration Process

**Frontend Matter Addition**
Convert existing ADRs to include Jekyll front matter:
```yaml
---
layout: adr
title: "Observability-First Architecture"
date: 2025-01-31
status: proposed
category: Architecture
adr_number: "0001"
---
```

**Automated Index Generation**
- Use Jekyll's collection features
- Generate dynamic ADR index with carousel
- Maintain backward compatibility

### Phase 4: URL Preservation (Priority: High)

#### 4.1 URL Mapping Strategy

**Current URLs → Jekyll URLs**
```
/                     → /                    (unchanged)
/adr/                 → /adr/                (unchanged)  
/adr/0001-...html     → /adr/0001-...       (remove .html)
/caxton/              → /caxton/             (unchanged - rustdoc)
```

**Redirect Rules (if needed)**
```html
<!-- In _includes/redirects.html -->
<script>
if (window.location.pathname.endsWith('.html') && window.location.pathname.includes('/adr/')) {
  window.location.replace(window.location.pathname.replace('.html', ''));
}
</script>
```

### Phase 5: GitHub Actions Migration (Priority: Medium)

#### 5.1 New Jekyll Workflow

**Current Workflow Issues:**
- Uses Pandoc for ADR conversion
- Manual rustdoc building  
- Complex shell scripting

**New Jekyll Workflow:**
```yaml
name: Deploy Jekyll Site to GitHub Pages

on:
  push:
    branches: [ main ]
  release:
    types: [published]
  workflow_dispatch:

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Ruby
        uses: ruby/setup-ruby@v1
        with:
          ruby-version: '3.1'
          bundler-cache: true
          working-directory: ./jekyll-migration
          
      - name: Setup Rust (for rustdoc)  
        uses: actions-rust-lang/setup-rust-toolchain@v1
        
      - name: Build Documentation
        run: |
          cargo doc --no-deps --all-features
          mkdir -p jekyll-migration/_site/caxton
          cp -r target/doc/* jekyll-migration/_site/caxton/
          
      - name: Build Jekyll Site
        run: |
          cd jekyll-migration
          bundle exec jekyll build
          
      - name: Deploy to GitHub Pages
        uses: actions/deploy-pages@v4
        with:
          path: jekyll-migration/_site
```

### Phase 6: Functionality Preservation (Priority: High)

#### 6.1 ADR Carousel Migration

**Current Implementation:**
- JavaScript detects ADR index page
- Converts `<ul>` to carousel dynamically
- Supports navigation, auto-play, responsive design

**Jekyll Implementation:**
- Use Jekyll collections to generate carousel data
- Maintain same JavaScript functionality
- Add Liquid template for carousel structure

**Carousel Template (`_includes/adr-carousel.html`):**
```liquid
{% if site.features.adr_carousel %}
<div class="adr-carousel-container">
  <div class="adr-carousel">
    {% for adr in site.adrs reversed %}
    <div class="adr-carousel-item" onclick="location.href='{{ adr.url }}'">
      <div class="adr-number">ADR {{ adr.adr_number }}</div>
      <h3 class="adr-title">{{ adr.title }}</h3>
      <p class="adr-description">{{ adr.excerpt | strip_html | truncate: 150 }}</p>
    </div>
    {% endfor %}
  </div>
</div>
{% endif %}
```

#### 6.2 Navigation Functionality
- Preserve smooth scrolling
- Maintain responsive design
- Keep GitHub link styling

#### 6.3 Interactive Elements
- View tabs in architecture section
- Code syntax highlighting  
- Responsive mobile menu

## Migration Checklist

### Pre-Migration
- [ ] Backup current static site
- [ ] Test Jekyll build locally
- [ ] Verify all assets load correctly
- [ ] Test ADR carousel functionality
- [ ] Validate responsive design

### Migration Execution  
- [ ] Create Jekyll layouts and includes
- [ ] Migrate SCSS and JavaScript
- [ ] Convert ADR collection
- [ ] Test URL preservation
- [ ] Update GitHub Actions workflow
- [ ] Deploy to staging environment

### Post-Migration Validation
- [ ] Verify all pages load correctly
- [ ] Test ADR carousel functionality
- [ ] Check mobile responsiveness
- [ ] Validate SEO metadata
- [ ] Performance testing
- [ ] Cross-browser testing

### Rollback Plan
- [ ] Keep static site in `docs/site-backup/`
- [ ] Maintain old GitHub Actions workflow as `.yml.backup`
- [ ] Document rollback procedure
- [ ] Test rollback process in staging

## Risk Mitigation

### High-Risk Items
1. **ADR Carousel Functionality**
   - Risk: Complex JavaScript may break
   - Mitigation: Thorough testing, maintain exact same JS

2. **URL Structure Changes**  
   - Risk: Broken external links
   - Mitigation: Preserve exact URL structure, add redirects

3. **GitHub Actions Complexity**
   - Risk: Build failures
   - Mitigation: Gradual migration, keep backup workflow

### Medium-Risk Items
1. **CSS Framework Migration**
   - Risk: Styling inconsistencies
   - Mitigation: SCSS variables for consistency

2. **Asset Loading**
   - Risk: Missing images/fonts
   - Mitigation: Asset fingerprinting, thorough testing

## Implementation Timeline

### Week 1: Template Setup
- Create Jekyll layouts and includes
- Basic site structure working
- Local development environment

### Week 2: Asset Migration  
- Convert CSS to SCSS
- Migrate JavaScript files
- Test responsive design

### Week 3: ADR Collection
- Convert ADR markdown files
- Implement carousel in Jekyll
- Test functionality

### Week 4: Integration & Testing
- GitHub Actions migration
- End-to-end testing
- Performance optimization

### Week 5: Deployment
- Deploy to production
- Monitor for issues
- Documentation updates

## Success Criteria

✅ **Zero Downtime**: Site remains accessible during migration
✅ **Functionality Preserved**: All features work identically
✅ **URL Compatibility**: No broken links or redirects needed
✅ **Performance Maintained**: Page load times unchanged or improved
✅ **Design Consistency**: Pixel-perfect visual match
✅ **Mobile Responsiveness**: All devices supported
✅ **SEO Maintained**: Search rankings unaffected

## Additional Benefits

### Jekyll Advantages
- **Content Management**: Easier ADR creation and editing
- **Plugin Ecosystem**: SEO, analytics, search functionality  
- **GitHub Integration**: Native GitHub Pages support
- **Performance**: Built-in asset optimization
- **Maintainability**: Cleaner, more modular codebase

### Future Enhancements
- ADR search functionality
- ADR timeline visualization
- Automated ADR numbering
- RSS feeds for ADR updates
- Multi-language support potential

---

**Generated by Migration Planner Agent**  
*Part of Caxton Multi-Agent Orchestration Platform*