// Anchor Links for Long Documents
// Provides deep-linking, smooth scrolling, and copy-to-clipboard functionality

class AnchorLinks {
    constructor() {
        this.headingSelectors = 'h2, h3, h4, h5, h6';
        this.anchorClass = 'anchor-link';
        this.anchorIconClass = 'anchor-icon';
        this.copiedClass = 'anchor-copied';

        this.init();
    }

    init() {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => this.setup());
        } else {
            this.setup();
        }
    }

    setup() {
        this.addCSS();
        this.generateAnchors();
        this.setupSmoothScrolling();
        this.updateTableOfContents();
        this.setupCopyToClipboard();

        // Handle direct navigation to anchors (e.g., from bookmarks)
        this.handleInitialAnchor();
    }

    addCSS() {
        const style = document.createElement('style');
        style.textContent = `
            /* Anchor link styles */
            .anchor-link {
                position: absolute;
                left: -2rem;
                top: 50%;
                transform: translateY(-50%);
                width: 1.5rem;
                height: 1.5rem;
                display: flex;
                align-items: center;
                justify-content: center;
                opacity: 0;
                visibility: hidden;
                transition: all var(--transition-fast, 0.2s ease);
                border-radius: var(--radius-sm, 0.25rem);
                background: var(--bg-tertiary, rgba(255, 255, 255, 0.1));
                border: 1px solid var(--color-surface1, rgba(255, 255, 255, 0.2));
                text-decoration: none;
                cursor: pointer;
                user-select: none;
                z-index: 10;
            }

            .anchor-link:hover {
                background: var(--color-primary, #89b4fa);
                border-color: var(--color-primary, #89b4fa);
                transform: translateY(-50%) scale(1.1);
                box-shadow: var(--shadow-sm, 0 2px 4px rgba(0, 0, 0, 0.1));
            }

            .anchor-link:focus {
                outline: 2px solid var(--color-primary, #89b4fa);
                outline-offset: 2px;
                opacity: 1;
                visibility: visible;
            }

            .anchor-icon {
                font-size: 0.875rem;
                color: var(--text-secondary, rgba(255, 255, 255, 0.7));
                transition: color var(--transition-fast, 0.2s ease);
            }

            .anchor-link:hover .anchor-icon {
                color: white;
            }

            /* Show anchor link on heading hover */
            h1:hover .anchor-link,
            h2:hover .anchor-link,
            h3:hover .anchor-link,
            h4:hover .anchor-link,
            h5:hover .anchor-link,
            h6:hover .anchor-link,
            .anchor-link:hover,
            .anchor-link:focus {
                opacity: 1;
                visibility: visible;
            }

            /* Adjust heading positioning for anchor links */
            h1, h2, h3, h4, h5, h6 {
                position: relative;
            }

            /* Special styling for docs content */
            .docs-content h2,
            .docs-content h3,
            .docs-content h4 {
                scroll-margin-top: 6rem;
            }

            /* Copy feedback styling */
            .anchor-copied {
                background: var(--color-green, #a6e3a1) !important;
                border-color: var(--color-green, #a6e3a1) !important;
            }

            .anchor-copied .anchor-icon {
                color: var(--text-primary, white) !important;
            }

            /* Mobile responsiveness */
            @media (max-width: 768px) {
                .anchor-link {
                    left: -1.5rem;
                    width: 1.25rem;
                    height: 1.25rem;
                }

                .anchor-icon {
                    font-size: 0.75rem;
                }

                /* Always show on mobile for better accessibility */
                h1 .anchor-link,
                h2 .anchor-link,
                h3 .anchor-link,
                h4 .anchor-link,
                h5 .anchor-link,
                h6 .anchor-link {
                    opacity: 0.6;
                    visibility: visible;
                }
            }

            /* Smooth scroll behavior */
            html {
                scroll-behavior: smooth;
            }

            @media (prefers-reduced-motion: reduce) {
                html {
                    scroll-behavior: auto;
                }

                .anchor-link {
                    transition: none;
                }
            }

            /* Table of contents enhancement */
            .toc-anchor {
                color: var(--color-primary, #89b4fa);
                text-decoration: none;
                transition: color var(--transition-fast, 0.2s ease);
            }

            .toc-anchor:hover {
                color: var(--color-lavender, #cba6f7);
                text-decoration: underline;
            }

            .toc-list {
                list-style: none;
                padding-left: 0;
                margin: var(--space-4, 1rem) 0;
            }

            .toc-list ul {
                list-style: none;
                padding-left: var(--space-6, 1.5rem);
                margin: var(--space-2, 0.5rem) 0;
            }

            .toc-list li {
                margin: var(--space-2, 0.5rem) 0;
                position: relative;
            }

            .toc-list > li::before {
                content: '§';
                position: absolute;
                left: -1rem;
                color: var(--color-primary, #89b4fa);
                opacity: 0.6;
            }

            .toc-list ul li::before {
                content: '▸';
                position: absolute;
                left: -1rem;
                color: var(--color-lavender, #cba6f7);
                opacity: 0.6;
                font-size: 0.875rem;
            }
        `;
        document.head.appendChild(style);
    }

    generateAnchors() {
        const headings = document.querySelectorAll(this.headingSelectors);

        headings.forEach((heading, index) => {
            // Generate ID if it doesn't exist
            if (!heading.id) {
                heading.id = this.generateId(heading.textContent, index);
            }

            // Create anchor link
            const anchor = document.createElement('a');
            anchor.href = `#${heading.id}`;
            anchor.className = this.anchorClass;
            anchor.setAttribute('aria-label', `Link to ${heading.textContent}`);
            anchor.setAttribute('tabindex', '0');

            // Create icon
            const icon = document.createElement('span');
            icon.className = this.anchorIconClass;
            icon.innerHTML = '#'; // Using # symbol, can be changed to 🔗 if preferred
            icon.setAttribute('aria-hidden', 'true');

            anchor.appendChild(icon);
            heading.appendChild(anchor);
        });
    }

    generateId(text, fallbackIndex) {
        return text
            .toLowerCase()
            .trim()
            .replace(/[^\w\s-]/g, '') // Remove special characters except spaces and hyphens
            .replace(/\s+/g, '-') // Replace spaces with hyphens
            .replace(/-+/g, '-') // Replace multiple hyphens with single hyphen
            .replace(/^-|-$/g, '') // Remove leading/trailing hyphens
            || `heading-${fallbackIndex}`; // Fallback if text is empty
    }

    setupSmoothScrolling() {
        // Enhanced smooth scrolling with offset for fixed headers
        document.addEventListener('click', (e) => {
            const anchor = e.target.closest(`.${this.anchorClass}`);
            if (!anchor) return;

            e.preventDefault();

            const targetId = anchor.getAttribute('href').substring(1);
            const targetElement = document.getElementById(targetId);

            if (targetElement) {
                this.scrollToElement(targetElement);

                // Update URL without triggering scroll
                window.history.pushState(null, null, `#${targetId}`);

                // Focus management for accessibility
                targetElement.setAttribute('tabindex', '-1');
                targetElement.focus();

                // Remove tabindex after focus to avoid interfering with normal tab flow
                setTimeout(() => {
                    targetElement.removeAttribute('tabindex');
                }, 100);
            }
        });

        // Handle browser back/forward
        window.addEventListener('popstate', () => {
            if (window.location.hash) {
                const targetId = window.location.hash.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    this.scrollToElement(targetElement);
                }
            }
        });
    }

    scrollToElement(element) {
        const headerHeight = this.getHeaderHeight();
        const elementTop = element.getBoundingClientRect().top + window.pageYOffset;
        const offsetTop = elementTop - headerHeight - 20; // 20px extra padding

        window.scrollTo({
            top: Math.max(0, offsetTop),
            behavior: 'smooth'
        });
    }

    getHeaderHeight() {
        const header = document.querySelector('header, .navbar, .docs-header');
        return header ? header.offsetHeight : 0;
    }

    setupCopyToClipboard() {
        document.addEventListener('click', (e) => {
            const anchor = e.target.closest(`.${this.anchorClass}`);
            if (!anchor) return;

            // Right-click or Ctrl+click to copy URL
            if (e.button === 2 || e.ctrlKey || e.metaKey) {
                e.preventDefault();
                this.copyToClipboard(window.location.origin + window.location.pathname + anchor.getAttribute('href'));
                this.showCopyFeedback(anchor);
                return;
            }
        });

        // Also handle context menu
        document.addEventListener('contextmenu', (e) => {
            const anchor = e.target.closest(`.${this.anchorClass}`);
            if (anchor) {
                e.preventDefault();
                this.copyToClipboard(window.location.origin + window.location.pathname + anchor.getAttribute('href'));
                this.showCopyFeedback(anchor);
            }
        });
    }

    async copyToClipboard(text) {
        try {
            if (navigator.clipboard && window.isSecureContext) {
                await navigator.clipboard.writeText(text);
                this.showToast('Link copied to clipboard!');
            } else {
                // Fallback for older browsers
                const textArea = document.createElement('textarea');
                textArea.value = text;
                textArea.style.position = 'fixed';
                textArea.style.left = '-9999px';
                document.body.appendChild(textArea);
                textArea.focus();
                textArea.select();
                document.execCommand('copy');
                document.body.removeChild(textArea);
                this.showToast('Link copied to clipboard!');
            }
        } catch (err) {
            console.warn('Failed to copy to clipboard:', err);
            this.showToast('Failed to copy link', 'error');
        }
    }

    showCopyFeedback(anchor) {
        anchor.classList.add(this.copiedClass);
        const icon = anchor.querySelector(`.${this.anchorIconClass}`);
        if (icon) {
            const originalText = icon.innerHTML;
            icon.innerHTML = '✓';

            setTimeout(() => {
                anchor.classList.remove(this.copiedClass);
                icon.innerHTML = originalText;
            }, 1500);
        }
    }

    showToast(message, type = 'success') {
        // Remove any existing toasts
        const existingToasts = document.querySelectorAll('.anchor-copy-toast');
        existingToasts.forEach(toast => toast.remove());

        // Create new toast
        const toast = document.createElement('div');
        toast.className = 'anchor-copy-toast';
        toast.textContent = message;

        if (type === 'error') {
            toast.style.background = 'var(--color-red, #f38ba8)';
        }

        document.body.appendChild(toast);

        // Show toast
        setTimeout(() => {
            toast.classList.add('show');
        }, 10);

        // Hide and remove toast
        setTimeout(() => {
            toast.classList.remove('show');
            setTimeout(() => {
                toast.remove();
            }, 300);
        }, 2500);
    }

    handleInitialAnchor() {
        if (window.location.hash) {
            // Small delay to ensure page is fully loaded
            setTimeout(() => {
                const targetId = window.location.hash.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    this.scrollToElement(targetElement);
                }
            }, 100);
        }
    }

    updateTableOfContents() {
        // Look for existing table of contents containers
        const tocContainers = document.querySelectorAll('.table-of-contents, .toc, #toc');

        if (tocContainers.length === 0) {
            // Auto-generate TOC if there's a suitable place for it
            this.autoGenerateTableOfContents();
            return;
        }

        tocContainers.forEach(container => {
            this.generateTableOfContents(container);
        });
    }

    autoGenerateTableOfContents() {
        const headings = document.querySelectorAll(this.headingSelectors);
        if (headings.length < 3) return; // Don't create TOC for short documents

        // Look for a good place to insert TOC
        const docBody = document.querySelector('.docs-body, .docs-content, main, article');
        if (!docBody) return;

        const firstHeading = docBody.querySelector(this.headingSelectors);
        if (!firstHeading) return;

        // Create TOC container
        const tocContainer = document.createElement('div');
        tocContainer.className = 'table-of-contents';
        tocContainer.innerHTML = '<h3>Table of Contents</h3>';

        this.generateTableOfContents(tocContainer);

        // Insert TOC before the first heading
        firstHeading.parentNode.insertBefore(tocContainer, firstHeading);
    }

    generateTableOfContents(container) {
        const headings = document.querySelectorAll(this.headingSelectors);
        if (headings.length === 0) return;

        // Clear existing content except title
        const title = container.querySelector('h1, h2, h3, h4, h5, h6');
        container.innerHTML = title ? title.outerHTML : '<h3>Table of Contents</h3>';

        const tocList = document.createElement('ul');
        tocList.className = 'toc-list';

        let currentLevel = null;
        let currentList = tocList;
        const listStack = [tocList];

        headings.forEach(heading => {
            const level = parseInt(heading.tagName.substring(1));

            // Handle nesting
            if (currentLevel === null) {
                currentLevel = level;
            } else if (level > currentLevel) {
                // Deeper level - create nested list
                const nestedList = document.createElement('ul');
                if (currentList.lastElementChild) {
                    currentList.lastElementChild.appendChild(nestedList);
                } else {
                    const tempLi = document.createElement('li');
                    tempLi.appendChild(nestedList);
                    currentList.appendChild(tempLi);
                }
                listStack.push(nestedList);
                currentList = nestedList;
                currentLevel = level;
            } else if (level < currentLevel) {
                // Shallower level - pop from stack
                while (listStack.length > 1 && level < currentLevel) {
                    listStack.pop();
                    currentLevel--;
                }
                currentList = listStack[listStack.length - 1];
                currentLevel = level;
            }

            // Create list item
            const li = document.createElement('li');
            const a = document.createElement('a');
            a.href = `#${heading.id}`;
            a.className = 'toc-anchor';
            a.textContent = heading.textContent.replace(/[#¶§▪]/, '').trim();
            a.setAttribute('aria-label', `Go to ${a.textContent}`);

            li.appendChild(a);
            currentList.appendChild(li);
        });

        container.appendChild(tocList);
    }

    // Public API for programmatic control
    scrollToAnchor(anchorId) {
        const element = document.getElementById(anchorId);
        if (element) {
            this.scrollToElement(element);
            window.history.pushState(null, null, `#${anchorId}`);
        }
    }

    refreshAnchors() {
        // Remove existing anchors
        document.querySelectorAll(`.${this.anchorClass}`).forEach(anchor => {
            anchor.remove();
        });

        // Regenerate
        this.generateAnchors();
        this.updateTableOfContents();
    }
}

// Initialize anchor links
window.anchorLinks = new AnchorLinks();

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = AnchorLinks;
}
