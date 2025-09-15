# Caxton Design System and Style Guide

**Version**: 1.0
**Date**: 2025-09-14
**Status**: Phase 5 - Design System Complete
**Author**: ux-ui-design-expert
**Methodology**: Atomic Design

## Executive Summary

This style guide defines the complete design system for Caxton, a multi-agent
orchestration platform optimized for **5-10 minute developer onboarding**.
The design system follows Atomic Design methodology, progressing from
foundational atoms through complete page experiences. All patterns prioritize
developer productivity, terminal-first interaction, and progressive
enhancement with HTMX for optional web interfaces.

## Design Principles

### Core Principles

1. **Developer-First**: Optimize for developer productivity and rapid onboarding
2. **Clarity Over Cleverness**: Clear, explicit interfaces over implicit magic
3. **Progressive Disclosure**: Show simple defaults, reveal complexity when
   needed
4. **Consistency**: Uniform patterns across CLI, API, and web interfaces
5. **Accessibility by Default**: WCAG 2.1 AA compliance built-in
6. **Performance Conscious**: Sub-100ms CLI feedback, efficient rendering
7. **Error Recovery**: Actionable error messages with suggested fixes
8. **Zero-Friction**: 5-10 minute from install to working agents

## Design Tokens

### Color Palette

#### Terminal Colors (Primary Interface)

```yaml
# ANSI Color Codes for Terminal
terminal_colors:
  # Core Status Colors
  success: "\033[32m" # Green - successful operations
  warning: "\033[33m" # Yellow - warnings and attention
  error: "\033[31m" # Red - errors and failures
  info: "\033[34m" # Blue - informational messages

  # Text Hierarchy
  primary: "\033[0m" # Default text
  secondary: "\033[90m" # Gray - secondary information
  emphasis: "\033[1m" # Bold - important text
  dim: "\033[2m" # Dim - de-emphasized content

  # Interactive Elements
  link: "\033[36m" # Cyan - clickable/interactive
  highlight: "\033[43m" # Yellow background - highlighted
  selection: "\033[7m" # Inverted - selected items
```

#### Web Interface Colors (Optional Enhancement)

```css
:root {
  /* Primary Brand Colors */
  --caxton-primary: #2563eb; /* Blue-600 - primary actions */
  --caxton-primary-hover: #1d4ed8; /* Blue-700 - hover state */
  --caxton-secondary: #64748b; /* Slate-500 - secondary elements */

  /* Status Colors */
  --caxton-success: #16a34a; /* Green-600 */
  --caxton-warning: #ca8a04; /* Yellow-600 */
  --caxton-error: #dc2626; /* Red-600 */
  --caxton-info: #2563eb; /* Blue-600 */

  /* Background Hierarchy */
  --caxton-bg-primary: #ffffff; /* Main background */
  --caxton-bg-secondary: #f8fafc; /* Subtle background */
  --caxton-bg-tertiary: #f1f5f9; /* Cards and containers */

  /* Text Hierarchy */
  --caxton-text-primary: #0f172a; /* Slate-900 - primary text */
  --caxton-text-secondary: #475569; /* Slate-600 - secondary text */
  --caxton-text-muted: #94a3b8; /* Slate-400 - muted text */

  /* Borders and Dividers */
  --caxton-border: #e2e8f0; /* Slate-200 */
  --caxton-border-strong: #cbd5e1; /* Slate-300 */

  /* Dark Mode Support */
  --caxton-dark-bg: #0f172a; /* Slate-900 */
  --caxton-dark-surface: #1e293b; /* Slate-800 */
  --caxton-dark-text: #f8fafc; /* Slate-50 */
}
```

### Typography

#### Terminal Typography

````yaml
terminal_typography:
  # Font Families (terminal-dependent)
  monospace: "Native terminal font"

  # Text Styles
  heading_1: "══════════════════════════════════════" # Box drawing
  heading_2: "──────────────────────────────────────" # Lighter divider
  heading_3: "▶ " # Arrow prefix

  # Text Formatting
  code_inline: "`code`" # Backticks for inline code
  code_block: "```\n...\n```" # Triple backticks for blocks
  emphasis: "*text*" # Asterisks for emphasis
  strong: "**text**" # Double asterisks for strong
````

#### Web Typography

