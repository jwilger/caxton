// ADR Timeline Navigation JavaScript
// Enhanced navigation with timeline visualization and search integration

(function() {
    'use strict';

    // Initialize timeline functionality when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initADRTimeline);
    } else {
        initADRTimeline();
    }

    function initADRTimeline() {
        console.log('ADR Timeline: Initializing...');

        const timelineContainer = document.querySelector('.adr-timeline');
        if (!timelineContainer) {
            console.log('ADR Timeline: No timeline container found');
            return;
        }

        // Initialize timeline features
        setupTimelineNavigation();
        setupTimelineFiltering();
        setupTimelineCollapse();
        setupTimelineKeyboardNav();

        console.log('ADR Timeline: Successfully initialized');
    }

    function setupTimelineNavigation() {
        const timelineItems = document.querySelectorAll('.timeline-item');
        const currentItem = document.querySelector('.timeline-item.current');

        if (currentItem) {
            // Scroll current item into view
            setTimeout(() => {
                currentItem.scrollIntoView({
                    behavior: 'smooth',
                    block: 'center'
                });
            }, 100);
        }

        // Add click handlers for timeline items
        timelineItems.forEach(item => {
            const link = item.querySelector('.timeline-link');
            if (link) {
                // Add hover effect
                item.addEventListener('mouseenter', () => {
                    item.classList.add('timeline-hover');
                });

                item.addEventListener('mouseleave', () => {
                    item.classList.remove('timeline-hover');
                });

                // Add focus management
                link.addEventListener('focus', () => {
                    item.scrollIntoView({
                        behavior: 'smooth',
                        block: 'center'
                    });
                });
            }
        });
    }

    function setupTimelineFiltering() {
        const filterButtons = document.querySelectorAll('.filter-btn');
        const timelineItems = document.querySelectorAll('.timeline-item');

        filterButtons.forEach(button => {
            button.addEventListener('click', function() {
                const filter = this.dataset.filter;

                // Update active filter button
                filterButtons.forEach(btn => btn.classList.remove('active'));
                this.classList.add('active');

                // Filter timeline items
                timelineItems.forEach(item => {
                    const link = item.querySelector('.timeline-link');
                    const statusBadge = item.querySelector('.status');

                    if (filter === 'all') {
                        item.style.display = 'block';
                    } else {
                        const status = statusBadge ? statusBadge.textContent.toLowerCase() : '';
                        if (status.includes(filter)) {
                            item.style.display = 'block';
                        } else {
                            item.style.display = 'none';
                        }
                    }
                });

                // Update timeline line visibility
                updateTimelineLine();
            });
        });
    }

    function setupTimelineCollapse() {
        const timelineHeader = document.querySelector('.adr-timeline h4');
        const timelineList = document.querySelector('.timeline-list');

        if (timelineHeader && timelineList) {
            // Make timeline collapsible on mobile
            timelineHeader.addEventListener('click', function() {
                if (window.innerWidth <= 1024) {
                    const isCollapsed = timelineList.style.display === 'none';
                    timelineList.style.display = isCollapsed ? 'block' : 'none';
                    timelineHeader.classList.toggle('collapsed', !isCollapsed);
                }
            });

            // Handle window resize
            window.addEventListener('resize', () => {
                if (window.innerWidth > 1024) {
                    timelineList.style.display = 'block';
                    timelineHeader.classList.remove('collapsed');
                }
            });
        }
    }

    function setupTimelineKeyboardNav() {
        const timelineItems = document.querySelectorAll('.timeline-item');
        const timelineLinks = document.querySelectorAll('.timeline-link');

        timelineLinks.forEach((link, index) => {
            link.addEventListener('keydown', function(e) {
                let targetIndex = -1;

                switch(e.key) {
                    case 'ArrowUp':
                        e.preventDefault();
                        targetIndex = index > 0 ? index - 1 : timelineLinks.length - 1;
                        break;
                    case 'ArrowDown':
                        e.preventDefault();
                        targetIndex = index < timelineLinks.length - 1 ? index + 1 : 0;
                        break;
                    case 'Home':
                        e.preventDefault();
                        targetIndex = 0;
                        break;
                    case 'End':
                        e.preventDefault();
                        targetIndex = timelineLinks.length - 1;
                        break;
                }

                if (targetIndex >= 0) {
                    timelineLinks[targetIndex].focus();
                }
            });
        });
    }

    function updateTimelineLine() {
        // Update the connecting line between visible timeline items
        const visibleItems = document.querySelectorAll('.timeline-item[style*="block"], .timeline-item:not([style])');
        const timelineLine = document.querySelector('.timeline-list::before');

        if (visibleItems.length > 0) {
            const firstItem = visibleItems[0];
            const lastItem = visibleItems[visibleItems.length - 1];

            // Calculate line height based on visible items
            // This is handled by CSS, but we could add dynamic adjustments here
        }
    }

    // Export for use by other modules
    window.ADRTimeline = {
        updateTimelineLine: updateTimelineLine
    };

})();
