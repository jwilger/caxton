/**
 * Roadmap Progress Visualization System
 * Creates animated progress bars and milestone tracking for the Caxton roadmap
 * Uses Catppuccin colors and follows the design system
 */

class RoadmapProgress {
  constructor() {
    this.phases = [
      {
        id: "v1-isolation-core",
        name: "V1.0 Isolation Core",
        version: "1.0",
        progress: 75,
        status: "in-progress",
        quarter: "Q1 2025",
        estimatedCompletion: "2025-03-31",
        milestones: [
          { name: "Container Isolation Foundation", completed: true },
          { name: "Process Sandboxing", completed: true },
          { name: "Resource Constraints", completed: true },
          { name: "Security Framework", completed: false },
          { name: "Basic Monitoring", completed: false },
        ],
        description:
          "Core isolation mechanisms and foundational security features",
      },
      {
        id: "v2-heterogeneous-agents",
        name: "V2.0 Heterogeneous Agents",
        version: "2.0",
        progress: 25,
        status: "in-progress",
        quarter: "Q3 2025",
        estimatedCompletion: "2025-09-30",
        milestones: [
          { name: "Multi-Language Support", completed: false },
          { name: "Agent Communication Protocol", completed: false },
          { name: "Dynamic Scaling", completed: false },
          { name: "Cross-Platform Compatibility", completed: false },
          { name: "Performance Optimization", completed: false },
        ],
        description:
          "Support for diverse agent types and advanced orchestration",
      },
      {
        id: "v3-production-scale",
        name: "V3.0 Production Scale",
        version: "3.0",
        progress: 5,
        status: "planning",
        quarter: "Q1 2026",
        estimatedCompletion: "2026-03-31",
        milestones: [
          { name: "Enterprise Features", completed: false },
          { name: "Advanced Analytics", completed: false },
          { name: "High Availability", completed: false },
          { name: "Auto-scaling Infrastructure", completed: false },
          { name: "Production Hardening", completed: false },
        ],
        description:
          "Enterprise-ready features and production-scale capabilities",
      },
    ];

    this.init();
  }

  init() {
    this.createProgressBars();
    this.updateProgressBars();
    this.addEventListeners();
    this.startAnimations();
  }

  createProgressBars() {
    const container = document.querySelector(".roadmap-progress-container");
    if (!container) return;

    const progressHTML = this.phases
      .map((phase) => this.createPhaseHTML(phase))
      .join("");
    container.innerHTML = progressHTML;
  }

  createPhaseHTML(phase) {
    const statusClass = this.getStatusClass(phase.status);
    const progressColor = this.getProgressColor(phase.status);
    const milestonesList = phase.milestones
      .map(
        (milestone) =>
          `<li class="milestone-item ${milestone.completed ? "completed" : "pending"}">
                <span class="milestone-icon">${milestone.completed ? "✓" : "○"}</span>
                <span class="milestone-name">${milestone.name}</span>
            </li>`,
      )
      .join("");

    return `
            <div class="roadmap-phase ${statusClass}" data-phase-id="${phase.id}">
                <div class="phase-header">
                    <div class="phase-info">
                        <h3 class="phase-title">${phase.name}</h3>
                        <div class="phase-meta">
                            <span class="phase-quarter">${phase.quarter}</span>
                            <span class="phase-status">${this.getStatusLabel(phase.status)}</span>
                        </div>
                    </div>
                    <div class="phase-progress-info">
                        <span class="progress-percentage">${phase.progress}%</span>
                        <div class="estimated-completion">
                            Est: ${this.formatDate(phase.estimatedCompletion)}
                        </div>
                    </div>
                </div>

                <div class="progress-bar-container">
                    <div class="progress-bar-track">
                        <div class="progress-bar-fill ${progressColor}"
                             data-progress="${phase.progress}"
                             style="width: 0%">
                        </div>
                    </div>
                </div>

                <p class="phase-description">${phase.description}</p>

                <div class="milestones-section">
                    <h4 class="milestones-title">Key Milestones</h4>
                    <ul class="milestones-list">
                        ${milestonesList}
                    </ul>
                </div>
            </div>
        `;
  }

