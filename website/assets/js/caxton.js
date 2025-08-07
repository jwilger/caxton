// Caxton Website Progressive Enhancement
class CaxtonSite {
    constructor() {
        this.initializeVersionInfo();
        this.initializeViewTabs();
        this.initializeScrollEffects();
        this.initializeCodeHighlighting();
        this.initializeSearchEnhancement();
    }

    async initializeVersionInfo() {
        try {
            const response = await fetch('/caxton/release-info.json');
            if (response.ok) {
                const data = await response.json();
                this.updateReleaseSection(data);
            }
        } catch (e) {
            // Graceful degradation - site works without dynamic content
            console.debug('Release info not available');
        }
    }

    updateReleaseSection(data) {
        const releaseCard = document.querySelector('.release-card');
        if (!releaseCard) return;

        const releaseDate = new Date(data.date).toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'long',
            day: 'numeric'
        });

        // Update the release card with actual data
        const statusBadge = data.version.includes('alpha') || data.version.includes('beta')
            ? 'Pre-release'
            : 'Latest';

        releaseCard.innerHTML = `
            <div class="release-status">
                <span class="status-badge">${statusBadge}</span>
                <span class="release-date">${releaseDate}</span>
            </div>
            <h3 class="release-title">${data.name || data.version}</h3>
            <div class="release-description">${this.parseMarkdown(data.notes)}</div>
            <div class="release-actions">
                <a href="https://github.com/jwilger/caxton/releases/tag/${data.version}" class="btn btn-outline">
                    View Release
                </a>
                <a href="https://github.com/jwilger/caxton/releases" class="btn btn-outline">
                    All Releases
                </a>
            </div>
        `;
    }

    parseMarkdown(text) {
        // Simple markdown parsing for release notes
        // In production, you might want to use a proper markdown parser
        return text
            .replace(/^### (.+)$/gm, '<h4>$1</h4>')
            .replace(/^## (.+)$/gm, '<h3>$1</h3>')
            .replace(/^# (.+)$/gm, '<h2>$1</h2>')
            .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
            .replace(/\*(.+?)\*/g, '<em>$1</em>')
            .replace(/`(.+?)`/g, '<code>$1</code>')
            .replace(/\n\n/g, '</p><p>')
            .replace(/^/, '<p>')
            .replace(/$/, '</p>');
    }

    initializeViewTabs() {
        const tabs = document.querySelectorAll('.view-tab');
        const panes = document.querySelectorAll('.view-pane');

        tabs.forEach(tab => {
            tab.addEventListener('click', () => {
                const view = tab.dataset.view;

                // Update active states
                tabs.forEach(t => t.classList.remove('active'));
                panes.forEach(p => p.classList.remove('active'));

                tab.classList.add('active');
                document.getElementById(`${view}-view`).classList.add('active');
            });
        });
    }

    initializeScrollEffects() {
        // Navbar shadow on scroll
        const navbar = document.querySelector('.navbar');
        let lastScroll = 0;

        window.addEventListener('scroll', () => {
            const currentScroll = window.pageYOffset;

            if (currentScroll <= 0) {
                navbar.style.boxShadow = 'none';
            } else {
                navbar.style.boxShadow = '0 2px 20px rgba(0, 0, 0, 0.3)';
            }

            lastScroll = currentScroll;
        });

        // Smooth reveal animations
        const observerOptions = {
            threshold: 0.1,
            rootMargin: '0px 0px -100px 0px'
        };

        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    entry.target.style.opacity = '1';
                    entry.target.style.transform = 'translateY(0)';
                }
            });
        }, observerOptions);

        // Apply to feature cards and other elements
        const animatedElements = document.querySelectorAll('.feature-card, .step-card, .resource-card');
        animatedElements.forEach((el, index) => {
            el.style.opacity = '0';
            el.style.transform = 'translateY(20px)';
            el.style.transition = `all 0.6s ease ${index * 0.1}s`;
            observer.observe(el);
        });
    }

    initializeCodeHighlighting() {
        // Simple syntax highlighting for inline code examples
        document.querySelectorAll('pre code').forEach(block => {
            // Skip if already highlighted
            if (block.querySelector('span')) return;

            let text = block.textContent;
            const lang = block.className.match(/language-(\w+)/)?.[1] || 'rust';

            if (lang === 'rust') {
                text = this.highlightRust(text);
            } else if (lang === 'bash' || lang === 'sh') {
                text = this.highlightBash(text);
            }

            block.innerHTML = text;
        });
    }

    highlightRust(code) {
        const keywords = /\b(use|fn|impl|struct|enum|trait|async|await|match|if|else|for|while|loop|return|break|continue|let|const|mut|pub|mod|self|Self|super|crate|move|ref|where|type|unsafe|extern|static|as|in|from|into)\b/g;
        const types = /\b(String|Result|Ok|Err|Option|Some|None|Vec|HashMap|bool|u8|u16|u32|u64|i8|i16|i32|i64|f32|f64|usize|isize|char|str)\b/g;
        const strings = /("(?:[^"\\]|\\.)*")/g;
        const comments = /(\/\/[^\n]*)/g;
        const attributes = /(#\[[^\]]+\])/g;
        const functions = /\b([a-z_][a-zA-Z0-9_]*)\s*\(/g;

        return code
            .replace(strings, '<span class="syntax-string">$1</span>')
            .replace(comments, '<span class="syntax-comment">$1</span>')
            .replace(attributes, '<span class="syntax-attribute">$1</span>')
            .replace(keywords, '<span class="syntax-keyword">$1</span>')
            .replace(types, '<span class="syntax-type">$1</span>')
            .replace(functions, '<span class="syntax-function">$1</span>(');
    }

    highlightBash(code) {
        const strings = /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/g;
        const comments = /(#[^\n]*)/g;
        const variables = /(\$\w+|\$\{[^}]+\})/g;
        const commands = /^(\s*)([\w-]+)(?=\s|$)/gm;

        return code
            .replace(strings, '<span class="syntax-string">$1</span>')
            .replace(comments, '<span class="syntax-comment">$1</span>')
            .replace(variables, '<span class="syntax-type">$1</span>')
            .replace(commands, '$1<span class="syntax-function">$2</span>');
    }

    initializeSearchEnhancement() {
        this.searchState = {
            currentIndex: -1,
            highlights: [],
            searchTerm: '',
            searchInput: null,
            clearButton: null,
            resultCounter: null,
            searchResults: null,
            debounceTimer: null
        };

        this.setupSearchUI();
        this.loadSearchFromURL();
        this.bindSearchEvents();
    }

    setupSearchUI() {
        // Find all search inputs on the page
        const searchInputs = document.querySelectorAll('input[type="search"], input[placeholder*="search"], #adr-search');

        searchInputs.forEach(input => {
            this.enhanceSearchInput(input);
        });
    }

    enhanceSearchInput(input) {
        const container = input.parentElement;
        const isADRSearch = input.id === 'adr-search';

        // Store reference to the main search input
        if (!this.searchState.searchInput) {
            this.searchState.searchInput = input;
        }

        // Add accessibility attributes to input
        if (!input.hasAttribute('role')) {
            input.setAttribute('role', 'searchbox');
            input.setAttribute('aria-label', 'Search content');
            input.setAttribute('aria-describedby', 'search-instructions');
        }

        // Create enhanced search container
        if (!container.classList.contains('enhanced-search')) {
            container.classList.add('enhanced-search');

            // Create search instructions for screen readers
            const instructions = document.createElement('div');
            instructions.id = 'search-instructions';
            instructions.className = 'sr-only';
            instructions.textContent = 'Type to search, use arrow keys to navigate results, Enter to jump to result, Escape to clear';
            instructions.style.cssText = `
                position: absolute;
                left: -10000px;
                top: auto;
                width: 1px;
                height: 1px;
                overflow: hidden;
            `;
            container.appendChild(instructions);

            // Create clear button
            const clearButton = document.createElement('button');
            clearButton.className = 'search-clear-btn';
            clearButton.innerHTML = '×';
            clearButton.title = 'Clear search';
            clearButton.type = 'button';
            clearButton.setAttribute('aria-label', 'Clear search');
            clearButton.style.cssText = `
                position: absolute;
                right: ${isADRSearch ? '12px' : '40px'};
                top: 50%;
                transform: translateY(-50%);
                background: none;
                border: none;
                font-size: 18px;
                color: var(--text-muted);
                cursor: pointer;
                display: none;
                z-index: 10;
                width: 24px;
                height: 24px;
                border-radius: 50%;
                transition: all var(--transition-fast);
            `;

            // Create result counter
            const resultCounter = document.createElement('div');
            resultCounter.className = 'search-result-counter';
            resultCounter.style.cssText = `
                position: absolute;
                right: ${isADRSearch ? '45px' : '70px'};
                top: 50%;
                transform: translateY(-50%);
                font-size: var(--font-size-sm);
                color: var(--text-muted);
                font-family: var(--font-mono);
                display: none;
                z-index: 10;
                background: var(--bg-surface);
                padding: 2px 6px;
                border-radius: var(--radius-sm);
                border: 1px solid var(--color-surface1);
            `;

            container.style.position = 'relative';
            container.appendChild(clearButton);
            container.appendChild(resultCounter);

            // Store references
            if (!this.searchState.clearButton) {
                this.searchState.clearButton = clearButton;
                this.searchState.resultCounter = resultCounter;
            }
        }
    }

    bindSearchEvents() {
        if (!this.searchState.searchInput) return;

        const input = this.searchState.searchInput;

        // Main search functionality
        input.addEventListener('input', (e) => {
            clearTimeout(this.searchState.debounceTimer);
            this.searchState.debounceTimer = setTimeout(() => {
                this.handleSearch(e.target.value.trim());
            }, 200);
        });

        // Keyboard navigation
        input.addEventListener('keydown', (e) => {
            if (!this.searchState.highlights.length) return;

            switch(e.key) {
                case 'ArrowDown':
                    e.preventDefault();
                    this.navigateResults(1);
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    this.navigateResults(-1);
                    break;
                case 'Enter':
                    e.preventDefault();
                    if (this.searchState.currentIndex >= 0) {
                        this.jumpToHighlight(this.searchState.currentIndex);
                    }
                    break;
                case 'Escape':
                    this.clearSearch();
                    input.blur();
                    break;
            }
        });

        // Clear button
        if (this.searchState.clearButton) {
            this.searchState.clearButton.addEventListener('click', () => {
                this.clearSearch();
                input.focus();
            });
        }

        // Show/hide clear button based on input
        input.addEventListener('input', (e) => {
            const hasValue = e.target.value.length > 0;
            if (this.searchState.clearButton) {
                this.searchState.clearButton.style.display = hasValue ? 'block' : 'none';
            }
        });

        // Handle URL changes
        window.addEventListener('popstate', () => {
            this.loadSearchFromURL();
        });

        // Global keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            // Ctrl/Cmd + F for search focus
            if ((e.ctrlKey || e.metaKey) && e.key === 'f' && input) {
                e.preventDefault();
                input.focus();
                input.select();
            }

            // Escape to clear search when input is focused
            if (e.key === 'Escape' && document.activeElement === input) {
                this.clearSearch();
                input.blur();
            }

            // F3 or Ctrl/Cmd + G for next result
            if ((e.key === 'F3' || ((e.ctrlKey || e.metaKey) && e.key === 'g')) &&
                this.searchState.highlights.length > 0) {
                e.preventDefault();
                this.navigateResults(e.shiftKey ? -1 : 1);
            }
        });
    }

    handleSearch(searchTerm) {
        this.searchState.searchTerm = searchTerm;

        if (searchTerm.length === 0) {
            this.clearHighlights();
            this.updateResultCounter();
            this.updateURL();
            return;
        }

        if (searchTerm.length < 2) {
            return; // Wait for at least 2 characters
        }

        this.performHighlighting(searchTerm);
        this.updateResultCounter();
        this.updateURL(searchTerm);
    }

    performHighlighting(searchTerm) {
        this.clearHighlights();

        // Throttle search for very short terms
        if (searchTerm.length < 2) return;

        // Create search regex (case-insensitive, whole words and partial matches)
        const escapedTerm = searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        const regex = new RegExp(`(${escapedTerm})`, 'gi');

        // Define searchable selectors (excluding script, style, and existing highlights)
        const searchableSelectors = [
            'p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'li', 'td', 'th',
            '.adr-title', '.adr-excerpt', '.docs-content', '.timeline-link',
            'blockquote', '.feature-card', '.step-card', '.resource-card',
            '.adr-card-content', '.timeline-description', '.nav-link'
        ];

        // Performance optimization: Use requestAnimationFrame for heavy operations
        const performSearch = () => {
            const searchableElements = document.querySelectorAll(
                searchableSelectors.join(', ')
            );

            const elementsArray = Array.from(searchableElements);
            let processed = 0;

            const processChunk = () => {
                const chunkSize = 10; // Process 10 elements at a time
                const endIndex = Math.min(processed + chunkSize, elementsArray.length);

                for (let i = processed; i < endIndex; i++) {
                    const element = elementsArray[i];

                    // Skip if element contains child elements that might be searchable
                    if (element.querySelector(searchableSelectors.join(', '))) {
                        continue;
                    }

                    // Skip if element is inside a script or style tag
                    if (element.closest('script, style, .search-highlight')) {
                        continue;
                    }

                    this.highlightInElement(element, regex, searchTerm);
                }

                processed = endIndex;

                if (processed < elementsArray.length) {
                    // Continue processing in next frame
                    requestAnimationFrame(processChunk);
                } else {
                    // Processing complete
                    this.finishHighlighting();
                }
            };

            processChunk();
        };

        requestAnimationFrame(performSearch);
    }

    finishHighlighting() {
        // If we have highlights, set current to first one
        if (this.searchState.highlights.length > 0) {
            this.searchState.currentIndex = 0;
            this.updateCurrentHighlight();

            // Announce to screen readers
            this.announceSearchResults();
        } else {
            this.announceNoResults();
        }
    }

    announceSearchResults() {
        // Create or update ARIA live region for accessibility
        let liveRegion = document.getElementById('search-announcements');
        if (!liveRegion) {
            liveRegion = document.createElement('div');
            liveRegion.id = 'search-announcements';
            liveRegion.setAttribute('aria-live', 'polite');
            liveRegion.setAttribute('aria-atomic', 'true');
            liveRegion.style.cssText = `
                position: absolute;
                left: -10000px;
                top: auto;
                width: 1px;
                height: 1px;
                overflow: hidden;
            `;
            document.body.appendChild(liveRegion);
        }

        const count = this.searchState.highlights.length;
        const term = this.searchState.searchTerm;
        liveRegion.textContent = `Found ${count} result${count !== 1 ? 's' : ''} for "${term}". Use arrow keys to navigate between results.`;
    }

    announceNoResults() {
        let liveRegion = document.getElementById('search-announcements');
        if (liveRegion) {
            const term = this.searchState.searchTerm;
            liveRegion.textContent = `No results found for "${term}".`;
        }
    }

    highlightInElement(element, regex, originalTerm) {
        const textNodes = this.getTextNodes(element);

        textNodes.forEach(textNode => {
            const text = textNode.textContent;
            if (regex.test(text)) {
                const highlightedHTML = text.replace(regex, (match) => {
                    return `<mark class="search-highlight" data-search-term="${originalTerm}">${match}</mark>`;
                });

                // Create a temporary container to parse the HTML
                const tempDiv = document.createElement('div');
                tempDiv.innerHTML = highlightedHTML;

                // Replace the text node with the highlighted content
                const fragment = document.createDocumentFragment();
                while (tempDiv.firstChild) {
                    fragment.appendChild(tempDiv.firstChild);
                }

                textNode.parentNode.replaceChild(fragment, textNode);
            }
        });

        // Collect all highlights after replacement
        const highlights = element.querySelectorAll('.search-highlight');
        highlights.forEach(highlight => {
            this.searchState.highlights.push(highlight);
        });
    }

    getTextNodes(element) {
        const textNodes = [];
        const walker = document.createTreeWalker(
            element,
            NodeFilter.SHOW_TEXT,
            {
                acceptNode: function(node) {
                    // Skip empty text nodes and those inside scripts/styles
                    if (node.textContent.trim() === '' ||
                        node.parentElement.closest('script, style, .search-highlight')) {
                        return NodeFilter.FILTER_REJECT;
                    }
                    return NodeFilter.FILTER_ACCEPT;
                }
            }
        );

        let node;
        while (node = walker.nextNode()) {
            textNodes.push(node);
        }

        return textNodes;
    }

    navigateResults(direction) {
        if (this.searchState.highlights.length === 0) return;

        // Update current index
        this.searchState.currentIndex += direction;

        if (this.searchState.currentIndex >= this.searchState.highlights.length) {
            this.searchState.currentIndex = 0;
        } else if (this.searchState.currentIndex < 0) {
            this.searchState.currentIndex = this.searchState.highlights.length - 1;
        }

        this.updateCurrentHighlight();
        this.jumpToHighlight(this.searchState.currentIndex);
        this.updateResultCounter();
    }

    updateCurrentHighlight() {
        // Remove current class from all highlights
        this.searchState.highlights.forEach(highlight => {
            highlight.classList.remove('search-highlight-current');
        });

        // Add current class to active highlight
        if (this.searchState.currentIndex >= 0 &&
            this.searchState.currentIndex < this.searchState.highlights.length) {
            this.searchState.highlights[this.searchState.currentIndex].classList.add('search-highlight-current');
        }
    }

    jumpToHighlight(index) {
        if (index < 0 || index >= this.searchState.highlights.length) return;

        const highlight = this.searchState.highlights[index];

        // Smooth scroll to highlight with offset for fixed headers
        const rect = highlight.getBoundingClientRect();
        const absoluteTop = window.pageYOffset + rect.top;
        const headerOffset = 100; // Adjust based on your header height

        window.scrollTo({
            top: absoluteTop - headerOffset,
            behavior: 'smooth'
        });

        // Flash animation for better visibility
        highlight.style.animation = 'searchFlash 1s ease-in-out';
        setTimeout(() => {
            highlight.style.animation = '';
        }, 1000);
    }

    updateResultCounter() {
        if (!this.searchState.resultCounter) return;

        if (this.searchState.highlights.length === 0) {
            this.searchState.resultCounter.style.display = 'none';
        } else {
            this.searchState.resultCounter.style.display = 'block';
            this.searchState.resultCounter.textContent =
                `${this.searchState.currentIndex + 1}/${this.searchState.highlights.length}`;
        }
    }

    clearHighlights() {
        // Remove all existing highlights
        document.querySelectorAll('.search-highlight').forEach(highlight => {
            const parent = highlight.parentNode;
            parent.replaceChild(document.createTextNode(highlight.textContent), highlight);
            parent.normalize(); // Merge adjacent text nodes
        });

        this.searchState.highlights = [];
        this.searchState.currentIndex = -1;
    }

    clearSearch() {
        this.clearHighlights();
        if (this.searchState.searchInput) {
            this.searchState.searchInput.value = '';
        }
        this.updateResultCounter();
        if (this.searchState.clearButton) {
            this.searchState.clearButton.style.display = 'none';
        }
        this.updateURL();
    }

    updateURL(searchTerm = '') {
        const url = new URL(window.location);
        if (searchTerm) {
            url.searchParams.set('search', searchTerm);
        } else {
            url.searchParams.delete('search');
        }
        window.history.replaceState({}, '', url);
    }

    loadSearchFromURL() {
        const url = new URL(window.location);
        const searchTerm = url.searchParams.get('search');

        if (searchTerm && this.searchState.searchInput) {
            this.searchState.searchInput.value = searchTerm;
            this.handleSearch(searchTerm);
            if (this.searchState.clearButton) {
                this.searchState.clearButton.style.display = 'block';
            }
        }
    }

    // Public API for integration with existing search modules
    getSearchAPI() {
        return {
            performSearch: (term) => this.handleSearch(term),
            clearSearch: () => this.clearSearch(),
            navigateNext: () => this.navigateResults(1),
            navigatePrev: () => this.navigateResults(-1),
            jumpToResult: (index) => this.jumpToHighlight(index),
            getResultCount: () => this.searchState.highlights.length,
            getCurrentIndex: () => this.searchState.currentIndex,
            setSearchTerm: (term) => {
                if (this.searchState.searchInput) {
                    this.searchState.searchInput.value = term;
                    this.handleSearch(term);
                }
            }
        };
    }
}