```css
:root {
  /* Font Families */
  --font-sans: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  --font-mono: "JetBrains Mono", "Fira Code", "Cascadia Code", monospace;

  /* Font Sizes */
  --text-xs: 0.75rem; /* 12px */
  --text-sm: 0.875rem; /* 14px */
  --text-base: 1rem; /* 16px */
  --text-lg: 1.125rem; /* 18px */
  --text-xl: 1.25rem; /* 20px */
  --text-2xl: 1.5rem; /* 24px */
  --text-3xl: 1.875rem; /* 30px */

  /* Line Heights */
  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.625;

  /* Font Weights */
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
}
```

### Spacing System

```yaml
spacing:
  # Base unit: 4px
  0: 0
  1: 4px # Tight spacing
  2: 8px # Small spacing
  3: 12px # Medium-small
  4: 16px # Default spacing
  5: 20px # Medium-large
  6: 24px # Large spacing
  8: 32px # Extra large
  10: 40px # Huge spacing
  12: 48px # Massive spacing

  # Terminal-specific (character-based)
  terminal:
    indent: "  " # 2 spaces
    section: "\n\n" # Double newline
    list: " • " # Bullet with space
```

### Timing and Animation

```yaml
timing:
  # Response Times (Critical for UX)
  instant: 0-100ms # CLI acknowledgment
  fast: 100-300ms # Quick operations
  moderate: 300-1000ms # Standard operations
  slow: 1-3s # Complex operations

  # Animation Durations (Web)
  transition_fast: 150ms
  transition_normal: 250ms
  transition_slow: 350ms

  # Easing Functions
  ease_in_out: cubic-bezier(0.4, 0, 0.2, 1)
  ease_out: cubic-bezier(0.0, 0, 0.2, 1)
  ease_in: cubic-bezier(0.4, 0, 1, 1)
```

## Atoms - Foundational Elements

### CLI Atoms

#### Command Prompt

```bash
# Standard prompt format
$ caxton [resource] [action] [options]

# Examples
$ caxton agent create --template data-analyzer
$ caxton server start --port 8080
$ caxton help init
```

#### Status Indicators

```yaml
cli_status_indicators:
  success: "✓" # Check mark
  error: "✗" # Cross mark
  warning: "⚠" # Warning triangle
  info: "ℹ" # Info symbol
  pending: "○" # Empty circle
  in_progress: "◐" # Half-filled circle
  complete: "●" # Filled circle
```

#### Progress Indicators

```bash
# Spinner variations
spinner_dots: "⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏"
spinner_line: "- \\ | /"
spinner_arrow: "← ↖ ↑ ↗ → ↘ ↓ ↙"

# Progress bar
[████████████████████████░░░░░░░░░░] 67% | 2.3s remaining
```

### Web Atoms

#### Buttons

```html
<!-- Primary Button -->
<button
  class="caxton-btn caxton-btn-primary"
  hx-post="/api/agents"
  hx-target="#agent-list"
  hx-indicator="#loading"
>
  Create Agent
</button>

<!-- Secondary Button -->
<button class="caxton-btn caxton-btn-secondary">Cancel</button>

<!-- Danger Button -->
<button
  class="caxton-btn caxton-btn-danger"
  hx-delete="/api/agents/{{id}}"
  hx-confirm="Are you sure?"
>
  Delete
</button>
```

```css
.caxton-btn {
  padding: var(--spacing-2) var(--spacing-4);
  border-radius: 6px;
  font-weight: var(--font-medium);
  transition: all var(--transition-fast);
  cursor: pointer;
  border: 1px solid transparent;
}

.caxton-btn-primary {
  background: var(--caxton-primary);
  color: white;
}

.caxton-btn-primary:hover {
  background: var(--caxton-primary-hover);
}

.caxton-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

#### Form Inputs

```html
<!-- Text Input -->
<div class="caxton-form-group">
  <label for="agent-name" class="caxton-label"> Agent Name </label>
  <input
    type="text"
    id="agent-name"
    class="caxton-input"
    placeholder="e.g., data-analyzer"
    hx-post="/api/validate/name"
    hx-trigger="keyup changed delay:500ms"
    hx-target="#name-validation"
  />
  <div id="name-validation"></div>
</div>

