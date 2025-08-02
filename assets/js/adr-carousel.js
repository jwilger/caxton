// ADR Carousel Module
(function() {
    'use strict';

    // Initialize carousel when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initADRCarousel);
    } else {
        // Add a small delay to ensure DOM is fully parsed
        setTimeout(initADRCarousel, 100);
    }

    function initADRCarousel() {
        console.log('ADR Carousel: Initializing...');
        
        // Transform ADR list into carousel if on index page
        const adrContent = document.querySelector('.adr-content');
        console.log('ADR Content found:', !!adrContent);
        
        const h1 = document.querySelector('h1');
        console.log('H1 text:', h1?.textContent);
        
        // Normalize whitespace in H1 text to handle multi-line titles
        const h1Text = h1?.textContent.replace(/\s+/g, ' ').trim();
        console.log('H1 normalized text:', h1Text);
        
        if (!adrContent || !h1Text?.includes('Architecture Decision Records')) {
            console.log('ADR Carousel: Not on ADR index page, skipping initialization');
            return;
        }

        const list = adrContent.querySelector('ul');
        console.log('UL element found:', !!list);
        if (!list) {
            console.log('ADR Carousel: No UL element found, skipping initialization');
            return;
        }

        // Create carousel structure
        const container = document.createElement('div');
        container.className = 'adr-carousel-container';
        
        const carousel = document.createElement('div');
        carousel.className = 'adr-carousel';
        
        const indicators = document.createElement('div');
        indicators.className = 'adr-carousel-indicators';

        // Transform list items into carousel cards
        const items = Array.from(list.querySelectorAll('li'));
        items.forEach((item, index) => {
            const card = createADRCard(item, index);
            carousel.appendChild(card);
            
            // Create indicator
            const indicator = document.createElement('button');
            indicator.className = 'carousel-indicator';
            indicator.setAttribute('aria-label', `Go to slide ${index + 1}`);
            if (index === 0) indicator.classList.add('active');
            indicator.addEventListener('click', () => goToSlide(index));
            indicators.appendChild(indicator);
        });

        // Create controls
        const controls = createCarouselControls();
        
        // Replace list with carousel
        list.replaceWith(container);
        container.appendChild(carousel);
        container.appendChild(controls);
        container.appendChild(indicators);

        // Set up carousel state
        let currentIndex = 0;
        let autoPlayInterval = null;
        let isHovered = false;

        // Control functions
        function updateCarousel() {
            const cardWidth = 350 + 32; // card width + gap
            const offset = -currentIndex * cardWidth;
            carousel.style.transform = `translateX(${offset}px)`;
            
            // Update indicators
            indicators.querySelectorAll('.carousel-indicator').forEach((ind, i) => {
                ind.classList.toggle('active', i === currentIndex);
            });
            
            // Update button states
            const prevBtn = controls.querySelector('.carousel-prev');
            const nextBtn = controls.querySelector('.carousel-next');
            prevBtn.disabled = currentIndex === 0;
            nextBtn.disabled = currentIndex >= items.length - getVisibleCards();
        }

        function getVisibleCards() {
            const containerWidth = container.offsetWidth;
            return Math.floor(containerWidth / 382); // card width + gap
        }

        function goToSlide(index) {
            currentIndex = Math.max(0, Math.min(index, items.length - getVisibleCards()));
            updateCarousel();
        }

        function nextSlide() {
            if (currentIndex < items.length - getVisibleCards()) {
                currentIndex++;
            } else {
                currentIndex = 0; // Loop back to start
            }
            updateCarousel();
        }

        function prevSlide() {
            if (currentIndex > 0) {
                currentIndex--;
            } else {
                currentIndex = Math.max(0, items.length - getVisibleCards());
            }
            updateCarousel();
        }

        // Set up controls
        controls.querySelector('.carousel-prev').addEventListener('click', prevSlide);
        controls.querySelector('.carousel-next').addEventListener('click', nextSlide);

        // Auto-play functionality
        function startAutoPlay() {
            if (!isHovered) {
                autoPlayInterval = setInterval(nextSlide, 5000);
            }
        }

        function stopAutoPlay() {
            clearInterval(autoPlayInterval);
        }

        // Mouse hover detection
        container.addEventListener('mouseenter', () => {
            isHovered = true;
            stopAutoPlay();
        });

        container.addEventListener('mouseleave', () => {
            isHovered = false;
            startAutoPlay();
        });

        // Keyboard navigation
        container.addEventListener('keydown', (e) => {
            if (e.key === 'ArrowLeft') {
                prevSlide();
                stopAutoPlay();
            } else if (e.key === 'ArrowRight') {
                nextSlide();
                stopAutoPlay();
            }
        });

        // Handle window resize
        let resizeTimeout;
        window.addEventListener('resize', () => {
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                updateCarousel();
            }, 250);
        });

        // Initialize
        updateCarousel();
        startAutoPlay();
        
        console.log('ADR Carousel: Successfully initialized with', items.length, 'items');
    }

    function createADRCard(listItem, index) {
        const card = document.createElement('div');
        card.className = 'adr-carousel-item';
        card.setAttribute('tabindex', '0');
        
        // Extract link and content
        const link = listItem.querySelector('a');
        const linkText = link ? link.textContent : '';
        const fullText = listItem.textContent;
        
        console.log('Processing item:', { linkText, fullText });
        
        // Updated regex to handle **0001. Title** format
        // Try to match from the link text first (which contains the ADR number and title)
        const linkMatch = linkText.match(/(\d+)\.\s+(.+)/);
        
        if (linkMatch) {
            const [, number, title] = linkMatch;
            
            // Extract description - everything after the link in the list item
            const descriptionStart = fullText.indexOf(linkText) + linkText.length;
            let description = fullText.substring(descriptionStart).trim();
            
            // Remove leading dash if present
            if (description.startsWith('-')) {
                description = description.substring(1).trim();
            }
            
            console.log('Parsed ADR:', { number, title, description });
            
            // ADR number badge
            const badge = document.createElement('div');
            badge.className = 'adr-number';
            badge.textContent = `ADR ${number}`;
            card.appendChild(badge);
            
            // Title
            const titleEl = document.createElement('h3');
            titleEl.className = 'adr-title';
            titleEl.textContent = title;
            card.appendChild(titleEl);
            
            // Description
            if (description) {
                const desc = document.createElement('p');
                desc.className = 'adr-description';
                desc.textContent = description;
                card.appendChild(desc);
            }
        } else {
            console.log('No match found, using fallback');
            // Fallback for non-matching format
            card.innerHTML = listItem.innerHTML;
        }
        
        // Make card clickable
        if (link) {
            card.addEventListener('click', () => {
                window.location.href = link.href;
            });
            card.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    window.location.href = link.href;
                }
            });
        }
        
        return card;
    }

    function createCarouselControls() {
        const controls = document.createElement('div');
        controls.className = 'adr-carousel-controls';
        
        const prevBtn = document.createElement('button');
        prevBtn.className = 'carousel-btn carousel-prev';
        prevBtn.setAttribute('aria-label', 'Previous slide');
        prevBtn.innerHTML = '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"></polyline></svg>';
        
        const nextBtn = document.createElement('button');
        nextBtn.className = 'carousel-btn carousel-next';
        nextBtn.setAttribute('aria-label', 'Next slide');
        nextBtn.innerHTML = '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"></polyline></svg>';
        
        controls.appendChild(prevBtn);
        controls.appendChild(nextBtn);
        
        return controls;
    }
})();