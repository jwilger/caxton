// ADR Search Functionality
// Client-side search with filtering and result highlighting

(function() {
    'use strict';

    let searchIndex = [];
    let searchInput = null;
    let searchResults = null;

    // Initialize search when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initADRSearch);
    } else {
        initADRSearch();
    }

    function initADRSearch() {
        console.log('ADR Search: Initializing...');

        searchInput = document.getElementById('adr-search');
        searchResults = document.getElementById('search-results');

        if (!searchInput) {
            console.log('ADR Search: No search input found');
            return;
        }

        // Build search index
        buildSearchIndex();

        // Setup search functionality
        setupSearchInput();
        setupSearchResults();

        console.log('ADR Search: Successfully initialized with', searchIndex.length, 'ADRs');
    }

    function buildSearchIndex() {
        // Build index from ADR cards and timeline items
        const adrCards = document.querySelectorAll('.adr-card');
        const timelineItems = document.querySelectorAll('.timeline-item');

        // Index from ADR cards (index page)
        adrCards.forEach(card => {
            const titleElement = card.querySelector('.adr-title a') || card.querySelector('.adr-title');
            const excerptElement = card.querySelector('.adr-excerpt');
            const statusElement = card.querySelector('.adr-status');
            const linkElement = card.querySelector('.adr-title a') || card.querySelector('a');

            if (titleElement) {
                const item = {
                    title: titleElement.textContent.trim(),
                    excerpt: excerptElement ? excerptElement.textContent.trim() : '',
                    status: statusElement ? statusElement.textContent.trim() : '',
                    url: linkElement ? linkElement.href : '#',
                    element: card,
                    type: 'card'
                };

                searchIndex.push(item);
            }
        });

        // Index from timeline items (individual ADR page)
        timelineItems.forEach(item => {
            const titleElement = item.querySelector('.timeline-link h5');
            const statusElement = item.querySelector('.status');
            const linkElement = item.querySelector('.timeline-link');

            if (titleElement && !searchIndex.find(indexed => indexed.title === titleElement.textContent.trim())) {
                const indexItem = {
                    title: titleElement.textContent.trim(),
                    excerpt: '',
                    status: statusElement ? statusElement.textContent.trim() : '',
                    url: linkElement ? linkElement.href : '#',
                    element: item,
                    type: 'timeline'
                };

                searchIndex.push(indexItem);
            }
        });

        console.log('ADR Search: Built index with', searchIndex.length, 'items');
    }

    function setupSearchInput() {
        let searchTimeout;

        searchInput.addEventListener('input', function() {
            clearTimeout(searchTimeout);
            searchTimeout = setTimeout(() => {
                const query = this.value.trim();
                if (query.length > 0) {
                    performSearch(query);
                } else {
                    hideSearchResults();
                    resetVisibility();
                }
            }, 150); // Debounce search
        });

        // Handle keyboard navigation
        searchInput.addEventListener('keydown', function(e) {
            if (!searchResults) return;

            const resultItems = searchResults.querySelectorAll('.search-result');
            const activeResult = searchResults.querySelector('.search-result.active');
            let newIndex = -1;

            switch(e.key) {
                case 'ArrowDown':
                    e.preventDefault();
                    if (activeResult) {
                        const currentIndex = Array.from(resultItems).indexOf(activeResult);
                        newIndex = currentIndex < resultItems.length - 1 ? currentIndex + 1 : 0;
                    } else {
                        newIndex = 0;
                    }
                    break;

                case 'ArrowUp':
                    e.preventDefault();
                    if (activeResult) {
                        const currentIndex = Array.from(resultItems).indexOf(activeResult);
                        newIndex = currentIndex > 0 ? currentIndex - 1 : resultItems.length - 1;
                    } else {
                        newIndex = resultItems.length - 1;
                    }
                    break;

                case 'Enter':
                    e.preventDefault();
                    if (activeResult) {
                        const link = activeResult.querySelector('a');
                        if (link) link.click();
                    }
                    break;

                case 'Escape':
                    hideSearchResults();
                    resetVisibility();
                    this.blur();
                    break;
            }

            if (newIndex >= 0 && resultItems[newIndex]) {
                resultItems.forEach(item => item.classList.remove('active'));
                resultItems[newIndex].classList.add('active');
            }
        });

        // Hide search results when clicking outside
        document.addEventListener('click', function(e) {
            if (!searchInput.contains(e.target) && (!searchResults || !searchResults.contains(e.target))) {
                hideSearchResults();
            }
        });
    }

    function setupSearchResults() {
        if (!searchResults) {
            // Create search results container if it doesn't exist
            searchResults = document.createElement('div');
            searchResults.id = 'search-results';
            searchResults.className = 'search-results';
            searchInput.parentNode.appendChild(searchResults);
        }
    }

    function performSearch(query) {
        const results = searchADRs(query);
        displaySearchResults(results, query);

        // Also filter visible content
        filterVisibleContent(query);
    }

    function searchADRs(query) {
        const queryLower = query.toLowerCase();
        const results = [];

        searchIndex.forEach(item => {
            let score = 0;
            let matchedText = '';

            // Search in title (higher weight)
            if (item.title.toLowerCase().includes(queryLower)) {
                score += 10;
                matchedText = item.title;
            }

            // Search in excerpt (medium weight)
            if (item.excerpt.toLowerCase().includes(queryLower)) {
                score += 5;
                if (!matchedText) matchedText = item.excerpt;
            }

            // Search in status (lower weight)
            if (item.status.toLowerCase().includes(queryLower)) {
                score += 2;
                if (!matchedText) matchedText = item.status;
            }

            if (score > 0) {
                results.push({
                    ...item,
                    score,
                    matchedText,
                    highlightedTitle: highlightMatch(item.title, query),
                    highlightedExcerpt: highlightMatch(item.excerpt, query)
                });
            }
        });

        // Sort by score (highest first)
        results.sort((a, b) => b.score - a.score);

        return results.slice(0, 10); // Limit to top 10 results
    }

    function displaySearchResults(results, query) {
        if (!searchResults) return;

        if (results.length === 0) {
            searchResults.innerHTML = `
                <div class="search-result no-results">
                    <div class="no-results-text">No ADRs found for "${query}"</div>
                </div>
            `;
        } else {
            searchResults.innerHTML = results.map(result => `
                <div class="search-result" data-type="${result.type}">
                    <a href="${result.url}" class="search-result-link">
                        <div class="search-result-title">${result.highlightedTitle}</div>
                        ${result.highlightedExcerpt ? `<div class="search-result-excerpt">${result.highlightedExcerpt}</div>` : ''}
                        <div class="search-result-meta">
                            ${result.status ? `<span class="search-result-status">${result.status}</span>` : ''}
                        </div>
                    </a>
                </div>
            `).join('');
        }

        showSearchResults();

        // Add click handlers
        const resultLinks = searchResults.querySelectorAll('.search-result-link');
        resultLinks.forEach(link => {
            link.addEventListener('click', () => {
                hideSearchResults();
            });
        });
    }

    function filterVisibleContent(query) {
        const queryLower = query.toLowerCase();

        // Filter ADR cards
        const adrCards = document.querySelectorAll('.adr-card');
        let visibleCount = 0;

        adrCards.forEach(card => {
            const title = card.querySelector('.adr-title')?.textContent.toLowerCase() || '';
            const excerpt = card.querySelector('.adr-excerpt')?.textContent.toLowerCase() || '';
            const status = card.querySelector('.adr-status')?.textContent.toLowerCase() || '';

            if (title.includes(queryLower) || excerpt.includes(queryLower) || status.includes(queryLower)) {
                card.style.display = 'block';
                visibleCount++;
            } else {
                card.style.display = 'none';
            }
        });

        // Filter timeline items
        const timelineItems = document.querySelectorAll('.timeline-item');
        timelineItems.forEach(item => {
            const title = item.querySelector('.timeline-link h5')?.textContent.toLowerCase() || '';
            const status = item.querySelector('.status')?.textContent.toLowerCase() || '';

            if (title.includes(queryLower) || status.includes(queryLower)) {
                item.style.display = 'block';
            } else {
                item.style.display = 'none';
            }
        });

        // Show/hide empty state
        const emptyState = document.getElementById('empty-state');
        const adrGrid = document.getElementById('adr-grid');

        if (emptyState && adrGrid) {
            if (visibleCount === 0) {
                emptyState.style.display = 'block';
                adrGrid.style.display = 'none';
            } else {
                emptyState.style.display = 'none';
                adrGrid.style.display = 'grid';
            }
        }

        // Update timeline line
        if (window.ADRTimeline && window.ADRTimeline.updateTimelineLine) {
            window.ADRTimeline.updateTimelineLine();
        }
    }

    function resetVisibility() {
        // Reset all cards and timeline items to visible
        const adrCards = document.querySelectorAll('.adr-card');
        adrCards.forEach(card => {
            card.style.display = 'block';
        });

        const timelineItems = document.querySelectorAll('.timeline-item');
        timelineItems.forEach(item => {
            item.style.display = 'block';
        });

        // Hide empty state
        const emptyState = document.getElementById('empty-state');
        const adrGrid = document.getElementById('adr-grid');

        if (emptyState && adrGrid) {
            emptyState.style.display = 'none';
            adrGrid.style.display = 'grid';
        }

        // Update timeline line
        if (window.ADRTimeline && window.ADRTimeline.updateTimelineLine) {
            window.ADRTimeline.updateTimelineLine();
        }
    }

    function highlightMatch(text, query) {
        if (!text || !query) return text;

        const regex = new RegExp(`(${escapeRegex(query)})`, 'gi');
        return text.replace(regex, '<mark>$1</mark>');
    }

    function escapeRegex(string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    function showSearchResults() {
        if (searchResults) {
            searchResults.style.display = 'block';
        }
    }

    function hideSearchResults() {
        if (searchResults) {
            searchResults.style.display = 'none';
        }
    }

    // Export for use by other modules
    window.ADRSearch = {
        performSearch: performSearch,
        hideSearchResults: hideSearchResults,
        resetVisibility: resetVisibility
    };

})();
