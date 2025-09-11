/**
 * Caxton Architecture Diagrams
 * Interactive SVG diagrams using Catppuccin Mocha color scheme
 */

class ArchitectureDiagrams {
  constructor() {
    this.colors = {
      // Catppuccin Mocha colors from design system
      base: '#1e1e2e',
      mantle: '#181825',
      crust: '#11111b',
      surface0: '#313244',
      surface1: '#45475a',
      surface2: '#585b70',
      text: '#cdd6f4',
      subtext1: '#bac2de',
      subtext0: '#a6adc8',
      overlay2: '#9399b2',
      overlay1: '#7f849c',
      overlay0: '#6c7086',
      blue: '#89b4fa',
      lavender: '#b4befe',
      sapphire: '#74c7ec',
      sky: '#89dceb',
      teal: '#94e2d5',
      green: '#a6e3a1',
      yellow: '#f9e2af',
      peach: '#fab387',
      maroon: '#eba0ac',
      red: '#f38ba8',
      mauve: '#cba6f7',
      pink: '#f5c2e7',
      flamingo: '#f2cdcd',
      rosewater: '#f5e0dc'
    };

    this.diagrams = new Map();
    this.animationEnabled = !window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }

  /**
   * Initialize all diagrams on the page
   */
  init() {
    // Find all diagram containers
    const containers = document.querySelectorAll('[data-diagram]');
    containers.forEach(container => {
      const diagramType = container.dataset.diagram;
      const method = `create${diagramType.charAt(0).toUpperCase() + diagramType.slice(1)}Diagram`;

      if (this[method]) {
        this[method](container);
      }
    });
  }