<!-- Select Dropdown -->
<select
  class="caxton-select"
  hx-get="/api/templates/preview"
  hx-target="#template-preview"
>
  <option value="">Choose a template...</option>
  <option value="task-automator">Task Automator</option>
  <option value="data-analyzer">Data Analyzer</option>
</select>
```

#### Badges and Tags

```html
<!-- Status Badge -->
<span class="caxton-badge caxton-badge-success">Active</span>
<span class="caxton-badge caxton-badge-warning">Degraded</span>
<span class="caxton-badge caxton-badge-error">Failed</span>

<!-- Capability Tag -->
<span class="caxton-tag">data-analysis</span>
<span class="caxton-tag">report-generation</span>
```

## Molecules - Composite Components

### CLI Molecules

#### Command Input with Validation

```bash
$ caxton agent create
? Agent name: my-analyzer
✓ Name available
? Select template: [Use arrows to navigate]
  > task-automator     - General purpose task automation
    data-analyzer      - Data analysis and visualization
    web-scraper       - Web data extraction
    custom            - Start from scratch
? Workspace: default
```

#### Error Message with Recovery

```bash
✗ Error: Agent 'my-analyzer' already exists

Suggested actions:
  1. Use a different name:
     $ caxton agent create --name my-analyzer-2

  2. Delete the existing agent:
     $ caxton agent delete my-analyzer

  3. Update the existing agent:
     $ caxton agent update my-analyzer

For more help: caxton help agent create
```

#### Progress with Steps

```bash
Creating agent 'my-analyzer'...

[1/5] ✓ Validating configuration
[2/5] ✓ Applying template
[3/5] ◐ Generating files...
      └─ Created: agents/my-analyzer/agent.toml
      └─ Created: agents/my-analyzer/README.md
[4/5] ○ Registering capabilities
[5/5] ○ Starting agent

Time elapsed: 2.3s
```

### Web Molecules

#### Agent Card

```html
<article class="caxton-card caxton-agent-card" data-agent-id="{{agent.id}}">
  <header class="caxton-card-header">
    <h3 class="caxton-agent-name">{{agent.name}}</h3>
    <span class="caxton-badge caxton-badge-{{agent.status}}">
      {{agent.status}}
    </span>
  </header>

  <div class="caxton-card-body">
    <p class="caxton-agent-description">{{agent.description}}</p>

    <div class="caxton-agent-capabilities">
      <span class="caxton-tag">data-analysis</span>
      <span class="caxton-tag">visualization</span>
    </div>
  </div>

  <footer class="caxton-card-footer">
    <button
      class="caxton-btn caxton-btn-sm"
      hx-get="/api/agents/{{agent.id}}/logs"
      hx-target="#log-viewer"
    >
      View Logs
    </button>
    <button
      class="caxton-btn caxton-btn-sm caxton-btn-secondary"
      hx-put="/api/agents/{{agent.id}}/restart"
    >
      Restart
    </button>
  </footer>
</article>
```

#### Form with Real-time Validation

```html
<form
  class="caxton-form"
  hx-post="/api/agents"
  hx-target="#agent-list"
  hx-swap="afterbegin"
>
  <div class="caxton-form-group">
    <label class="caxton-label">Agent Name</label>
    <input
      name="name"
      class="caxton-input"
      required
      hx-post="/api/validate/agent-name"
      hx-trigger="keyup changed delay:500ms"
      hx-target="#name-feedback"
    />
    <div id="name-feedback" class="caxton-feedback"></div>
  </div>

  <div class="caxton-form-group">
    <label class="caxton-label">Template</label>
    <select
      name="template"
      class="caxton-select"
      hx-get="/api/templates/preview"
      hx-target="#template-preview"
    >
      <option value="">Select template...</option>
    </select>
  </div>

  <div id="template-preview" class="caxton-preview"></div>

  <button type="submit" class="caxton-btn caxton-btn-primary">
    Create Agent
  </button>
</form>
```

## Organisms - Complex UI Components

### CLI Organisms

#### Complete Agent Creation Flow

```bash
$ caxton agent create

╔══════════════════════════════════════════════════════════════╗
║  Welcome to Caxton Agent Creator                            ║
║  Create a working agent in under 5 minutes!                 ║
╚══════════════════════════════════════════════════════════════╝

Step 1: Basic Information
─────────────────────────
? Agent name: sales-analyzer
✓ Name available

