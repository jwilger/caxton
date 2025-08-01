// Caxton Website Progressive Enhancement
class CaxtonSite {
    constructor() {
        this.initializeVersionInfo();
        this.initializeViewTabs();
        this.initializeScrollEffects();
        this.initializeCodeHighlighting();
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
}

// Initialize only if DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => new CaxtonSite());
} else {
    new CaxtonSite();
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

// Add CSS for active state
const style = document.createElement('style');
style.textContent = `
    .nav-link.active {
        color: var(--accent-primary);
    }
    .nav-link.active::after {
        width: 100%;
    }
`;
document.head.appendChild(style);