  /**
   * Create WebAssembly Agent Isolation Architecture diagram
   */
  createWasmIsolationDiagram(container) {
    const svg = this.createSVGElement(container, 800, 600);

    // Title
    this.addText(svg, 400, 30, 'WebAssembly Agent Isolation Architecture', {
      fontSize: '18px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.text
    });

    // Host System
    this.addRect(svg, 50, 80, 700, 480, {
      fill: this.colors.surface0,
      stroke: this.colors.overlay0,
      strokeWidth: 2,
      rx: 8
    });
    this.addText(svg, 60, 100, 'Host System (Caxton Runtime)', {
      fontSize: '14px',
      fontWeight: 'bold',
      fill: this.colors.subtext1
    });

    // Runtime Core
    this.addRect(svg, 80, 120, 280, 120, {
      fill: this.colors.surface1,
      stroke: this.colors.blue,
      strokeWidth: 2,
      rx: 6
    });
    this.addText(svg, 220, 140, 'Runtime Core', {
      fontSize: '14px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.blue
    });

    // Core components
    this.addRect(svg, 90, 160, 80, 30, {
      fill: this.colors.surface2,
      stroke: this.colors.sapphire,
      strokeWidth: 1,
      rx: 4
    });
    this.addText(svg, 130, 178, 'Scheduler', {
      fontSize: '11px',
      textAnchor: 'middle',
      fill: this.colors.sapphire
    });

    this.addRect(svg, 180, 160, 80, 30, {
      fill: this.colors.surface2,
      stroke: this.colors.sapphire,
      strokeWidth: 1,
      rx: 4
    });
    this.addText(svg, 220, 178, 'Message Bus', {
      fontSize: '11px',
      textAnchor: 'middle',
      fill: this.colors.sapphire
    });

    this.addRect(svg, 270, 160, 80, 30, {
      fill: this.colors.surface2,
      stroke: this.colors.sapphire,
      strokeWidth: 1,
      rx: 4
    });
    this.addText(svg, 310, 178, 'Resource Mgr', {
      fontSize: '11px',
      textAnchor: 'middle',
      fill: this.colors.sapphire
    });

    // WASM Sandbox Instances
    const agents = [
      { x: 400, y: 120, name: 'Agent A', color: this.colors.green },
      { x: 580, y: 120, name: 'Agent B', color: this.colors.yellow },
      { x: 400, y: 280, name: 'Agent C', color: this.colors.pink },
      { x: 580, y: 280, name: 'Agent D', color: this.colors.mauve }
    ];

    agents.forEach((agent, index) => {
      // Sandbox container
      this.addRect(svg, agent.x, agent.y, 140, 140, {
        fill: this.colors.surface1,
        stroke: agent.color,
        strokeWidth: 2,
        rx: 6,
        strokeDasharray: '5,5'
      });

      this.addText(svg, agent.x + 70, agent.y + 20, `WASM Sandbox ${index + 1}`, {
        fontSize: '12px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: agent.color
      });

      // Agent instance
      this.addRect(svg, agent.x + 10, agent.y + 30, 120, 45, {
        fill: this.colors.surface2,
        stroke: agent.color,
        strokeWidth: 1,
        rx: 4
      });
      this.addText(svg, agent.x + 70, agent.y + 55, agent.name, {
        fontSize: '12px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: agent.color
      });

      // Memory isolation
      this.addRect(svg, agent.x + 10, agent.y + 85, 120, 25, {
        fill: this.colors.crust,
        stroke: this.colors.overlay1,
        strokeWidth: 1,
        rx: 3
      });
      this.addText(svg, agent.x + 70, agent.y + 100, 'Isolated Memory', {
        fontSize: '10px',
        textAnchor: 'middle',
        fill: this.colors.overlay1
      });

      // File system
      this.addRect(svg, agent.x + 10, agent.y + 115, 120, 25, {
        fill: this.colors.crust,
        stroke: this.colors.overlay1,
        strokeWidth: 1,
        rx: 3
      });
      this.addText(svg, agent.x + 70, agent.y + 130, 'Virtual FS', {
        fontSize: '10px',
        textAnchor: 'middle',
        fill: this.colors.overlay1
      });

      // Connection to message bus
      this.addLine(svg, 260, 175, agent.x, agent.y + 52, {
        stroke: this.colors.blue,
        strokeWidth: 2,
        opacity: 0.7
      });

      if (this.animationEnabled) {
        // Add flowing data animation
        const animLine = this.addLine(svg, 260, 175, agent.x, agent.y + 52, {
          stroke: this.colors.sky,
          strokeWidth: 3,
          opacity: 0,
          strokeDasharray: '10,5'
        });

        animLine.innerHTML = `
          <animate attributeName="opacity" values="0;1;0" dur="2s" repeatCount="indefinite" begin="${index * 0.5}s"/>
          <animateTransform attributeName="transform" type="translate" values="0,0;10,5;0,0" dur="2s" repeatCount="indefinite" begin="${index * 0.5}s"/>
        `;
      }
    });

    // Security boundary
    this.addRect(svg, 75, 115, 670, 430, {
      fill: 'none',
      stroke: this.colors.red,
      strokeWidth: 3,
      strokeDasharray: '10,10',
      opacity: 0.7,
      rx: 8
    });
    this.addText(svg, 400, 555, 'Security Boundary', {
      fontSize: '12px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.red
    });

    // Add tooltips
    this.addTooltips(svg, [
      { element: agents[0], text: 'Each agent runs in an isolated WASM sandbox with restricted system access' },
      { x: 220, y: 175, text: 'Message Bus handles secure inter-agent communication via agent messaging protocols' },
      { x: 310, y: 175, text: 'Resource Manager enforces memory and CPU limits per agent' }
    ]);

    this.diagrams.set('wasm-isolation', svg);
  }