? Description: Analyzes sales data and generates reports
✓ Description saved

Step 2: Choose Template
───────────────────────
? Select a template to start from:

  ┌─────────────────────────────────────────────────────────┐
  │ ▶ data-analyzer                                        │
  │   Perfect for data analysis and visualization          │
  │   Includes: CSV parser, chart generator, statistics    │
  │   Setup time: ~2 minutes                              │
  └─────────────────────────────────────────────────────────┘

  task-automator
    General purpose task automation

  custom
    Start from scratch (advanced)

✓ Template 'data-analyzer' selected

Step 3: Configuration
─────────────────────
? Customize configuration? (Y/n) Y

Opening configuration in editor...
✓ Configuration saved

Step 4: Review
──────────────
Agent Summary:
  Name:         sales-analyzer
  Template:     data-analyzer
  Capabilities: [data-analysis, report-generation]
  Tools:        [csv_parser, chart_generator, http_client]
  Memory:       Enabled (workspace scope)

? Create agent with these settings? (Y/n) Y

Creating Agent...
─────────────────
[████████████████████████████████████████] 100% | Complete!

✓ Agent 'sales-analyzer' created successfully!

Next Steps:
  1. Start the agent:
     $ caxton agent start sales-analyzer

  2. Test with sample data:
     $ caxton agent test sales-analyzer --sample

  3. View documentation:
     $ caxton agent docs sales-analyzer

Time elapsed: 3.2 seconds
```

#### Terminal Dashboard

```
╔══════════════════════════════════════════════════════════════╗
║  Caxton Server Dashboard                    [R]efresh [Q]uit ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  Server Status                                              ║
║  ─────────────                                              ║
║  Status:     ● Running                                      ║
║  Uptime:     2h 34m 12s                                    ║
║  Version:    1.0.0                                         ║
║  Port:       8080                                          ║
║                                                              ║
║  Active Agents (3)                                          ║
║  ─────────────────                                          ║
║  ● sales-analyzer      [data-analysis]        12 msg/min   ║
║  ● report-generator    [reporting]            8 msg/min    ║
║  ◐ data-fetcher       [data-collection]      Starting...   ║
║                                                              ║
║  System Resources                                           ║
║  ───────────────                                            ║
║  CPU:    [████████░░░░░░░░░░] 42%                         ║
║  Memory: [██████████████░░░░] 73% (1.4GB / 2.0GB)         ║
║  Disk:   [████░░░░░░░░░░░░░░] 23% (4.6GB / 20GB)          ║
║                                                              ║
║  Recent Activity                                            ║
║  ──────────────                                             ║
║  17:23:45  ✓  Agent 'sales-analyzer' processed request     ║
║  17:23:44  ℹ  New connection from 192.168.1.105           ║
║  17:23:42  ⚠  High memory usage detected (>70%)           ║
║  17:23:38  ✓  Configuration reloaded                       ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

### Web Organisms

#### Agent Management Dashboard

```html
<div class="caxton-dashboard">
  <!-- Header with Actions -->
  <header class="caxton-dashboard-header">
    <h1>Agent Management</h1>
    <div class="caxton-actions">
      <button
        class="caxton-btn caxton-btn-primary"
        hx-get="/agents/new"
        hx-target="#modal-container"
      >
        + Create Agent
      </button>
    </div>
  </header>

  <!-- Filter Bar -->
  <div class="caxton-filter-bar">
    <input
      type="search"
      placeholder="Search agents..."
      hx-get="/api/agents/search"
      hx-trigger="keyup changed delay:300ms"
      hx-target="#agent-grid"
    />

    <select hx-get="/api/agents/filter" hx-target="#agent-grid">
      <option value="">All Status</option>
      <option value="running">Running</option>
      <option value="stopped">Stopped</option>
    </select>
  </div>

  <!-- Agent Grid -->
  <div
    id="agent-grid"
    class="caxton-grid"
    hx-get="/api/agents"
    hx-trigger="load, every 5s"
  >
    <!-- Agent cards loaded here -->
  </div>

  <!-- Real-time Status -->
  <div class="caxton-status-bar" hx-sse="connect:/api/events">
    <div hx-sse="message-count">Messages: <span>0</span>/min</div>
    <div hx-sse="agent-count">Active Agents: <span>0</span></div>
  </div>
</div>
```