  getStatusClass(status) {
    const statusMap = {
      completed: "phase-completed",
      "in-progress": "phase-in-progress",
      planning: "phase-planning",
    };
    return statusMap[status] || "phase-planning";
  }

  getProgressColor(status) {
    const colorMap = {
      completed: "progress-complete",
      "in-progress": "progress-active",
      planning: "progress-planning",
    };
    return colorMap[status] || "progress-planning";
  }

  getStatusLabel(status) {
    const labelMap = {
      completed: "Complete",
      "in-progress": "In Progress",
      planning: "Planning",
    };
    return labelMap[status] || "Planned";
  }

  formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", {
      month: "short",
      year: "numeric",
    });
  }

  updateProgressBars() {
    const progressBars = document.querySelectorAll(".progress-bar-fill");

    progressBars.forEach((bar, index) => {
      const progress = parseInt(bar.dataset.progress);

      // Animate progress bar fill
      setTimeout(() => {
        bar.style.width = `${progress}%`;
      }, index * 200); // Stagger animations
    });
  }

  addEventListeners() {
    // Add hover effects and interactions
    document.querySelectorAll(".roadmap-phase").forEach((phase) => {
      phase.addEventListener("mouseenter", (e) => {
        this.highlightPhase(e.currentTarget);
      });

      phase.addEventListener("mouseleave", (e) => {
        this.unhighlightPhase(e.currentTarget);
      });
    });

    // Add milestone toggle functionality
    document.querySelectorAll(".milestones-title").forEach((title) => {
      title.addEventListener("click", (e) => {
        this.toggleMilestones(e.currentTarget);
      });
    });
  }

  highlightPhase(phaseElement) {
    phaseElement.classList.add("phase-highlighted");

    // Add glow effect to progress bar
    const progressBar = phaseElement.querySelector(".progress-bar-fill");
    if (progressBar) {
      progressBar.style.boxShadow = "0 0 20px rgba(137, 180, 250, 0.5)";
    }
  }

  unhighlightPhase(phaseElement) {
    phaseElement.classList.remove("phase-highlighted");

    // Remove glow effect
    const progressBar = phaseElement.querySelector(".progress-bar-fill");
    if (progressBar) {
      progressBar.style.boxShadow = "";
    }
  }

  toggleMilestones(titleElement) {
    const milestonesList = titleElement.nextElementSibling;
    const isExpanded = milestonesList.classList.contains("expanded");

    if (isExpanded) {
      milestonesList.classList.remove("expanded");
      titleElement.textContent = titleElement.textContent.replace("▼", "▶");
    } else {
      milestonesList.classList.add("expanded");
      titleElement.textContent = titleElement.textContent.replace("▶", "▼");
    }
  }

  startAnimations() {
    // Add intersection observer for scroll-triggered animations
    if ("IntersectionObserver" in window) {
      const observer = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            if (entry.isIntersecting) {
              entry.target.classList.add("animate-in");
            }
          });
        },
        {
          threshold: 0.1,
          rootMargin: "50px",
        },
      );

      document.querySelectorAll(".roadmap-phase").forEach((phase) => {
        observer.observe(phase);
      });
    }
  }

  // Public API methods
  updatePhaseProgress(phaseId, newProgress) {
    const phase = this.phases.find((p) => p.id === phaseId);
    if (phase) {
      phase.progress = Math.max(0, Math.min(100, newProgress));

      const progressBar = document.querySelector(
        `[data-phase-id="${phaseId}"] .progress-bar-fill`,
      );
      const percentageDisplay = document.querySelector(
        `[data-phase-id="${phaseId}"] .progress-percentage`,
      );

      if (progressBar) {
        progressBar.style.width = `${phase.progress}%`;
        progressBar.dataset.progress = phase.progress;
      }

      if (percentageDisplay) {
        percentageDisplay.textContent = `${phase.progress}%`;
      }
    }
  }

  completeMilestone(phaseId, milestoneName) {
    const phase = this.phases.find((p) => p.id === phaseId);
    if (phase) {
      const milestone = phase.milestones.find((m) => m.name === milestoneName);
      if (milestone) {
        milestone.completed = true;

        // Update DOM
        const milestoneElement = document.querySelector(
          `[data-phase-id="${phaseId}"] .milestone-name:contains("${milestoneName}")`,
        );

        if (milestoneElement) {
          const listItem = milestoneElement.closest(".milestone-item");
          listItem.classList.add("completed");
          listItem.classList.remove("pending");

          const icon = listItem.querySelector(".milestone-icon");
          if (icon) {
            icon.textContent = "✓";
          }
        }

        // Update phase progress based on completed milestones
        this.recalculatePhaseProgress(phaseId);
      }
    }
  }

  recalculatePhaseProgress(phaseId) {
    const phase = this.phases.find((p) => p.id === phaseId);
    if (phase) {
      const completedMilestones = phase.milestones.filter(
        (m) => m.completed,
      ).length;
      const totalMilestones = phase.milestones.length;
      const calculatedProgress = Math.round(
        (completedMilestones / totalMilestones) * 100,
      );

      // Update progress if it's significantly different
      if (Math.abs(calculatedProgress - phase.progress) > 5) {
        this.updatePhaseProgress(phaseId, calculatedProgress);
      }
    }
  }
}

