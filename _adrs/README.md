# Jekyll ADR Collection Directory

⚠️ **This directory is automatically synchronized** ⚠️

## Purpose
This `_adrs/` directory contains copies of Architecture Decision Records (ADRs) for Jekyll static site generation. It exists because Jekyll requires collections to be in directories prefixed with an underscore.

## Important Notes
- **DO NOT edit files in this directory directly**
- **DO NOT add new ADRs here**
- The canonical location for ADRs is: `docs/adr/`
- Files in this directory are automatically synchronized from `docs/adr/`

## How to Add a New ADR
1. Create your ADR in `docs/adr/` following the naming convention: `NNNN-descriptive-name.md`
2. The file will be automatically copied to this directory by GitHub Actions
3. The Jekyll site will then include it in the ADR index

## Synchronization
This directory is kept in sync with `docs/adr/` through:
- Manual sync during development (temporary)
- Automated GitHub Actions workflow (coming in Phase 2)

## Background
This approach treats the `_adrs/` directory as a build artifact, similar to compiled binaries or generated documentation. It allows us to maintain a single source of truth while accommodating Jekyll's collection requirements.

For more details, see the expert discussion that led to this decision in the project's WORK.md history.