#### Configuration Editor with Live Validation

```html
<div class="caxton-config-editor">
  <div class="caxton-editor-header">
    <h2>Edit Configuration: {{agent.name}}</h2>
    <div class="caxton-editor-status">
      <span id="save-status">Saved</span>
    </div>
  </div>

  <div class="caxton-editor-container">
    <!-- Line Numbers -->
    <div class="caxton-line-numbers">
      <span>1</span>
      <span>2</span>
      <!-- Generated dynamically -->
    </div>

    <!-- Editor -->
    <textarea
      class="caxton-editor"
      spellcheck="false"
      hx-post="/api/agents/{{agent.id}}/validate"
      hx-trigger="keyup changed delay:1000ms"
      hx-target="#validation-panel"
    >
# Agent Configuration
name = "{{agent.name}}"
version = "1.0.0"

[capabilities]
skills = ["data-analysis", "reporting"]

[memory]
enabled = true
scope = "workspace"
    </textarea>
  </div>

  <!-- Validation Panel -->
  <div id="validation-panel" class="caxton-validation">
    <!-- Live validation results -->
  </div>

  <!-- Actions -->
  <div class="caxton-editor-actions">
    <button
      class="caxton-btn caxton-btn-primary"
      hx-put="/api/agents/{{agent.id}}/config"
      hx-include=".caxton-editor"
    >
      Save & Apply
    </button>
    <button class="caxton-btn caxton-btn-secondary">Discard Changes</button>
  </div>
</div>
```

## Templates - Page Layouts

### CLI Template Patterns

#### Standard Command Output Template

```
╔═══════════════════════════════════════════════════╗
║  {COMMAND TITLE}                                 ║
╚═══════════════════════════════════════════════════╝

{PRIMARY CONTENT}

{SECONDARY INFORMATION}

─────────────────────────────────────────────────────
{STATUS MESSAGE}
{TIMING INFORMATION}
```

#### Interactive Wizard Template

```
Step {N}: {STEP TITLE}
──────────────────────
{PROMPT}

{OPTIONS/INPUT}

{VALIDATION FEEDBACK}

[Previous] [Next] [Cancel]
```

### Web Template Layouts

#### Main Application Layout

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Caxton - {{page.title}}</title>
    <link rel="stylesheet" href="/static/caxton.css" />
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
  </head>
  <body class="caxton-app">
    <!-- Navigation -->
    <nav class="caxton-nav">
      <div class="caxton-nav-brand">
        <a href="/" hx-boost="true">Caxton</a>
      </div>
      <ul class="caxton-nav-menu">
        <li><a href="/agents" hx-boost="true">Agents</a></li>
        <li><a href="/tools" hx-boost="true">Tools</a></li>
        <li><a href="/memory" hx-boost="true">Memory</a></li>
        <li><a href="/docs" hx-boost="true">Docs</a></li>
      </ul>
    </nav>

    <!-- Main Content -->
    <main class="caxton-main">
      <div class="caxton-container">{{content}}</div>
    </main>

    <!-- Footer -->
    <footer class="caxton-footer">
      <div class="caxton-footer-content">
        <span>Caxton v{{version}}</span>
        <span>{{server.status}}</span>
      </div>
    </footer>

    <!-- Modal Container -->
    <div id="modal-container"></div>

    <!-- Toast Container -->
    <div id="toast-container" class="caxton-toasts"></div>
  </body>
</html>
```

#### Dashboard Grid Layout

```html
<div class="caxton-dashboard-grid">
  <!-- Sidebar -->
  <aside class="caxton-sidebar">
    <nav class="caxton-sidebar-nav">
      <!-- Navigation items -->
    </nav>
  </aside>

  <!-- Main Content Area -->
  <div class="caxton-content">
    <!-- Page Header -->
    <header class="caxton-page-header">
      <h1>{{page.title}}</h1>
      <div class="caxton-breadcrumb">
        <!-- Breadcrumb navigation -->
      </div>
    </header>

    <!-- Content Grid -->
    <div class="caxton-grid-layout">
      <!-- Dynamic content -->
    </div>
  </div>

  <!-- Right Panel (optional) -->
  <aside class="caxton-panel">
    <!-- Contextual information -->
  </aside>