  /**
   * Create Agent Message Flow diagram
   */
  createFipaMessageFlowDiagram(container) {
    const svg = this.createSVGElement(container, 900, 700);

    // Title
    this.addText(svg, 450, 30, 'Agent Message Flow Architecture', {
      fontSize: '18px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.text
    });

    // Agents
    const agents = [
      { x: 100, y: 100, name: 'Initiator\nAgent', color: this.colors.blue },
      { x: 400, y: 100, name: 'Participant\nAgent', color: this.colors.green },
      { x: 700, y: 100, name: 'Facilitator\nAgent', color: this.colors.yellow }
    ];

    agents.forEach(agent => {
      this.addCircle(svg, agent.x, agent.y, 40, {
        fill: agent.color,
        stroke: this.colors.text,
        strokeWidth: 2,
        opacity: 0.8
      });

      this.addText(svg, agent.x, agent.y, agent.name, {
        fontSize: '11px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.base,
        dominantBaseline: 'middle'
      });
    });

    // Message Bus
    this.addRect(svg, 50, 250, 800, 80, {
      fill: this.colors.surface0,
      stroke: this.colors.overlay0,
      strokeWidth: 2,
      rx: 8
    });

    this.addText(svg, 450, 270, 'Agent Message Bus', {
      fontSize: '16px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.text
    });

    // Message types
    const messageTypes = [
      { x: 120, y: 300, text: 'REQUEST', color: this.colors.blue },
      { x: 220, y: 300, text: 'INFORM', color: this.colors.green },
      { x: 320, y: 300, text: 'PROPOSE', color: this.colors.yellow },
      { x: 420, y: 300, text: 'ACCEPT', color: this.colors.teal },
      { x: 520, y: 300, text: 'REJECT', color: this.colors.red },
      { x: 620, y: 300, text: 'CFP', color: this.colors.mauve },
      { x: 720, y: 300, text: 'CANCEL', color: this.colors.maroon },
      { x: 780, y: 300, text: 'QUERY', color: this.colors.pink }
    ];

    messageTypes.forEach(msg => {
      this.addRect(svg, msg.x - 25, msg.y - 10, 50, 20, {
        fill: msg.color,
        rx: 10,
        opacity: 0.7
      });

      this.addText(svg, msg.x, msg.y, msg.text, {
        fontSize: '9px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.base,
        dominantBaseline: 'middle'
      });
    });

    // Message flow arrows
    const flows = [
      { from: { x: 100, y: 140 }, to: { x: 450, y: 250 }, label: '1. CFP', color: this.colors.mauve },
      { from: { x: 450, y: 250 }, to: { x: 400, y: 140 }, label: '2. PROPOSE', color: this.colors.yellow },
      { from: { x: 450, y: 250 }, to: { x: 700, y: 140 }, label: '3. PROPOSE', color: this.colors.yellow },
      { from: { x: 100, y: 140 }, to: { x: 450, y: 250 }, label: '4. ACCEPT', color: this.colors.teal, offset: 20 },
      { from: { x: 450, y: 250 }, to: { x: 400, y: 140 }, label: '5. INFORM', color: this.colors.green, offset: 20 }
    ];

    flows.forEach((flow, index) => {
      const offsetY = flow.offset || 0;
      const midY = (flow.from.y + flow.to.y) / 2 + offsetY;

      this.addPath(svg, `M ${flow.from.x} ${flow.from.y + offsetY} Q ${(flow.from.x + flow.to.x) / 2} ${midY} ${flow.to.x} ${flow.to.y + offsetY}`, {
        fill: 'none',
        stroke: flow.color,
        strokeWidth: 3,
        markerEnd: 'url(#arrowhead)',
        opacity: 0.8
      });

      // Message label
      this.addText(svg, (flow.from.x + flow.to.x) / 2, midY - 10, flow.label, {
        fontSize: '11px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: flow.color
      });

      if (this.animationEnabled) {
        // Animate message flow
        const animCircle = this.addCircle(svg, flow.from.x, flow.from.y + offsetY, 4, {
          fill: flow.color,
          opacity: 0
        });

        animCircle.innerHTML = `
          <animate attributeName="opacity" values="0;1;1;0" dur="3s" repeatCount="indefinite" begin="${index * 0.6}s"/>
          <animateMotion dur="3s" repeatCount="indefinite" begin="${index * 0.6}s">
            <mpath href="#path${index}"/>
          </animateMotion>
        `;

        // Hidden path for animation
        this.addPath(svg, `M ${flow.from.x} ${flow.from.y + offsetY} Q ${(flow.from.x + flow.to.x) / 2} ${midY} ${flow.to.x} ${flow.to.y + offsetY}`, {
          id: `path${index}`,
          fill: 'none',
          stroke: 'none'
        });
      }
    });

    // Contract Net Protocol steps
    this.addRect(svg, 50, 400, 800, 250, {
      fill: this.colors.surface0,
      stroke: this.colors.overlay0,
      strokeWidth: 2,
      rx: 8
    });

    this.addText(svg, 450, 430, 'Contract Net Protocol Flow', {
      fontSize: '16px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.text
    });

    const steps = [
      { step: 1, text: 'Initiator broadcasts\nCall for Proposals (CFP)', y: 470 },
      { step: 2, text: 'Participants evaluate\nand send PROPOSE', y: 510 },
      { step: 3, text: 'Initiator selects best\nproposal and sends ACCEPT', y: 550 },
      { step: 4, text: 'Rejected participants\nreceive REJECT messages', y: 590 },
      { step: 5, text: 'Winner completes task\nand sends INFORM result', y: 630 }
    ];

    steps.forEach(step => {
      this.addCircle(svg, 100, step.y, 15, {
        fill: this.colors.blue,
        stroke: this.colors.text,
        strokeWidth: 2
      });

      this.addText(svg, 100, step.y, step.step.toString(), {
        fontSize: '12px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.text,
        dominantBaseline: 'middle'
      });

      this.addText(svg, 140, step.y, step.text, {
        fontSize: '12px',
        fill: this.colors.subtext1,
        dominantBaseline: 'middle'
      });
    });

    // Add arrowhead marker
    const defs = svg.querySelector('defs') || this.addDefs(svg);
    defs.innerHTML += `
      <marker id="arrowhead" markerWidth="10" markerHeight="7"
              refX="9" refY="3.5" orient="auto" markerUnits="strokeWidth">
        <polygon points="0 0, 10 3.5, 0 7" fill="${this.colors.text}" />
      </marker>
    `;

    this.diagrams.set('agent-message-flow', svg);
  }

