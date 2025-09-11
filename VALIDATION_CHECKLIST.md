# Documentation Reorganization Validation Checklist

## Pre-Deployment Validation

### ✅ Phase 1: Structure Analysis (COMPLETED)

- [x] Complete file manifest created
- [x] Navigation structure designed and implemented
- [x] Orphaned files identified
- [x] Cleanup plan documented
- [x] Cross-reference audit completed

### 🔄 Phase 2: File Cleanup (READY TO EXECUTE)

#### Legacy File Removal

- [ ] Remove `/docs/site/` directory entirely

  ```bash
  rm -rf /home/jwilger/projects/caxton/docs/site/
  ```

- [ ] Verify no critical assets were lost in removal
- [ ] Clean up any system files (`.DS_Store`, `Thumbs.db`)

#### Content Consolidation

- [ ] Compare and merge duplicate files:
  - `/docs/contributors/development-guide.md` vs `/docs/audiences/contributors/development-setup.md`
  - `/docs/contributors/testing.md` vs `/docs/audiences/contributors/testing-guide.md`
- [ ] Move root-level docs to appropriate sections:
  - `README.md` → integrate with main documentation index
  - `TESTING.md` → merge with contributor testing guide
  - `type_system_improvements_report.md` → move to contributors section
  - `domain-types.md` → integrate with developer guide

#### Directory Cleanup

- [ ] Remove empty directories after file cleanup
- [ ] Consolidate `/docs/contributors/` into `/docs/audiences/contributors/`
- [ ] Merge `/docs/developer/` with `/docs/developer-guide/`

### 🔄 Phase 3: Navigation Integration (READY TO IMPLEMENT)

#### Jekyll Configuration Updates

- [ ] Update `_config.yml` to reference navigation data file
- [ ] Test navigation component with new structure
- [ ] Add sidebar navigation for documentation pages
- [ ] Implement breadcrumb navigation

#### Navigation Testing

- [ ] Verify all navigation links are functional
- [ ] Test mobile responsive navigation
- [ ] Ensure keyboard navigation accessibility
- [ ] Validate aria labels and screen reader support

### 🔄 Phase 4: Link Validation (CRITICAL)

#### Automated Link Checking

- [ ] Run link checker on entire documentation set

  ```bash
  # Install link checker if needed
  npm install -g markdown-link-check

  # Check all markdown files
  find /home/jwilger/projects/caxton/docs -name "*.md" -exec markdown-link-check {} \;
  find /home/jwilger/projects/caxton/website -name "*.md" -exec markdown-link-check {} \;
  ```

#### Manual Link Updates Required

- [ ] Update ADR cross-references for new Jekyll collection format
- [ ] Fix getting-started guide internal links
- [ ] Update API documentation cross-references
- [ ] Verify external links are still accessible

#### Specific Link Patterns to Update

- [ ] `../adr/XXXX-title.md` → `/adr/XXXX-title/` (Jekyll format)
- [ ] `../api/filename.md` → `/docs/api/filename/` (documentation format)
- [ ] `../getting-started/filename.md` → `/docs/getting-started/filename/`
- [ ] Relative image paths to absolute paths where needed

### 🔄 Phase 5: Jekyll Site Validation (BEFORE DEPLOYMENT)

#### Build Testing

- [ ] Jekyll site builds without errors

  ```bash
  cd /home/jwilger/projects/caxton/website
  bundle install
  bundle exec jekyll build
  ```

- [ ] No broken internal links in build output
- [ ] All collections render correctly
- [ ] CSS and JavaScript assets load properly

#### Content Validation

- [ ] All documentation pages accessible through navigation
- [ ] ADR collection displays with proper metadata
- [ ] Search functionality returns relevant results
- [ ] Mobile responsive design works across device sizes

#### Performance Validation

- [ ] Site builds in reasonable time (<2 minutes)
- [ ] Page load times acceptable (<3 seconds)
- [ ] Search index builds correctly
- [ ] Images optimized for web delivery

### 🔄 Phase 6: User Experience Testing

#### Navigation Flow Testing