</div>
```

## Pages - Complete User Experiences

### Onboarding Experience (0-10 minutes)

#### Minute 0-1: Installation

```bash
# Terminal shows clear progress with timing
$ curl -sSf https://caxton.dev/install.sh | sh

Installing Caxton...
[████████████████████████████████████████] 100%

✓ Caxton installed successfully!
  Version: 1.0.0
  Location: /usr/local/bin/caxton

Ready to create your first agent? Run:
  $ caxton quickstart
```

#### Minutes 1-5: First Agent Creation

```bash
$ caxton quickstart

Welcome to Caxton! Let's create your first agent in under 5 minutes.

Step 1: Choose Your First Agent Type
────────────────────────────────────
What would you like your agent to do?

  1. Analyze data and create reports
  2. Automate repetitive tasks
  3. Monitor systems and alert on issues
  4. Process and transform documents
  5. Something else (custom)

Your choice: 1

Great choice! The data analyzer template is perfect for beginners.

Step 2: Name Your Agent
───────────────────────
? Agent name (e.g., sales-analyzer): my-first-agent
✓ Perfect! 'my-first-agent' is available.

Step 3: Quick Configuration
───────────────────────────
We'll use smart defaults. You can customize later.

  ✓ Memory enabled (remembers past analyses)
  ✓ Basic tools included (CSV, JSON, charts)
  ✓ Workspace isolation (safe experimentation)

Creating your agent...
[████████████████████████████████████████] 100%

🎉 Success! Your first agent is ready!

Step 4: Test Your Agent
───────────────────────
Let's verify everything works:

$ caxton agent start my-first-agent
● Agent 'my-first-agent' is running

$ caxton agent test my-first-agent
Sending test message... ✓
Response received in 234ms
Agent is working perfectly!

Congratulations! You've created a working Caxton agent in 3 minutes!

What's Next?
───────────
1. Explore the agent configuration:
   $ caxton agent edit my-first-agent

2. Create another agent:
   $ caxton agent create

3. Learn about agent collaboration:
   $ caxton docs collaboration

4. Join the community:
   https://discord.gg/caxton
```

### Agent Management Page

```html
<!-- Complete agent management interface -->
<div class="caxton-page" data-page="agents">
  <!-- Page Header -->
  <header class="caxton-page-header">
    <div class="caxton-header-content">
      <h1>Agents</h1>
      <p class="caxton-subtitle">Manage your configuration-driven agents</p>
    </div>
    <div class="caxton-header-actions">
      <button
        class="caxton-btn caxton-btn-primary"
        onclick="showCreateWizard()"
      >
        <span class="caxton-icon">+</span>
        Create Agent
      </button>
    </div>
  </header>

  <!-- Stats Bar -->
  <div class="caxton-stats-bar">
    <div class="caxton-stat">
      <span class="caxton-stat-value">12</span>
      <span class="caxton-stat-label">Total Agents</span>
    </div>
    <div class="caxton-stat">
      <span class="caxton-stat-value">8</span>
      <span class="caxton-stat-label">Running</span>
    </div>
    <div class="caxton-stat">
      <span class="caxton-stat-value">1,234</span>
      <span class="caxton-stat-label">Messages/hour</span>
    </div>
  </div>

  <!-- Filter and Search -->
  <div class="caxton-controls">
    <input
      type="search"
      class="caxton-search"
      placeholder="Search agents..."
      hx-get="/api/agents/search"
      hx-trigger="keyup changed delay:300ms"
      hx-target="#agent-list"
    />

    <div class="caxton-filters">
      <button class="caxton-filter active">All</button>
      <button class="caxton-filter">Running</button>
      <button class="caxton-filter">Stopped</button>
      <button class="caxton-filter">Failed</button>
    </div>
  </div>

  <!-- Agent List/Grid -->
  <div
    id="agent-list"
    class="caxton-agent-list"
    hx-get="/api/agents"
    hx-trigger="load, every 5s"
  >
    <!-- Agent cards render here -->
  </div>
</div>
```

## Interaction Patterns

### HTMX-Based Interactions

#### Progressive Enhancement Strategy

```html
<!-- Form works without JavaScript -->
<form action="/api/agents" method="POST">
  <input name="name" required />
  <button type="submit">Create</button>