  /**
   * Create OpenTelemetry Observability Pipeline diagram
   */
  createObservabilityPipelineDiagram(container) {
    const svg = this.createSVGElement(container, 1000, 800);

    // Title
    this.addText(svg, 500, 30, 'OpenTelemetry Observability Pipeline', {
      fontSize: '18px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.text
    });

    // Data sources layer
    this.addRect(svg, 50, 80, 900, 120, {
      fill: this.colors.surface0,
      stroke: this.colors.overlay0,
      strokeWidth: 2,
      rx: 8
    });

    this.addText(svg, 70, 105, 'Data Sources', {
      fontSize: '14px',
      fontWeight: 'bold',
      fill: this.colors.subtext1
    });

    const sources = [
      { x: 120, y: 140, name: 'Agent\nMetrics', color: this.colors.blue },
      { x: 250, y: 140, name: 'Message\nTraces', color: this.colors.green },
      { x: 380, y: 140, name: 'System\nLogs', color: this.colors.yellow },
      { x: 510, y: 140, name: 'Resource\nUsage', color: this.colors.pink },
      { x: 640, y: 140, name: 'Performance\nCounters', color: this.colors.mauve },
      { x: 770, y: 140, name: 'Error\nEvents', color: this.colors.red },
      { x: 900, y: 140, name: 'Custom\nInstruments', color: this.colors.teal }
    ];

    sources.forEach(source => {
      this.addRect(svg, source.x - 40, source.y - 25, 80, 50, {
        fill: source.color,
        stroke: this.colors.text,
        strokeWidth: 1,
        rx: 6,
        opacity: 0.8
      });

      this.addText(svg, source.x, source.y, source.name, {
        fontSize: '10px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.base,
        dominantBaseline: 'middle'
      });
    });

    // OpenTelemetry Collector
    this.addRect(svg, 200, 250, 600, 100, {
      fill: this.colors.surface1,
      stroke: this.colors.sapphire,
      strokeWidth: 3,
      rx: 8
    });

    this.addText(svg, 500, 280, 'OpenTelemetry Collector', {
      fontSize: '16px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.sapphire
    });

    // Collector components
    const components = [
      { x: 250, y: 320, name: 'Receivers', color: this.colors.green },
      { x: 380, y: 320, name: 'Processors', color: this.colors.yellow },
      { x: 500, y: 320, name: 'Samplers', color: this.colors.pink },
      { x: 620, y: 320, name: 'Exporters', color: this.colors.blue },
      { x: 750, y: 320, name: 'Extensions', color: this.colors.mauve }
    ];

    components.forEach(comp => {
      this.addRect(svg, comp.x - 35, comp.y - 15, 70, 30, {
        fill: comp.color,
        stroke: this.colors.text,
        strokeWidth: 1,
        rx: 4,
        opacity: 0.7
      });

      this.addText(svg, comp.x, comp.y, comp.name, {
        fontSize: '11px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.base,
        dominantBaseline: 'middle'
      });
    });

    // Processing pipeline arrows
    const pipelineArrows = [
      { from: { x: 285, y: 320 }, to: { x: 345, y: 320 } },
      { from: { x: 415, y: 320 }, to: { x: 465, y: 320 } },
      { from: { x: 535, y: 320 }, to: { x: 585, y: 320 } },
      { from: { x: 655, y: 320 }, to: { x: 715, y: 320 } }
    ];

    pipelineArrows.forEach(arrow => {
      this.addLine(svg, arrow.from.x, arrow.from.y, arrow.to.x, arrow.to.y, {
        stroke: this.colors.sapphire,
        strokeWidth: 3,
        markerEnd: 'url(#arrowhead-small)'
      });
    });

    // Backend systems
    this.addRect(svg, 50, 450, 900, 300, {
      fill: this.colors.surface0,
      stroke: this.colors.overlay0,
      strokeWidth: 2,
      rx: 8
    });

    this.addText(svg, 70, 475, 'Observability Backends', {
      fontSize: '14px',
      fontWeight: 'bold',
      fill: this.colors.subtext1
    });

    // Metrics backend (Prometheus)
    this.addRect(svg, 80, 500, 250, 120, {
      fill: this.colors.surface1,
      stroke: this.colors.red,
      strokeWidth: 2,
      rx: 6
    });

    this.addText(svg, 205, 530, 'Metrics Backend', {
      fontSize: '14px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.red
    });

    this.addText(svg, 205, 555, '(Prometheus)', {
      fontSize: '12px',
      textAnchor: 'middle',
      fill: this.colors.red
    });

    this.addText(svg, 205, 585, '• Counter metrics\n• Gauge metrics\n• Histogram metrics', {
      fontSize: '10px',
      textAnchor: 'middle',
      fill: this.colors.subtext0,
      dominantBaseline: 'middle'
    });

    // Tracing backend (Jaeger)
    this.addRect(svg, 370, 500, 250, 120, {
      fill: this.colors.surface1,
      stroke: this.colors.blue,
      strokeWidth: 2,
      rx: 6
    });

    this.addText(svg, 495, 530, 'Tracing Backend', {
      fontSize: '14px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.blue
    });

    this.addText(svg, 495, 555, '(Jaeger)', {
      fontSize: '12px',
      textAnchor: 'middle',
      fill: this.colors.blue
    });

    this.addText(svg, 495, 585, '• Distributed traces\n• Span relationships\n• Latency analysis', {
      fontSize: '10px',
      textAnchor: 'middle',
      fill: this.colors.subtext0,
      dominantBaseline: 'middle'
    });

    // Logging backend (Loki)
    this.addRect(svg, 660, 500, 250, 120, {
      fill: this.colors.surface1,
      stroke: this.colors.yellow,
      strokeWidth: 2,
      rx: 6
    });

    this.addText(svg, 785, 530, 'Logging Backend', {
      fontSize: '14px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.yellow
    });

    this.addText(svg, 785, 555, '(Loki)', {
      fontSize: '12px',
      textAnchor: 'middle',
      fill: this.colors.yellow
    });

    this.addText(svg, 785, 585, '• Structured logs\n• Log aggregation\n• Query interface', {
      fontSize: '10px',
      textAnchor: 'middle',
      fill: this.colors.subtext0,
      dominantBaseline: 'middle'
    });

    // Visualization layer
    this.addRect(svg, 200, 670, 600, 80, {
      fill: this.colors.surface1,
      stroke: this.colors.green,
      strokeWidth: 2,
      rx: 8
    });

    this.addText(svg, 500, 700, 'Visualization & Alerting (Grafana)', {
      fontSize: '16px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.green
    });

    this.addText(svg, 500, 725, 'Dashboards • Alerts • Query Builder • Data Correlation', {
      fontSize: '12px',
      textAnchor: 'middle',
      fill: this.colors.subtext0
    });

    // Data flow arrows
    sources.forEach(source => {
      this.addLine(svg, source.x, source.y + 25, source.x, 250, {
        stroke: this.colors.overlay1,
        strokeWidth: 2,
        strokeDasharray: '5,3',
        markerEnd: 'url(#arrowhead-small)'
      });
    });

    // Collector to backends
    const collectorOutputs = [
      { from: { x: 300, y: 350 }, to: { x: 205, y: 500 }, color: this.colors.red },
      { from: { x: 500, y: 350 }, to: { x: 495, y: 500 }, color: this.colors.blue },
      { from: { x: 700, y: 350 }, to: { x: 785, y: 500 }, color: this.colors.yellow }
    ];

    collectorOutputs.forEach(output => {
      this.addLine(svg, output.from.x, output.from.y, output.to.x, output.to.y, {
        stroke: output.color,
        strokeWidth: 3,
        markerEnd: 'url(#arrowhead-small)'
      });
    });

    // Backends to visualization
    [205, 495, 785].forEach(x => {
      this.addLine(svg, x, 620, 500, 670, {
        stroke: this.colors.green,
        strokeWidth: 2,
        markerEnd: 'url(#arrowhead-small)'
      });
    });

    // Add small arrowhead marker
    const defs = svg.querySelector('defs') || this.addDefs(svg);
    defs.innerHTML += `
      <marker id="arrowhead-small" markerWidth="8" markerHeight="6"
              refX="7" refY="3" orient="auto" markerUnits="strokeWidth">
        <polygon points="0 0, 8 3, 0 6" fill="currentColor" />
      </marker>
    `;

    // Add data flow animation
    if (this.animationEnabled) {
      sources.forEach((source, index) => {
        const animDot = this.addCircle(svg, source.x, source.y + 25, 3, {
          fill: this.colors.sky,
          opacity: 0
        });

        animDot.innerHTML = `
          <animate attributeName="opacity" values="0;1;1;0" dur="4s" repeatCount="indefinite" begin="${index * 0.3}s"/>
          <animate attributeName="cy" values="${source.y + 25};250" dur="2s" repeatCount="indefinite" begin="${index * 0.3}s"/>
        `;
      });
    }

    this.diagrams.set('observability-pipeline', svg);
  }