// CSS styles for the roadmap progress system
const roadmapProgressCSS = `
/* Roadmap Progress Styles */
.roadmap-progress-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: var(--space-8) var(--space-4);
}

.roadmap-phase {
    background-color: var(--bg-surface);
    border-radius: var(--radius-xl);
    padding: var(--space-8);
    margin-bottom: var(--space-8);
    border: 2px solid transparent;
    transition: all var(--transition-base);
    position: relative;
    overflow: hidden;
    opacity: 0;
    transform: translateY(30px);
}

.roadmap-phase.animate-in {
    opacity: 1;
    transform: translateY(0);
    transition: all var(--transition-slow) ease-out;
}

.roadmap-phase.phase-highlighted {
    border-color: var(--color-primary);
    transform: translateY(-4px);
    box-shadow: var(--shadow-lg);
}

.roadmap-phase::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 4px;
    transition: background-color var(--transition-base);
}

.roadmap-phase.phase-completed::before {
    background-color: var(--color-green);
}

.roadmap-phase.phase-in-progress::before {
    background-color: var(--color-blue);
}

.roadmap-phase.phase-planning::before {
    background-color: var(--color-overlay1);
}

.phase-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-6);
    flex-wrap: wrap;
    gap: var(--space-4);
}

.phase-info {
    flex: 1;
    min-width: 300px;
}

.phase-title {
    margin-bottom: var(--space-2);
    color: var(--text-primary);
    font-size: var(--font-size-xl);
    font-weight: var(--font-bold);
}

.phase-meta {
    display: flex;
    gap: var(--space-4);
    align-items: center;
    flex-wrap: wrap;
}

.phase-quarter {
    background-color: var(--color-surface1);
    color: var(--text-secondary);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-full);
    font-size: var(--font-size-sm);
    font-weight: var(--font-medium);
}

.phase-status {
    font-size: var(--font-size-sm);
    font-weight: var(--font-medium);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-full);
}

.phase-completed .phase-status {
    background-color: var(--color-green);
    color: var(--text-on-primary);
}

.phase-in-progress .phase-status {
    background-color: var(--color-blue);
    color: var(--text-on-primary);
}

.phase-planning .phase-status {
    background-color: var(--color-surface1);
    color: var(--text-secondary);
}

.phase-progress-info {
    text-align: right;
    min-width: 120px;
}

.progress-percentage {
    font-size: var(--font-size-2xl);
    font-weight: var(--font-bold);
    color: var(--text-primary);
    display: block;
}

.estimated-completion {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin-top: var(--space-1);
}

.progress-bar-container {
    margin-bottom: var(--space-6);
}

.progress-bar-track {
    width: 100%;
    height: 12px;
    background-color: var(--color-surface1);
    border-radius: var(--radius-full);
    overflow: hidden;
    position: relative;
}

.progress-bar-fill {
    height: 100%;
    border-radius: var(--radius-full);
    transition: width var(--transition-slower) cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
}

.progress-bar-fill::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
        90deg,
        transparent,
        rgba(255, 255, 255, 0.2),
        transparent
    );
    transform: translateX(-100%);
    animation: progressShimmer 2s infinite;
}

@keyframes progressShimmer {
    0% { transform: translateX(-100%); }
    50% { transform: translateX(100%); }
    100% { transform: translateX(100%); }
}

.progress-complete {
    background-color: var(--color-green);
}

.progress-active {
    background: linear-gradient(90deg, var(--color-blue), var(--color-sapphire));
}

.progress-planning {
    background-color: var(--color-overlay1);
}

.phase-description {
    color: var(--text-secondary);
    margin-bottom: var(--space-6);
    line-height: var(--line-height-relaxed);
}

.milestones-section {
    margin-top: var(--space-6);
}

.milestones-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-4);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    user-select: none;
    transition: color var(--transition-fast);
}

.milestones-title::before {
    content: '▶';
    font-size: var(--font-size-sm);
    transition: transform var(--transition-fast);
}

.milestones-title:hover {
    color: var(--color-primary);
}

.milestones-list {
    list-style: none;
    max-height: 0;
    overflow: hidden;
    transition: max-height var(--transition-base) ease-out;
}

.milestones-list.expanded {
    max-height: 500px;
}

.milestones-list.expanded ~ .milestones-title::before {
    transform: rotate(90deg);
}

.milestone-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) 0;
    transition: all var(--transition-fast);
}

.milestone-item:hover {
    padding-left: var(--space-2);
}

.milestone-icon {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--font-size-sm);
    font-weight: var(--font-bold);
    flex-shrink: 0;
    transition: all var(--transition-fast);
}

.milestone-item.completed .milestone-icon {
    background-color: var(--color-green);
    color: var(--text-on-primary);
}

.milestone-item.pending .milestone-icon {
    background-color: var(--color-surface1);
    color: var(--text-secondary);
}

.milestone-name {
    color: var(--text-secondary);
    transition: color var(--transition-fast);
}

.milestone-item.completed .milestone-name {
    color: var(--text-primary);
}

/* Responsive Design */
@media (max-width: 768px) {
    .roadmap-progress-container {
        padding: var(--space-4) var(--space-2);
    }

    .roadmap-phase {
        padding: var(--space-6);
        margin-bottom: var(--space-6);
    }

    .phase-header {
        flex-direction: column;
        align-items: stretch;
    }

    .phase-progress-info {
        text-align: left;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .progress-percentage {
        font-size: var(--font-size-xl);
    }
}

/* Print Styles */
@media print {
    .roadmap-phase {
        break-inside: avoid;
        box-shadow: none;
        border: 1px solid #ccc;
    }

    .progress-bar-fill::after {
        display: none;
    }
}

/* High Contrast Mode */
@media (prefers-contrast: high) {
    .roadmap-phase {
        border: 2px solid currentColor;
    }

    .progress-bar-track {
        border: 1px solid currentColor;
    }
}

/* Reduced Motion */
@media (prefers-reduced-motion: reduce) {
    .progress-bar-fill,
    .roadmap-phase,
    .milestone-item {
        transition: none;
    }

    .progress-bar-fill::after {
        animation: none;
    }

    .roadmap-phase.animate-in {
        opacity: 1;
        transform: none;
    }
}
`;

// Inject CSS styles
const styleSheet = document.createElement("style");
styleSheet.textContent = roadmapProgressCSS;
document.head.appendChild(styleSheet);

// Initialize the roadmap progress system when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    window.roadmapProgress = new RoadmapProgress();
  });
} else {
  window.roadmapProgress = new RoadmapProgress();
}

// Export for module usage
if (typeof module !== "undefined" && module.exports) {
  module.exports = RoadmapProgress;
}