</form>

<!-- Enhanced with HTMX -->
<form
  hx-post="/api/agents"
  hx-target="#agent-list"
  hx-swap="afterbegin"
  hx-indicator="#spinner"
>
  <input name="name" required />
  <button type="submit">Create</button>
</form>
```

#### Real-time Updates

```html
<!-- Server-Sent Events for live updates -->
<div hx-sse="connect:/api/events" hx-sse="agent-status">
  <div id="agent-status-container">
    <!-- Updates stream here -->
  </div>
</div>

<!-- Polling for simpler cases -->
<div hx-get="/api/agents/status" hx-trigger="every 2s" hx-target="this">
  <!-- Status refreshes every 2 seconds -->
</div>
```

#### Optimistic UI Updates

```html
<!-- Immediate feedback before server response -->
<button
  hx-post="/api/agents/start"
  hx-target="#status"
  hx-swap="innerHTML"
  hx-on="htmx:beforeRequest: this.disabled=true;
         document.getElementById('status').innerHTML='Starting...'"
>
  Start Agent
</button>
```

### CLI Interaction Patterns

#### Command Acknowledgment (<100ms)

```bash
# Every command acknowledges immediately
$ caxton agent start my-analyzer
◐ Starting agent 'my-analyzer'...  # Appears within 100ms
✓ Agent started successfully        # Final status
```

#### Progressive Disclosure

```bash
# Simple by default
$ caxton agent create my-agent
✓ Agent created with default settings

# Detailed when requested
$ caxton agent create my-agent --verbose
Creating agent 'my-agent'...
  → Validating name... OK
  → Loading template 'default'... OK
  → Creating directory structure... OK
  → Writing configuration... OK
  → Registering with server... OK
✓ Agent created successfully
```

#### Error Recovery Guidance

```bash
# Actionable error messages
$ caxton agent start my-agent
✗ Error: Agent configuration invalid

Issues found:
  Line 12: Missing required field 'capabilities'
  Line 18: Invalid tool reference 'undefined_tool'

To fix:
  1. Run: caxton agent validate my-agent --fix
  2. Or edit: caxton agent edit my-agent

Example valid configuration:
  capabilities = ["data-analysis", "reporting"]
  tools = ["csv_parser", "http_client"]
```

## Accessibility Requirements

### WCAG 2.1 AA Compliance

#### Color Contrast

- Normal text: 4.5:1 contrast ratio minimum
- Large text: 3:1 contrast ratio minimum
- Interactive elements: Clear focus indicators
- Never rely solely on color to convey information

#### Keyboard Navigation

```html
<!-- All interactive elements keyboard accessible -->
<div
  role="button"
  tabindex="0"
  onkeydown="if(event.key === 'Enter') handleClick()"
  onclick="handleClick()"
>
  Clickable Element
</div>

<!-- Skip links for navigation -->
<a href="#main-content" class="caxton-skip-link"> Skip to main content </a>
```

#### Screen Reader Support

```html
<!-- Semantic HTML -->
<nav aria-label="Main navigation">...</nav>
<main role="main">...</main>
<aside aria-label="Agent details">...</aside>

<!-- ARIA labels for dynamic content -->
<div role="status" aria-live="polite" aria-label="Agent status updates">
  <!-- Live updates announced to screen readers -->
</div>

<!-- Loading states -->
<div aria-busy="true" aria-label="Loading agents">
  <span class="spinner" aria-hidden="true"></span>
  Loading...
</div>
```

#### CLI Accessibility

```bash
# Alternative output formats
$ caxton agent list --format=plain
NAME            STATUS    CAPABILITIES
my-analyzer     running   data-analysis, reporting
data-fetcher    stopped   data-collection

# Verbose mode for screen readers
$ caxton agent status --accessible
Agent Status Report:
  Agent name: my-analyzer
  Current status: running
  Uptime: 2 hours 15 minutes
  Messages processed: 145
  Last activity: 30 seconds ago
```

## Performance Budgets

### Response Time Targets

```yaml
performance_targets:
  cli:
    command_acknowledgment: < 100ms # User sees immediate feedback
    simple_operations: < 500ms # List, status, etc.
    complex_operations: < 3s # Create, deploy, etc.

  web:
    first_contentful_paint: < 1.5s
    time_to_interactive: < 3.5s
    largest_contentful_paint: < 2.5s
    cumulative_layout_shift: < 0.1

  api:
    simple_queries: < 50ms
    complex_queries: < 200ms
    mutations: < 500ms