  /**
   * Create Multi-language Runtime Support diagram
   */
  createMultiLanguageDiagram(container) {
    const svg = this.createSVGElement(container, 900, 600);

    // Title
    this.addText(svg, 450, 30, 'Multi-Language Runtime Support Architecture', {
      fontSize: '18px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.text
    });

    // Common Runtime Interface
    this.addRect(svg, 50, 80, 800, 80, {
      fill: this.colors.surface0,
      stroke: this.colors.blue,
      strokeWidth: 3,
      rx: 8
    });

    this.addText(svg, 450, 110, 'Common Runtime Interface (WASI)', {
      fontSize: '16px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.blue
    });

    this.addText(svg, 450, 135, 'System Calls • File I/O • Network • Threading • Memory Management', {
      fontSize: '12px',
      textAnchor: 'middle',
      fill: this.colors.subtext0
    });

    // Language runtimes
    const languages = [
      { x: 150, y: 250, name: 'Rust', logo: '🦀', color: this.colors.red, features: ['Zero-cost abstractions', 'Memory safety', 'Native performance'] },
      { x: 350, y: 250, name: 'JavaScript', logo: '⚡', color: this.colors.yellow, features: ['V8 engine', 'JIT compilation', 'Dynamic typing'] },
      { x: 550, y: 250, name: 'Python', logo: '🐍', color: this.colors.green, features: ['Interpreted', 'Dynamic binding', 'Rich ecosystem'] },
      { x: 750, y: 250, name: 'Go', logo: '🚀', color: this.colors.sapphire, features: ['Garbage collected', 'Concurrency', 'Fast compilation'] }
    ];

    languages.forEach(lang => {
      // Runtime container
      this.addRect(svg, lang.x - 75, lang.y - 50, 150, 180, {
        fill: this.colors.surface1,
        stroke: lang.color,
        strokeWidth: 2,
        rx: 8
      });

      // Language header
      this.addRect(svg, lang.x - 70, lang.y - 45, 140, 40, {
        fill: lang.color,
        rx: 6
      });

      this.addText(svg, lang.x - 40, lang.y - 25, lang.logo, {
        fontSize: '20px',
        textAnchor: 'middle',
        dominantBaseline: 'middle'
      });

      this.addText(svg, lang.x + 10, lang.y - 25, lang.name, {
        fontSize: '14px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.base,
        dominantBaseline: 'middle'
      });

      // Runtime components
      this.addRect(svg, lang.x - 65, lang.y, 130, 30, {
        fill: this.colors.surface2,
        stroke: this.colors.overlay1,
        strokeWidth: 1,
        rx: 4
      });

      this.addText(svg, lang.x, lang.y + 15, 'Runtime Engine', {
        fontSize: '11px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.text,
        dominantBaseline: 'middle'
      });

      this.addRect(svg, lang.x - 65, lang.y + 40, 130, 30, {
        fill: this.colors.surface2,
        stroke: this.colors.overlay1,
        strokeWidth: 1,
        rx: 4
      });

      this.addText(svg, lang.x, lang.y + 55, 'Standard Library', {
        fontSize: '11px',
        fontWeight: 'bold',
        textAnchor: 'middle',
        fill: this.colors.text,
        dominantBaseline: 'middle'
      });

      // Features
      lang.features.forEach((feature, index) => {
        this.addText(svg, lang.x, lang.y + 90 + (index * 15), `• ${feature}`, {
          fontSize: '9px',
          textAnchor: 'middle',
          fill: this.colors.subtext1
        });
      });

      // Connection to runtime interface
      this.addLine(svg, lang.x, lang.y - 50, lang.x, 160, {
        stroke: lang.color,
        strokeWidth: 3,
        markerEnd: 'url(#arrowhead-up)'
      });
    });

    // WebAssembly Compilation Layer
    this.addRect(svg, 50, 450, 800, 80, {
      fill: this.colors.surface0,
      stroke: this.colors.mauve,
      strokeWidth: 3,
      rx: 8
    });

    this.addText(svg, 450, 480, 'WebAssembly Compilation & Execution', {
      fontSize: '16px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.mauve
    });

    this.addText(svg, 450, 505, 'WASM Bytecode • Just-In-Time Compilation • Sandboxed Execution', {
      fontSize: '12px',
      textAnchor: 'middle',
      fill: this.colors.subtext0
    });

    // Connections to WASM layer
    languages.forEach(lang => {
      this.addLine(svg, lang.x, lang.y + 130, lang.x, 450, {
        stroke: this.colors.mauve,
        strokeWidth: 2,
        strokeDasharray: '5,5',
        markerEnd: 'url(#arrowhead-small)'
      });
    });

    // Cross-language communication
    this.addRect(svg, 200, 360, 500, 50, {
      fill: this.colors.surface1,
      stroke: this.colors.pink,
      strokeWidth: 2,
      rx: 6
    });

    this.addText(svg, 450, 385, 'Inter-Language Communication Bridge', {
      fontSize: '14px',
      fontWeight: 'bold',
      textAnchor: 'middle',
      fill: this.colors.pink
    });

    // Bidirectional arrows between languages
    const connections = [
      { from: { x: 225, y: 360 }, to: { x: 325, y: 360 } },
      { from: { x: 375, y: 360 }, to: { x: 475, y: 360 } },
      { from: { x: 525, y: 360 }, to: { x: 625, y: 360 } }
    ];

    connections.forEach(conn => {
      // Forward arrow
      this.addLine(svg, conn.from.x, conn.from.y - 5, conn.to.x, conn.to.y - 5, {
        stroke: this.colors.pink,
        strokeWidth: 2,
        markerEnd: 'url(#arrowhead-small)'
      });

      // Backward arrow
      this.addLine(svg, conn.to.x, conn.to.y + 5, conn.from.x, conn.from.y + 5, {
        stroke: this.colors.pink,
        strokeWidth: 2,
        markerEnd: 'url(#arrowhead-small)'
      });
    });

    // Add upward pointing arrowhead marker
    const defs = svg.querySelector('defs') || this.addDefs(svg);
    defs.innerHTML += `
      <marker id="arrowhead-up" markerWidth="10" markerHeight="7"
              refX="5" refY="7" orient="auto" markerUnits="strokeWidth">
        <polygon points="0 7, 5 0, 10 7" fill="currentColor" />
      </marker>
    `;

    // Add performance indicators with animation
    if (this.animationEnabled) {
      languages.forEach((lang, index) => {
        const perfIndicator = this.addCircle(svg, lang.x + 60, lang.y - 30, 8, {
          fill: lang.color,
          opacity: 0.3
        });

        perfIndicator.innerHTML = `
          <animate attributeName="r" values="5;12;5" dur="2s" repeatCount="indefinite" begin="${index * 0.5}s"/>
          <animate attributeName="opacity" values="0.7;0.2;0.7" dur="2s" repeatCount="indefinite" begin="${index * 0.5}s"/>
        `;
      });
    }

    this.diagrams.set('multi-language', svg);
  }