- [ ] Clear path from homepage to all major sections
- [ ] Logical progression through getting-started guides
- [ ] Easy discovery of audience-specific documentation
- [ ] Intuitive ADR browsing experience

#### Content Accessibility Testing

- [ ] All headings follow proper hierarchy (h1 → h2 → h3)
- [ ] Alt text provided for all images
- [ ] Color contrast meets WCAG 2.1 AA standards
- [ ] Keyboard navigation fully functional

#### Cross-Browser Testing

- [ ] Functionality verified in Chrome
- [ ] Functionality verified in Firefox
- [ ] Functionality verified in Safari
- [ ] Mobile browser testing completed

## Deployment Validation

### Pre-Deployment Checklist

- [ ] All cleanup tasks completed successfully
- [ ] Zero broken internal links confirmed
- [ ] Jekyll site builds without warnings
- [ ] Navigation structure fully functional
- [ ] Search functionality operational

### Deployment Process

- [ ] Create deployment branch from current state
- [ ] Deploy to staging environment first
- [ ] Perform full site testing on staging
- [ ] Document any issues found during staging
- [ ] Address issues before production deployment

### Post-Deployment Validation

- [ ] Verify site loads correctly on GitHub Pages
- [ ] Test all navigation paths work in production
- [ ] Confirm search functionality works on live site
- [ ] Monitor for any 404 errors in logs
- [ ] Validate external links still work from production

## Rollback Plan

### If Issues Found

- [ ] Document specific issues encountered
- [ ] Determine if issues are fixable quickly (< 30 minutes)
- [ ] If not quickly fixable, rollback to previous version
- [ ] Create issues for problems to address later
- [ ] Plan remediation timeline

### Rollback Steps

1. Revert to previous git commit/branch
2. Redeploy previous version to production
3. Update team on rollback status
4. Schedule fix implementation
5. Plan re-deployment after fixes

## Success Criteria

### Quantitative Measures

- [ ] Zero broken internal links in final validation
- [ ] 100% of documentation accessible through navigation
- [ ] Jekyll site builds successfully in <2 minutes
- [ ] All major content types searchable
- [ ] Mobile responsive score >90% on Google PageSpeed

### Qualitative Measures

- [ ] Clear user journey for each audience type
- [ ] Logical information architecture maintained
- [ ] Consistent visual design and branding
- [ ] Professional appearance suitable for public documentation
- [ ] Intuitive navigation that reduces user confusion

## Risk Mitigation

### Backup Strategy

- [ ] Current state backed up in git branch
- [ ] Previous working version tagged for easy rollback
- [ ] Critical documentation backed up separately
- [ ] Asset files preserved during cleanup

### Communication Plan

- [ ] Team notified of deployment timeline
- [ ] Documentation updates communicated to users
- [ ] Migration guide provided for any changed URLs
- [ ] Feedback channel established for post-deployment issues

## Final Sign-off

### Technical Review

- [ ] Lead developer reviewed cleanup plan
- [ ] All automated tests passing
- [ ] Code review completed for navigation changes
- [ ] Security review completed for any new features

### Content Review

- [ ] Documentation accuracy verified
- [ ] All audience needs addressed
- [ ] Information architecture approved
- [ ] Brand consistency maintained

### Deployment Authorization

- [ ] All validation criteria met
- [ ] Rollback plan confirmed ready
- [ ] Team available for post-deployment support
- [ ] ✅ **DEPLOYMENT APPROVED**

---

## Notes

### Tools Used

- **Jekyll**: Static site generator for GitHub Pages
- **Lunr.js**: Client-side search functionality
- **Markdown-link-check**: Automated link validation
- **Google PageSpeed**: Performance testing
- **WAVE**: Accessibility testing tool

### Documentation Standards Applied

- Audience-first organization
- Consistent file naming conventions
- Proper front matter for Jekyll
- Mobile-responsive design principles
- Accessibility compliance (WCAG 2.1 AA)

This validation checklist ensures a systematic approach to verifying the
documentation reorganization before deployment to production.
