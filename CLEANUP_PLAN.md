# Documentation Reorganization Cleanup Plan

## File Manifest After Reorganization

### Current Documentation Structure

#### `/website/` - Jekyll Site (Production Documentation)

- **Purpose**: User-facing documentation website served via GitHub Pages
- **Status**: Primary documentation location
- **Structure**:
  - `_adrs/` - Architecture Decision Records for Jekyll collection (18 ADRs)
  - `docs/` - Core documentation pages (8 pages)
  - `assets/` - Static assets (CSS, JS, images)
  - `_includes/`, `_layouts/`, `_sass/` - Jekyll components

#### `/docs/` - Development Documentation (Internal Reference)

- **Purpose**: Comprehensive documentation for all audiences
- **Status**: Primary development reference
- **Structure**: Audience-first organization with 12 major sections:
  - `getting-started/` - 5 essential guides
  - `audiences/` - Role-specific documentation (4 audience types)
  - `api/` - Technical specifications (8 API references)
  - `architecture/` - System design (4 architectural documents)
  - `messaging/` - Communication protocols (5 guides)
  - `memory-system/` - Memory integration (5 documents)
  - `config-agents/` - Configuration-driven agents (5 guides)
  - `operations/` - Production guidance (5 operational guides)
  - `developer-guide/` - Development resources (3 guides)
  - `adr/` - Architecture decisions (31 ADRs)
  - Plus specialized topics in 8 additional directories

## Identified Issues

### 1. Orphaned Files Requiring Action

#### Legacy Static Site Files (REMOVE)

```bash
# Files to delete - old static site artifacts
/docs/site/adr-template.html
/docs/site/css/style.css
/docs/site/img/logo.svg
/docs/site/js/adr-carousel.js
/docs/site/js/caxton.js
/docs/site/.nojekyll
```

**Reason**: These are legacy static site files now replaced by Jekyll in `/website/`

#### Potential Duplicate Documentation

- `/docs/contributors/development-guide.md` vs `/docs/audiences/contributors/development-setup.md`
- `/docs/contributors/testing.md` vs `/docs/audiences/contributors/testing-guide.md`
- **Action Required**: Content comparison to determine merge or specialization strategy

#### Root-Level Documentation Files (EVALUATE)

```bash
/docs/README.md
/docs/TESTING.md
/docs/type_system_improvements_report.md
/docs/performance_and_safety_metrics.md
/docs/domain-types.md
/docs/wasm-runtime-architecture.md
```

**Action Required**: Determine if these should be integrated into
audience-specific sections

### 2. Directory Cleanup Plan

#### Directories to Remove

```bash
# Empty or obsolete directories
/docs/site/           # Old static site - replace with /website/
/docs/site/css/       # Empty after asset migration
/docs/site/js/        # Empty after asset migration
/docs/site/img/       # Empty after asset migration
```

#### Directories to Consolidate

- `/docs/contributors/` → merge with `/docs/audiences/contributors/`
- `/docs/developer/` → merge with `/docs/developer-guide/`

### 3. Navigation Structure Integration

#### Jekyll Navigation Configuration

✅ **COMPLETED**: Created `/website/_data/navigation.yml` with:

- Hierarchical navigation structure for all documentation sections
- Audience-first organization matching reorganized structure
- ADR categorization with proper metadata
- Search and breadcrumb configuration

#### Required Navigation Updates

- Update `/website/_config.yml` to reference new navigation data
- Update `/website/_includes/navigation.html` to use structured navigation
- Add sidebar navigation for documentation pages

## Cross-Reference Audit

### Internal Links Requiring Updates

#### Links to Moved Files

1. **ADR Cross-References**
   - All ADRs contain internal links to other ADRs
   - Format: `[ADR-XXXX](../adr/XXXX-title.md)`
   - **Status**: Need to update paths for website collection format

2. **Getting Started Guide Links**
   - Installation guide references configuration guide
   - First agent guide references API documentation
   - **Status**: Update paths to new audience-specific locations

3. **API Documentation Cross-References**
   - API specs reference implementation guides
   - Config agent docs reference messaging patterns
   - **Status**: Update paths to new structured locations