  /**
   * Helper method to create SVG element
   */
  createSVGElement(container, width, height) {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('width', '100%');
    svg.setAttribute('height', height);
    svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
    svg.setAttribute('role', 'img');
    svg.setAttribute('aria-labelledby', 'diagram-title');
    svg.style.background = this.colors.base;
    svg.style.borderRadius = '8px';
    svg.style.border = `2px solid ${this.colors.surface1}`;

    // Add defs for gradients and patterns
    this.addDefs(svg);

    container.appendChild(svg);
    return svg;
  }

  /**
   * Add defs element for reusable components
   */
  addDefs(svg) {
    const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
    svg.appendChild(defs);
    return defs;
  }

  /**
   * Add rectangle element
   */
  addRect(svg, x, y, width, height, attributes = {}) {
    const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    rect.setAttribute('x', x);
    rect.setAttribute('y', y);
    rect.setAttribute('width', width);
    rect.setAttribute('height', height);

    Object.entries(attributes).forEach(([key, value]) => {
      rect.setAttribute(key, value);
    });

    svg.appendChild(rect);
    return rect;
  }

  /**
   * Add circle element
   */
  addCircle(svg, cx, cy, r, attributes = {}) {
    const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    circle.setAttribute('cx', cx);
    circle.setAttribute('cy', cy);
    circle.setAttribute('r', r);

    Object.entries(attributes).forEach(([key, value]) => {
      circle.setAttribute(key, value);
    });

    svg.appendChild(circle);
    return circle;
  }