// Initialize only if DOM is ready
let caxtonSiteInstance;
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        caxtonSiteInstance = new CaxtonSite();
        // Make search API globally available
        window.CaxtonSearch = caxtonSiteInstance.getSearchAPI();
    });
} else {
    caxtonSiteInstance = new CaxtonSite();
    // Make search API globally available
    window.CaxtonSearch = caxtonSiteInstance.getSearchAPI();
}

// Active navigation highlighting
const sections = document.querySelectorAll('section[id]');
const navLinks = document.querySelectorAll('.nav-link');

window.addEventListener('scroll', () => {
    let current = '';

    sections.forEach(section => {
        const sectionTop = section.offsetTop;
        const sectionHeight = section.clientHeight;
        if (pageYOffset >= sectionTop - 200) {
            current = section.getAttribute('id');
        }
    });

    navLinks.forEach(link => {
        link.classList.remove('active');
        if (link.getAttribute('href') === `#${current}`) {
            link.classList.add('active');
        }
    });
});

// Add CSS for active state and search highlighting
const style = document.createElement('style');
style.textContent = `
    .nav-link.active {
        color: var(--accent-primary);
    }
    .nav-link.active::after {
        width: 100%;
    }

    /* Search Enhancement Styles */
    .enhanced-search {
        position: relative;
    }

    .search-highlight {
        background-color: var(--color-yellow);
        color: var(--bg-primary);
        padding: 1px 2px;
        border-radius: 2px;
        font-weight: var(--font-medium);
        transition: all var(--transition-fast);
    }

    .search-highlight-current {
        background-color: var(--color-primary) !important;
        color: var(--text-on-primary) !important;
        box-shadow: 0 0 8px rgba(137, 180, 250, 0.4);
        outline: 2px solid var(--color-primary);
        outline-offset: 1px;
    }

    .search-clear-btn:hover {
        background-color: var(--bg-surface) !important;
        color: var(--color-red) !important;
        transform: translateY(-50%) scale(1.1);
    }

    .search-clear-btn:focus {
        outline: 2px solid var(--color-primary);
        outline-offset: 2px;
    }

    .search-result-counter {
        font-weight: var(--font-medium);
        user-select: none;
    }

    /* Flash animation for current highlight */
    @keyframes searchFlash {
        0%, 100% {
            transform: scale(1);
            opacity: 1;
        }
        50% {
            transform: scale(1.05);
            opacity: 0.8;
            background-color: var(--color-lavender);
        }
    }

    /* Accessibility improvements */
    .search-highlight:focus {
        outline: 2px solid var(--color-primary);
        outline-offset: 2px;
    }

    /* Responsive adjustments for mobile */
    @media (max-width: 768px) {
        .search-result-counter {
            font-size: var(--font-size-xs);
            right: 35px !important;
        }

        .search-clear-btn {
            right: 8px !important;
            width: 20px;
            height: 20px;
            font-size: 16px;
        }
    }

    /* High contrast mode support */
    @media (prefers-contrast: high) {
        .search-highlight {
            background-color: var(--color-yellow);
            color: var(--bg-primary);
            border: 1px solid var(--text-primary);
        }

        .search-highlight-current {
            background-color: var(--text-primary) !important;
            color: var(--bg-primary) !important;
            border: 2px solid var(--color-primary) !important;
        }
    }

    /* Dark mode adjustments */
    @media (prefers-color-scheme: dark) {
        .search-highlight {
            background-color: var(--color-yellow);
            color: var(--bg-primary);
        }
    }

    /* Print styles - hide search elements */
    @media print {
        .search-highlight {
            background-color: transparent !important;
            color: inherit !important;
            font-weight: var(--font-bold);
            text-decoration: underline;
        }

        .search-clear-btn,
        .search-result-counter {
            display: none !important;
        }
    }
`;
document.head.appendChild(style);
