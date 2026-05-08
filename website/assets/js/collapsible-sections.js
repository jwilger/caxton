/**
 * Collapsible Sections for API Documentation
 * Makes h2 sections collapsible with expand/collapse functionality
 * Includes localStorage persistence and accessibility features
 */
(function () {
  "use strict";

  const CONFIG = {
    // Selectors
    sectionSelector: ".docs-content h2, .docs-body h2",
    contentSelector: ".docs-content, .docs-body",
    storageKey: "caxton-api-collapsed-sections",

    // CSS Classes
    classes: {
      collapsible: "collapsible-section",
      collapsed: "collapsed",
      toggleButton: "section-toggle",
      expandAllBtn: "expand-all-btn",
      collapseAllBtn: "collapse-all-btn",
      controlsWrapper: "collapsible-controls",
    },

    // Animation timing
    animationDuration: 300,
  };

  class CollapsibleSections {
    constructor() {
      this.sections = [];
      this.collapsedSections = this.loadCollapsedState();
      this.init();
    }

    init() {
      // Only initialize on API reference page
      if (!this.isApiReferencePage()) {
        return;
      }

      this.addStyles();
      this.createSections();
      this.addControls();
      this.bindEvents();
      this.restoreCollapsedState();

      console.log(
        "Collapsible sections initialized with",
        this.sections.length,
        "sections",
      );
    }

    isApiReferencePage() {
      return (
        window.location.pathname.includes("api-reference") ||
        document.title.toLowerCase().includes("api reference") ||
        document
          .querySelector("h1")
          ?.textContent?.toLowerCase()
          .includes("api reference")
      );
    }

    addStyles() {
      const styles = `
                .collapsible-controls {
                    display: flex;
                    gap: 12px;
                    margin-bottom: 24px;
                    padding: 16px;
                    background: var(--bg-tertiary, #2a2e3a);
                    border: 1px solid var(--color-surface1, #414559);
                    border-radius: var(--radius-lg, 12px);
                    box-shadow: var(--shadow-sm, 0 1px 3px rgba(0,0,0,0.1));
                }

                .collapsible-btn {
                    padding: 8px 16px;
                    background: var(--color-primary, #89b4fa);
                    color: var(--text-on-primary, #11111b);
                    border: none;
                    border-radius: var(--radius-md, 8px);
                    font-size: var(--font-size-sm, 14px);
                    font-weight: var(--font-medium, 500);
                    cursor: pointer;
                    transition: all var(--transition-fast, 0.15s ease);
                    min-width: 120px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 6px;
                    text-transform: none;
                }

                .collapsible-btn:hover {
                    background: var(--color-lavender, #b4befe);
                    transform: translateY(-1px);
                    box-shadow: var(--shadow-md, 0 4px 6px rgba(0,0,0,0.1));
                }

                .collapsible-btn:active {
                    transform: translateY(0);
                }

                .collapsible-btn:focus {
                    outline: 2px solid var(--color-primary, #89b4fa);
                    outline-offset: 2px;
                }

                .collapsible-section {
                    position: relative;
                }

                .section-toggle {
                    position: absolute;
                    left: -40px;
                    top: 50%;
                    transform: translateY(-50%);
                    width: 32px;
                    height: 32px;
                    border: none;
                    background: var(--bg-tertiary, #2a2e3a);
                    color: var(--color-primary, #89b4fa);
                    border-radius: 50%;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                    transition: all var(--transition-fast, 0.15s ease);
                    z-index: 10;
                    box-shadow: var(--shadow-sm, 0 1px 3px rgba(0,0,0,0.1));
                    border: 2px solid var(--color-surface1, #414559);
                    opacity: 0;
                    visibility: hidden;
                    line-height: 1;
                }

                .collapsible-section:hover .section-toggle {
                    opacity: 1;
                    visibility: visible;
                }

                .section-toggle:hover {
                    background: var(--color-primary, #89b4fa);
                    color: var(--text-on-primary, #11111b);
                    transform: translateY(-50%) scale(1.1);
                    box-shadow: var(--shadow-md, 0 4px 6px rgba(0,0,0,0.15));
                }

                .section-toggle:focus {
                    opacity: 1;
                    visibility: visible;
                    outline: 2px solid var(--color-lavender, #b4befe);
                    outline-offset: 2px;
                }

                .section-toggle:active {
                    transform: translateY(-50%) scale(0.95);
                }

                .collapsible-content {
                    overflow: hidden;
                    transition: max-height ${CONFIG.animationDuration}ms cubic-bezier(0.4, 0, 0.2, 1),
                                opacity ${CONFIG.animationDuration}ms ease;
                }

                .collapsed .collapsible-content {
                    max-height: 0 !important;
                    opacity: 0.3;
                }

                .collapsed .section-toggle::before {
                    content: '+';
                }

                .section-toggle::before {
                    content: '−';
                }

                .collapsed h2 {
                    opacity: 0.7;
                    cursor: pointer;
                }

                .collapsed h2:hover {
                    opacity: 1;
                }

                /* Mobile optimizations */
                @media (max-width: 768px) {
                    .section-toggle {
                        left: -36px;
                        width: 28px;
                        height: 28px;
                        font-size: 14px;
                        opacity: 1;
                        visibility: visible;
                    }

                    .collapsible-controls {
                        flex-direction: column;
                        gap: 8px;
                    }

                    .collapsible-btn {
                        min-width: unset;
                        width: 100%;
                    }
                }

                /* High contrast mode support */
                @media (prefers-contrast: high) {
                    .section-toggle {
                        border-width: 3px;
                    }

                    .collapsible-btn {
                        border: 2px solid currentColor;
                    }
                }

                /* Reduced motion support */
                @media (prefers-reduced-motion: reduce) {
                    .collapsible-content,
                    .section-toggle,
                    .collapsible-btn {
                        transition: none;
                    }
                }

                /* Dark mode adjustments */
                @media (prefers-color-scheme: dark) {
                    .section-toggle {
                        box-shadow: 0 2px 8px rgba(0,0,0,0.3);
                    }
                }
            `;

      const styleSheet = document.createElement("style");
      styleSheet.textContent = styles;
      document.head.appendChild(styleSheet);
    }

    createSections() {
      const headers = document.querySelectorAll(CONFIG.sectionSelector);

      headers.forEach((header, index) => {
        const section = this.createSection(header, index);
        if (section) {
          this.sections.push(section);
        }
      });
    }

    createSection(header, index) {
      const sectionId = this.generateSectionId(header, index);
      const content = this.collectSectionContent(header);

      if (!content.length) {
        return null;
      }

      // Create wrapper for the section
      const wrapper = document.createElement("div");
      wrapper.className = CONFIG.classes.collapsible;
      wrapper.setAttribute("data-section-id", sectionId);

      // Create toggle button
      const toggleButton = this.createToggleButton(sectionId);

      // Create content wrapper
      const contentWrapper = document.createElement("div");
      contentWrapper.className = "collapsible-content";

      // Wrap the header and content
      header.parentNode.insertBefore(wrapper, header);
      wrapper.appendChild(header);
      header.appendChild(toggleButton);

      content.forEach((element) => {
        contentWrapper.appendChild(element);
      });
      wrapper.appendChild(contentWrapper);

      return {
        id: sectionId,
        element: wrapper,
        header: header,
        content: contentWrapper,
        toggle: toggleButton,
        isCollapsed: false,
      };
    }

    generateSectionId(header, index) {
      const text = header.textContent
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9\s]/g, "")
        .replace(/\s+/g, "-");
      return `section-${index}-${text}`;
    }

    collectSectionContent(header) {
      const content = [];
      let nextElement = header.nextElementSibling;

      while (nextElement && !this.isHeader(nextElement)) {
        const elementToMove = nextElement;
        nextElement = nextElement.nextElementSibling;
        content.push(elementToMove);
      }

      return content;
    }

    isHeader(element) {
      return element.tagName && element.tagName.match(/^H[1-6]$/);
    }

    createToggleButton(sectionId) {
      const button = document.createElement("button");
      button.className = CONFIG.classes.toggleButton;
      button.setAttribute("aria-expanded", "true");
      button.setAttribute("aria-controls", sectionId + "-content");
      button.title = "Toggle section";
      button.type = "button";

      return button;
    }

    addControls() {
      const contentContainer = document.querySelector(CONFIG.contentSelector);
      if (!contentContainer || this.sections.length === 0) {
        return;
      }

      const controlsWrapper = document.createElement("div");
      controlsWrapper.className = CONFIG.classes.controlsWrapper;
      controlsWrapper.innerHTML = `
                <button class="${CONFIG.classes.expandAllBtn} collapsible-btn" type="button"
                        aria-label="Expand all sections">
                    <span>📖</span> Expand All
                </button>
                <button class="${CONFIG.classes.collapseAllBtn} collapsible-btn" type="button"
                        aria-label="Collapse all sections">
                    <span>📚</span> Collapse All
                </button>
                <span style="color: var(--text-muted, #6c7086); font-size: var(--font-size-sm, 14px); margin-left: auto;">
                    ${this.sections.length} sections
                </span>
            `;

      // Insert controls before the first section
      const firstSection = this.sections[0].element;
      firstSection.parentNode.insertBefore(controlsWrapper, firstSection);
    }

    bindEvents() {
      // Individual section toggles
      this.sections.forEach((section) => {
        section.toggle.addEventListener("click", (e) => {
          e.stopPropagation();
          this.toggleSection(section);
        });

        // Also allow clicking on header to toggle
        section.header.addEventListener("click", (e) => {
          if (section.isCollapsed) {
            this.toggleSection(section);
          }
        });
      });

      // Expand/Collapse all buttons
      const expandAllBtn = document.querySelector(
        `.${CONFIG.classes.expandAllBtn}`,
      );
      const collapseAllBtn = document.querySelector(
        `.${CONFIG.classes.collapseAllBtn}`,
      );

      if (expandAllBtn) {
        expandAllBtn.addEventListener("click", () => this.expandAll());
      }

      if (collapseAllBtn) {
        collapseAllBtn.addEventListener("click", () => this.collapseAll());
      }

      // Keyboard navigation
      document.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
          this.expandAll();
        }
      });

      // Save state when page is unloaded
      window.addEventListener("beforeunload", () => {
        this.saveCollapsedState();
      });
    }

    toggleSection(section) {
      const isCurrentlyCollapsed = section.isCollapsed;

      if (isCurrentlyCollapsed) {
        this.expandSection(section);
      } else {
        this.collapseSection(section);
      }

      this.saveCollapsedState();
    }

    collapseSection(section) {
      const content = section.content;
      const currentHeight = content.scrollHeight;

      // Set initial height
      content.style.maxHeight = currentHeight + "px";

      // Trigger reflow
      content.offsetHeight;

      // Collapse
      requestAnimationFrame(() => {
        section.element.classList.add(CONFIG.classes.collapsed);
        content.style.maxHeight = "0px";
        section.toggle.setAttribute("aria-expanded", "false");
        section.isCollapsed = true;
      });
    }

    expandSection(section) {
      const content = section.content;

      section.element.classList.remove(CONFIG.classes.collapsed);
      content.style.maxHeight = content.scrollHeight + "px";
      section.toggle.setAttribute("aria-expanded", "true");
      section.isCollapsed = false;

      // Reset max-height after animation
      setTimeout(() => {
        if (!section.isCollapsed) {
          content.style.maxHeight = "none";
        }
      }, CONFIG.animationDuration);
    }

    expandAll() {
      this.sections.forEach((section) => {
        if (section.isCollapsed) {
          this.expandSection(section);
        }
      });
      this.saveCollapsedState();
    }

    collapseAll() {
      this.sections.forEach((section) => {
        if (!section.isCollapsed) {
          this.collapseSection(section);
        }
      });
      this.saveCollapsedState();
    }

    saveCollapsedState() {
      const collapsedIds = this.sections
        .filter((section) => section.isCollapsed)
        .map((section) => section.id);

      try {
        localStorage.setItem(CONFIG.storageKey, JSON.stringify(collapsedIds));
      } catch (error) {
        console.warn("Could not save collapsed state to localStorage:", error);
      }
    }

    loadCollapsedState() {
      try {
        const saved = localStorage.getItem(CONFIG.storageKey);
        return saved ? JSON.parse(saved) : [];
      } catch (error) {
        console.warn(
          "Could not load collapsed state from localStorage:",
          error,
        );
        return [];
      }
    }

    restoreCollapsedState() {
      this.collapsedSections.forEach((sectionId) => {
        const section = this.sections.find((s) => s.id === sectionId);
        if (section && !section.isCollapsed) {
          // Use a timeout to ensure DOM is ready
          setTimeout(() => {
            this.collapseSection(section);
          }, 50);
        }
      });
    }
  }

  // Initialize when DOM is ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
      new CollapsibleSections();
    });
  } else {
    new CollapsibleSections();
  }

  // Also make it available globally for debugging
  window.CollapsibleSections = CollapsibleSections;
})();