  /**
   * Add text element
   */
  addText(svg, x, y, text, attributes = {}) {
    const textElement = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    textElement.setAttribute('x', x);
    textElement.setAttribute('y', y);

    // Handle multiline text
    if (text.includes('\n')) {
      const lines = text.split('\n');
      lines.forEach((line, index) => {
        const tspan = document.createElementNS('http://www.w3.org/2000/svg', 'tspan');
        tspan.setAttribute('x', x);
        tspan.setAttribute('dy', index === 0 ? 0 : '1.2em');
        tspan.textContent = line;
        textElement.appendChild(tspan);
      });
    } else {
      textElement.textContent = text;
    }

    Object.entries(attributes).forEach(([key, value]) => {
      textElement.setAttribute(key, value);
    });

    svg.appendChild(textElement);
    return textElement;
  }

  /**
   * Add line element
   */
  addLine(svg, x1, y1, x2, y2, attributes = {}) {
    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    line.setAttribute('x1', x1);
    line.setAttribute('y1', y1);
    line.setAttribute('x2', x2);
    line.setAttribute('y2', y2);

    Object.entries(attributes).forEach(([key, value]) => {
      line.setAttribute(key, value);
    });

    svg.appendChild(line);
    return line;
  }

  /**
   * Add path element
   */
  addPath(svg, d, attributes = {}) {
    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('d', d);

    Object.entries(attributes).forEach(([key, value]) => {
      path.setAttribute(key, value);
    });

    svg.appendChild(path);
    return path;
  }