#### External Links to Verify

- GitHub repository links in documentation
- Links to external FIPA specification documents
- Links to WebAssembly and technology documentation

### Link Validation Requirements

#### Documentation Page Updates Needed

```yaml
# Pages with known internal link dependencies
/docs/getting-started/first-agent.md:
  - Links to: API reference, configuration guide, deployment docs

/docs/api/config-agents.md:
  - Links to: Messaging patterns, memory integration, examples

/docs/audiences/contributors/architecture-overview.md:
  - Links to: ADRs, specifications, technical guides

/website/docs/ARCHITECTURE.md:
  - Links to: ADR collection, technical specifications
```

#### Pattern-Based Link Updates

- Relative links to `../adr/` → `/adr/` (website format)
- Relative links to `../api/` → `/docs/api/` (documentation format)
- Relative links to `../getting-started/` → `/docs/getting-started/`

## Implementation Checklist

### Phase 1: File Cleanup (IMMEDIATE)

- [ ] Remove `/docs/site/` directory entirely
- [ ] Delete legacy static site artifacts
- [ ] Remove system files (`.DS_Store`, `Thumbs.db`, etc.)

### Phase 2: Content Consolidation (HIGH PRIORITY)

- [ ] Compare duplicate files in `/contributors/` vs `/audiences/contributors/`
- [ ] Merge or specialize duplicate content
- [ ] Move root-level docs to appropriate audience sections
- [ ] Update file paths in moved content

### Phase 3: Navigation Integration (HIGH PRIORITY)

- [ ] Update Jekyll configuration to use new navigation structure
- [ ] Test navigation functionality on development server
- [ ] Add documentation page templates with proper navigation
- [ ] Implement sidebar navigation for deep content

### Phase 4: Link Validation (CRITICAL)

- [ ] Run automated link checker on all documentation
- [ ] Update internal links to reflect new structure
- [ ] Verify external links are still valid
- [ ] Update relative path references

### Phase 5: Validation (BEFORE DEPLOYMENT)

- [ ] Build Jekyll site locally to verify no broken links
- [ ] Test all navigation paths work correctly
- [ ] Verify search functionality includes all content
- [ ] Check responsive design on mobile devices
- [ ] Validate HTML and accessibility compliance

## Validation Checklist

### Content Validation

- [ ] All documentation is accessible through navigation
- [ ] No broken internal links between pages
- [ ] All images and assets load correctly
- [ ] Search functionality returns relevant results

### Jekyll Site Validation

- [ ] Site builds without errors or warnings
- [ ] Navigation menus function on all screen sizes
- [ ] ADR collection displays correctly with metadata
- [ ] Documentation layouts render properly

### User Experience Validation

- [ ] Clear path from homepage to all major documentation sections
- [ ] Logical progression through getting-started guides
- [ ] Easy discovery of audience-specific documentation
- [ ] Consistent styling and branding throughout

## Risk Mitigation

### Backup Strategy

- Current git branch preserves all original files
- No destructive changes until validation complete
- Staged rollout with incremental testing

### Rollback Plan

- Keep current documentation structure until new site validated
- Maintain old navigation until new navigation tested
- Gradual migration of external links pointing to documentation

## Success Metrics

### Quantitative Measures

- Zero broken internal links in final validation
- All 100+ documentation files accessible through navigation
- Jekyll site builds in <30 seconds
- Search returns results for all major topics

### Qualitative Measures

- Clear user journey for each audience type
- Logical information architecture
- Consistent visual design and branding
- Mobile-responsive design throughout

## Notes

### Technology Decisions

- Jekyll used for static site generation (GitHub Pages compatible)
- Lunr.js for client-side search functionality
- Responsive design with mobile-first approach
- Accessibility compliance (WCAG 2.1 AA)

### Maintenance Considerations

- Navigation structure defined in data files for easy updates
- ADR collection automatically generates index pages
- Search index updates automatically with content changes
- CSS/JS assets optimized for performance

This cleanup plan provides a systematic approach to finalizing the
documentation reorganization while maintaining content integrity and user
experience.