```

### Resource Budgets

```yaml
resource_budgets:
  javascript:
    htmx: ~14kb (gzipped)
    custom: < 10kb
    total: < 25kb

  css:
    framework: 0kb (no framework)
    custom: < 20kb
    total: < 20kb

  images:
    icons: SVG only
    logos: < 10kb each

  fonts:
    system_fonts: preferred
    custom_fonts: 1 maximum (variable font)
```

## Responsive Design

### Breakpoints

```css
/* Mobile-first approach */
:root {
  --breakpoint-sm: 640px; /* Small tablets */
  --breakpoint-md: 768px; /* Tablets */
  --breakpoint-lg: 1024px; /* Desktop */
  --breakpoint-xl: 1280px; /* Wide desktop */
}

/* Responsive Grid */
.caxton-grid {
  display: grid;
  gap: var(--spacing-4);
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
}

@media (max-width: 768px) {
  .caxton-grid {
    grid-template-columns: 1fr;
  }
}
```

### Mobile Adaptations

```css
/* Touch-friendly targets */
@media (hover: none) {
  .caxton-btn {
    min-height: 44px; /* iOS touch target */
    min-width: 44px;
  }

  .caxton-link {
    padding: var(--spacing-2);
  }
}

/* Simplified mobile navigation */
@media (max-width: 768px) {
  .caxton-nav {
    position: fixed;
    bottom: 0;
    width: 100%;
  }
}
```

## Implementation Examples

### Complete Agent Creation Flow

```html
<!-- Wizard-style agent creation -->
<div class="caxton-wizard" data-step="1">
  <!-- Step 1: Basic Info -->
  <div class="caxton-wizard-step" data-step-id="1">
    <h2>Let's create your agent</h2>
    <form hx-post="/api/agents/validate-name">
      <input
        name="name"
        placeholder="Agent name"
        required
        hx-trigger="keyup changed delay:500ms"
      />
      <button type="button" onclick="nextStep()" disabled>Next →</button>
    </form>
  </div>

  <!-- Step 2: Template Selection -->
  <div class="caxton-wizard-step hidden" data-step-id="2">
    <h2>Choose a template</h2>
    <div class="caxton-template-grid">
      <!-- Template cards with preview -->
    </div>
  </div>

  <!-- Step 3: Configuration -->
  <div class="caxton-wizard-step hidden" data-step-id="3">
    <h2>Customize your agent</h2>
    <!-- Configuration editor -->
  </div>

  <!-- Step 4: Review & Create -->
  <div class="caxton-wizard-step hidden" data-step-id="4">
    <h2>Review and create</h2>
    <!-- Summary and create button -->
  </div>
</div>
```

## Success Patterns

### Onboarding Milestones

```yaml
success_milestones:
  minute_1:
    - Caxton installed
    - First command executed
    - Help accessed

  minute_5:
    - First agent created
    - Template applied
    - Configuration understood

  minute_10:
    - Agent running
    - First message processed
    - Success celebrated
```

### Celebration Moments

```bash
# First agent created
🎉 Congratulations! You've created your first Caxton agent!
   Time: 2m 34s (Well under our 5-minute promise!)

# First successful message
✨ Success! Your agent just processed its first message!
   Response time: 127ms

# Milestone achievements
🏆 Achievement Unlocked: Speed Demon
   Created 5 agents in under 10 minutes!
```

## Summary

This design system provides a complete foundation for Caxton's user interfaces
across CLI and web platforms. By following these patterns, we ensure:

1. **Rapid Onboarding**: 5-10 minute path to success
2. **Consistency**: Unified experience across all touchpoints
3. **Accessibility**: WCAG 2.1 AA compliance throughout
4. **Performance**: Sub-100ms CLI feedback, fast web interactions
5. **Progressive Enhancement**: Works without JavaScript, better with HTMX
6. **Developer Joy**: Clear patterns, helpful errors, celebration moments

The Atomic Design methodology ensures that every interface element, from
simple status indicators to complete workflows, maintains consistency and
supports the core mission of making agent creation accessible to all
developers.