  /**
   * Add tooltips to diagram elements
   */
  addTooltips(svg, tooltipData) {
    const tooltip = document.createElement('div');
    tooltip.className = 'architecture-tooltip';
    tooltip.style.cssText = `
      position: absolute;
      background: ${this.colors.surface1};
      color: ${this.colors.text};
      padding: 8px 12px;
      border-radius: 6px;
      font-size: 12px;
      pointer-events: none;
      opacity: 0;
      transition: opacity 0.2s ease;
      z-index: 1000;
      max-width: 300px;
      border: 1px solid ${this.colors.overlay0};
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    `;
    document.body.appendChild(tooltip);

    tooltipData.forEach(data => {
      let element;
      if (data.element) {
        // Find element by position or other criteria
        element = svg.querySelector(`[cx="${data.element.x}"]`) ||
                 svg.querySelector(`[x="${data.element.x - 40}"]`);
      } else {
        // Create invisible hit area for tooltip
        element = this.addRect(svg, data.x - 20, data.y - 20, 40, 40, {
          fill: 'transparent',
          cursor: 'help'
        });
      }

      if (element) {
        element.addEventListener('mouseenter', (e) => {
          tooltip.textContent = data.text;
          tooltip.style.opacity = '1';
        });

        element.addEventListener('mousemove', (e) => {
          const rect = svg.getBoundingClientRect();
          tooltip.style.left = `${e.clientX + 10}px`;
          tooltip.style.top = `${e.clientY - 10}px`;
        });

        element.addEventListener('mouseleave', () => {
          tooltip.style.opacity = '0';
        });
      }
    });
  }

  /**
   * Make diagrams responsive
   */
  makeResponsive() {
    this.diagrams.forEach(svg => {
      const container = svg.parentElement;
      const resizeObserver = new ResizeObserver(entries => {
        for (const entry of entries) {
          const { width } = entry.contentRect;
          const viewBox = svg.getAttribute('viewBox').split(' ');
          const aspectRatio = parseFloat(viewBox[2]) / parseFloat(viewBox[3]);
          svg.style.height = `${width / aspectRatio}px`;
        }
      });

      resizeObserver.observe(container);
    });
  }

  /**
   * Update diagrams based on theme changes
   */
  updateTheme(isDark = true) {
    this.diagrams.forEach(svg => {
      if (isDark) {
        svg.style.background = this.colors.base;
      } else {
        // Light theme adjustments would go here
        svg.style.background = '#ffffff';
      }
    });
  }
}

// Initialize diagrams when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  const diagrams = new ArchitectureDiagrams();
  diagrams.init();
  diagrams.makeResponsive();

  // Export for external use
  window.ArchitectureDiagrams = diagrams;
});

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
  module.exports = ArchitectureDiagrams;
}
