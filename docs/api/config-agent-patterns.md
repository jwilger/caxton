---
title: "Configuration Agent API Patterns & Examples"
date: 2025-09-10
layout: page
categories: [api, patterns, examples]
---

## Overview

This document provides comprehensive patterns and examples for using
configuration-driven agents APIs effectively. These patterns demonstrate
real-world workflows combining multiple API endpoints to achieve common
development and operational tasks.

**Architecture Context**: Configuration agents represent the primary user
experience for Caxton (ADR-0028), offering 5-10 minute onboarding through
markdown + YAML definitions versus 2-4 hour WASM compilation workflows.

## Development Patterns

### Pattern 1: Template-Based Agent Development

**Scenario**: Create a new data analysis agent using templates to accelerate development.

#### Step 1: Discover Available Templates

```javascript
// Find templates suitable for data analysis
const templates = await fetch(
  "/api/v1/templates?category=data-processing&complexity=simple",
);
const { templates: availableTemplates } = await templates.json();

console.log(
  "Available templates:",
  availableTemplates.map((t) => ({
    id: t.id,
    name: t.name,
    setup_time: t.estimated_setup_time,
  })),
);
```

#### Step 2: Get Template Details

```javascript
// Get detailed template information
const templateDetails = await fetch("/api/v1/templates/data-analyzer-basic");
const template = await templateDetails.json();

console.log("Template parameters:", template.parameters);
console.log("Required tools:", template.dependencies.required_tools);
```

#### Step 3: Generate Configuration

```javascript
// Generate configuration with custom parameters
const generated = await fetch(
  "/api/v1/templates/data-analyzer-basic/generate",
  {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      parameters: {
        AGENT_NAME: "SalesAnalyzer",
        MAX_FILE_SIZE: "25MB",
        MEMORY_ENABLED: true,
        SPECIALIZATION: "sales performance analysis",
      },
      workspace: "sales-team",
      validate: true,
    }),
  },
);

const config = await generated.json();
if (!config.ready_to_deploy) {
  throw new Error("Generated configuration failed validation");
}
```

#### Step 4: Deploy Agent

```javascript
// Deploy the generated configuration
const deployment = await fetch("/api/v1/config-agents", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    name: "SalesAnalyzer",
    content: config.generated_content,
    workspace: "sales-team",
    auto_start: true,
  }),
});

const agent = await deployment.json();
console.log(`Agent ${agent.name} deployed with ID: ${agent.id}`);
```

**Complete Pattern Function**:

```javascript
async function createAgentFromTemplate(
  templateId,
  parameters,
  workspace = "default",
) {
  try {
    // Generate configuration
    const configResponse = await fetch(
      `/api/v1/templates/${templateId}/generate`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          parameters,
          workspace,
          validate: true,
        }),
      },
    );

    const config = await configResponse.json();
    if (!config.ready_to_deploy) {
      throw new Error(
        `Configuration validation failed: ${JSON.stringify(config.validation_result.errors)}`,
      );
    }

    // Deploy agent
    const deployResponse = await fetch("/api/v1/config-agents", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: parameters.AGENT_NAME,
        content: config.generated_content,
        workspace,
        auto_start: true,
      }),
    });

    const agent = await deployResponse.json();

    // Wait for agent to be running
    await waitForAgentStatus(agent.id, "running", 30000);

    return agent;
  } catch (error) {
    console.error("Failed to create agent from template:", error);
    throw error;
  }
}

// Usage
const salesAgent = await createAgentFromTemplate("data-analyzer-basic", {
  AGENT_NAME: "SalesAnalyzer",
  SPECIALIZATION: "sales analysis",
  MEMORY_ENABLED: true,
});
```

### Pattern 2: Configuration Validation & Testing

**Scenario**: Validate and test a custom agent configuration before deployment.

#### Step 1: Comprehensive Validation

```javascript
async function validateConfiguration(content, workspace = "development") {
  const validation = await fetch("/api/v1/validate/config", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      content,
      workspace,
      strict_mode: true,
      check_tool_availability: true,
    }),
  });

  const result = await validation.json();

  if (!result.valid) {
    console.error("Validation errors:", result.errors);
    console.warn("Validation warnings:", result.warnings);
    return false;
  }

  console.log("Configuration valid!");
  console.log("Estimated resources:", result.estimated_resources);

  if (result.warnings.length > 0) {
    console.warn("Warnings to consider:", result.warnings);
  }

  return result;
}
```

#### Step 2: Test Configuration Behavior

```javascript
async function testConfiguration(content, testScenarios) {
  const test = await fetch("/api/v1/test/config", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      content,
      test_scenarios: testScenarios,
      workspace: "testing",
    }),
  });

  const result = await test.json();

  console.log(`Test result: ${result.overall_result}`);
  console.log("Performance metrics:", result.performance_metrics);

  result.scenarios.forEach((scenario) => {
    console.log(`Scenario "${scenario.name}": ${scenario.status}`);
    if (scenario.status === "passed") {
      console.log(
        `  - Capabilities: ${scenario.capabilities_triggered.join(", ")}`,
      );
      console.log(`  - Tools used: ${scenario.tools_used.join(", ")}`);
      console.log(`  - Response time: ${scenario.execution_time_ms}ms`);
    } else {
      console.error(`  - Error: ${scenario.error}`);
    }
  });

  return result.overall_result === "passed";
}
```

#### Complete Validation & Testing Pattern

```javascript
async function validateAndTestConfiguration(content) {
  // Define test scenarios based on agent capabilities
  const testScenarios = [
    {
      name: "basic_analysis_request",
      input: "Analyze the sales data from Q3 2024",
      expected_capabilities: ["data-analysis"],
      expected_tools: ["http_client", "csv_parser"],
    },
    {
      name: "chart_generation",
      input: "Create a bar chart showing monthly revenue",
      expected_capabilities: ["report-generation"],
      expected_tools: ["chart_generator"],
    },
    {
      name: "memory_recall",
      input: "What insights do you remember about Q2 performance?",
      expected_capabilities: ["data-analysis"],
      expected_tools: [], // Memory system integration
    },
  ];

  try {
    // Validate configuration
    console.log("Validating configuration...");
    const validation = await validateConfiguration(content);
    if (!validation) {
      return false;
    }

    // Test behavior
    console.log("Testing configuration behavior...");
    const testsPassed = await testConfiguration(content, testScenarios);
    if (!testsPassed) {
      console.error("Configuration tests failed");
      return false;
    }

    console.log("✅ Configuration is ready for deployment!");
    return true;
  } catch (error) {
    console.error("Validation/testing failed:", error);
    return false;
  }
}
```

### Pattern 3: Hot-Reload Development Workflow

**Scenario**: Iterative development with hot-reload for rapid configuration changes.

```javascript
class ConfigAgentDeveloper {
  constructor(agentId, initialConfig) {
    this.agentId = agentId;
    this.currentConfig = initialConfig;
    this.backups = [];
  }

  async hotReload(newConfig, options = {}) {
    const { validateFirst = true, createBackup = true } = options;

    try {
      // Optional validation before reload
      if (validateFirst) {
        const validation = await validateConfiguration(newConfig);
        if (!validation) {
          throw new Error("Configuration validation failed");
        }
      }

      // Perform hot-reload
      const reload = await fetch(
        `/api/v1/dev/config-agents/${this.agentId}/hot-reload`,
        {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            content: newConfig,
            validate_first: validateFirst,
            backup_current: createBackup,
          }),
        },
      );

      const result = await reload.json();

      if (result.status === "success") {
        console.log(`✅ Hot-reloaded in ${result.reload_time_ms}ms`);
        console.log("Changes:", result.changes_detected);

        // Track backup for rollback
        if (result.backup_created) {
          this.backups.push({
            id: result.backup_created,
            version: result.previous_version,
            timestamp: new Date(),
          });
        }

        this.currentConfig = newConfig;
        return true;
      } else {
        throw new Error(`Hot-reload failed: ${result.error}`);
      }
    } catch (error) {
      console.error("Hot-reload failed:", error);
      return false;
    }
  }

  async compareWithCurrent(newConfig) {
    const comparison = await fetch("/api/v1/dev/config/compare", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        original_content: this.currentConfig,
        updated_content: newConfig,
        comparison_type: "semantic",
      }),
    });

    const result = await comparison.json();

    console.log(`Found ${result.differences_found} differences:`);
    result.changes.forEach((change) => {
      console.log(
        `  - ${change.type}: ${change.field} (impact: ${change.impact})`,
      );
    });

    return result;
  }

  async rollbackToBackup(backupIndex = 0) {
    if (this.backups.length === 0) {
      throw new Error("No backups available for rollback");
    }

    const backup = this.backups[backupIndex];
    console.log(`Rolling back to backup from ${backup.timestamp}`);

    // Note: This would require additional API endpoint for backup retrieval
    // For now, this demonstrates the pattern
    console.log(`Would rollback to backup: ${backup.id}`);
  }
}

// Usage example
const developer = new ConfigAgentDeveloper("config-12345", originalConfig);

// Compare changes
await developer.compareWithCurrent(modifiedConfig);

// Hot-reload with validation
await developer.hotReload(modifiedConfig, {
  validateFirst: true,
  createBackup: true,
});
```

## Operational Patterns

### Pattern 4: Multi-Agent Capability Orchestration

**Scenario**: Deploy multiple agents with complementary capabilities and
coordinate their interactions.

#### Step 1: Deploy Agent Ensemble

```javascript
async function deployAgentEnsemble(agentConfigs, workspace) {
  const deployedAgents = [];

  for (const config of agentConfigs) {
    try {
      // Validate each configuration
      const validation = await validateConfiguration(config.content, workspace);
      if (!validation) {
        throw new Error(`Validation failed for ${config.name}`);
      }

      // Deploy agent
      const deployment = await fetch("/api/v1/config-agents", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: config.name,
          content: config.content,
          workspace,
          auto_start: true,
        }),
      });

      const agent = await deployment.json();
      deployedAgents.push(agent);

      console.log(
        `✅ Deployed ${agent.name} with capabilities: ${agent.capabilities.join(", ")}`,
      );
    } catch (error) {
      console.error(`❌ Failed to deploy ${config.name}:`, error);
      // Continue deploying other agents
    }
  }

  // Wait for all agents to be running
  await Promise.all(
    deployedAgents.map((agent) =>
      waitForAgentStatus(agent.id, "running", 30000),
    ),
  );

  return deployedAgents;
}
```

#### Step 2: Monitor Capability Coverage

```javascript
async function monitorCapabilityCoverage(workspace) {
  const capabilities = await fetch(
    `/api/v1/capabilities?workspace=${workspace}`,
  );
  const { capabilities: capabilityList } = await capabilities.json();

  console.log("Capability coverage:");
  capabilityList.forEach((cap) => {
    console.log(`  ${cap.capability}: ${cap.providers.length} providers`);

    const healthyProviders = cap.providers.filter((p) => p.status === "active");
    if (healthyProviders.length === 0) {
      console.warn(`    ⚠️  No healthy providers for ${cap.capability}`);
    } else if (healthyProviders.length === 1) {
      console.warn(`    ⚠️  Single point of failure for ${cap.capability}`);
    } else {
      console.log(`    ✅ ${healthyProviders.length} healthy providers`);
    }
  });

  return capabilityList;
}
```

#### Step 3: Coordinate Agent Communication

```javascript
async function orchestrateWorkflow(workflowSteps, workspace) {
  const results = [];

  for (const step of workflowSteps) {
    try {
      // Find agent with required capability
      const capabilityResponse = await fetch(
        `/api/v1/capabilities/${step.capability}?routing_strategy=least_loaded`,
      );
      const { providers } = await capabilityResponse.json();

      if (providers.length === 0) {
        throw new Error(
          `No providers found for capability: ${step.capability}`,
        );
      }

      const bestProvider = providers[0];
      console.log(
        `Routing step "${step.name}" to agent: ${bestProvider.agent_name}`,
      );

      // Send agent message to capability
      const message = await fetch("/api/v1/messages", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          to_capability: step.capability,
          performative: "REQUEST",
          content: step.request,
          conversation_context: results.map((r) => r.result), // Pass previous results
        }),
      });

      const result = await message.json();
      results.push({
        step: step.name,
        provider: bestProvider.agent_name,
        result: result.content,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      console.error(`❌ Step "${step.name}" failed:`, error);
      results.push({
        step: step.name,
        error: error.message,
        timestamp: new Date().toISOString(),
      });
    }
  }

  return results;
}

// Example workflow
const workflowSteps = [
  {
    name: "data_collection",
    capability: "data-analysis",
    request: "Collect and analyze Q3 sales data from all regions",
  },
  {
    name: "trend_analysis",
    capability: "data-analysis",
    request: "Identify trends and patterns in the collected data",
  },
  {
    name: "report_generation",
    capability: "report-generation",
    request: "Generate executive summary report with visualizations",
  },
];

const workflowResults = await orchestrateWorkflow(
  workflowSteps,
  "analytics-workspace",
);
```

### Pattern 5: Memory-Enabled Agent Collaboration

**Scenario**: Agents that learn from each other and share knowledge
through the memory system.

#### Step 1: Deploy Memory-Enabled Agents

```javascript
async function deployLearningAgents(agentConfigs, sharedWorkspace) {
  const agents = [];

  for (const config of agentConfigs) {
    // Ensure memory is enabled for learning
    const memoryEnabledConfig = {
      ...config,
      content: config.content
        .replace(/memory_enabled: false/g, "memory_enabled: true")
        .replace(
          /memory_scope: "agent-only"/g,
          'memory_scope: "workspace"', // Enable shared learning
        ),
    };

    const agent = await createAgentFromTemplate(
      config.template_id,
      memoryEnabledConfig.parameters,
      sharedWorkspace,
    );

    agents.push(agent);
  }

  return agents;
}
```

#### Step 2: Knowledge Sharing Pattern

```javascript
async function shareKnowledge(
  fromAgentId,
  toCapability,
  knowledgeType,
  workspace,
) {
  try {
    // Agent stores its findings in memory
    await fetch("/api/v1/memory/entities", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        agent_id: fromAgentId,
        entity_name: `Knowledge_${knowledgeType}_${Date.now()}`,
        entity_type: "shared_insight",
        observations: [
          "This insight is shared with the team",
          `Type: ${knowledgeType}`,
          `Available for: ${toCapability}`,
        ],
        memory_scope: "workspace",
        metadata: {
          knowledge_type: knowledgeType,
          target_capability: toCapability,
          shared_at: new Date().toISOString(),
        },
      }),
    });

    // Notify other agents about new knowledge
    await fetch("/api/v1/messages", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        to_capability: toCapability,
        performative: "INFORM",
        content: {
          type: "knowledge_update",
          knowledge_type: knowledgeType,
          message: "New shared insight available in workspace memory",
        },
      }),
    });

    console.log(
      `✅ Knowledge shared from agent ${fromAgentId} to ${toCapability} capability`,
    );
  } catch (error) {
    console.error("Knowledge sharing failed:", error);
  }
}
```

#### Step 3: Collaborative Learning Pattern

```javascript
async function facilitateCollaborativeLearning(workspace, learningTopics) {
  for (const topic of learningTopics) {
    try {
      // Search for existing knowledge on the topic
      const searchResponse = await fetch(
        `/api/v1/memory/search?query=${encodeURIComponent(topic)}` +
          `&memory_scope=workspace&limit=10`,
      );
      const { results } = await searchResponse.json();

      if (results.length === 0) {
        console.log(`No existing knowledge found for topic: ${topic}`);
        continue;
      }

      // Create relationships between related knowledge
      for (let i = 0; i < results.length - 1; i++) {
        await fetch("/api/v1/memory/relations", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            agent_id: results[i].agent_id,
            from_entity: results[i].entity_name,
            to_entity: results[i + 1].entity_name,
            relation_type: "relates_to",
            strength: 0.7,
            confidence: 0.8,
            memory_scope: "workspace",
            metadata: {
              topic: topic,
              relationship_basis: "collaborative_learning",
            },
          }),
        });
      }

      console.log(`✅ Created knowledge relationships for topic: ${topic}`);
    } catch (error) {
      console.error(`Learning facilitation failed for ${topic}:`, error);
    }
  }
}
```

### Pattern 6: Production Monitoring & Health Management

**Scenario**: Monitor agent health, capability availability, and system
performance in production.

```javascript
class ProductionMonitor {
  constructor(workspace) {
    this.workspace = workspace;
    this.healthCheckInterval = null;
    this.metrics = {
      agent_health: new Map(),
      capability_coverage: new Map(),
      performance_stats: [],
    };
  }

  async startMonitoring(intervalMs = 30000) {
    console.log(
      `Starting production monitoring for workspace: ${this.workspace}`,
    );

    // Initial health check
    await this.checkSystemHealth();

    // Schedule regular health checks
    this.healthCheckInterval = setInterval(() => {
      this.checkSystemHealth();
    }, intervalMs);
  }

  async checkSystemHealth() {
    try {
      // Check agent health
      await this.checkAgentHealth();

      // Check capability coverage
      await this.checkCapabilityCoverage();

      // Check memory system health
      await this.checkMemoryHealth();

      // Log overall system status
      this.logSystemStatus();
    } catch (error) {
      console.error("Health check failed:", error);
    }
  }

  async checkAgentHealth() {
    const agents = await fetch(
      `/api/v1/config-agents?workspace=${this.workspace}`,
    );
    const { agents: agentList } = await agents.json();

    for (const agent of agentList) {
      const previousHealth = this.metrics.agent_health.get(agent.id);
      const currentHealth = {
        status: agent.status,
        timestamp: new Date(),
        last_activity: new Date(agent.last_activity),
      };

      // Detect status changes
      if (previousHealth && previousHealth.status !== currentHealth.status) {
        console.log(
          `🔄 Agent ${agent.name} status changed: ${previousHealth.status} → ${currentHealth.status}`,
        );

        if (currentHealth.status === "error") {
          await this.handleAgentError(agent);
        }
      }

      this.metrics.agent_health.set(agent.id, currentHealth);
    }
  }

  async checkCapabilityCoverage() {
    const capabilities = await fetch(`/api/v1/capabilities/health`);
    const health = await capabilities.json();

    // Track capability health over time
    this.metrics.capability_coverage.set(new Date(), {
      overall_health: health.overall_health,
      healthy_capabilities: health.healthy_capabilities,
      degraded_capabilities: health.degraded_capabilities,
      unhealthy_capabilities: health.unhealthy_capabilities,
    });

    // Alert on degraded capabilities
    if (health.degraded_capabilities > 0 || health.unhealthy_capabilities > 0) {
      console.warn(
        `⚠️  Capability health degraded: ${health.degraded_capabilities} degraded, ${health.unhealthy_capabilities} unhealthy`,
      );

      for (const cap of health.capabilities) {
        if (cap.health !== "healthy") {
          await this.handleCapabilityIssue(cap);
        }
      }
    }
  }

  async checkMemoryHealth() {
    const memoryHealth = await fetch("/api/v1/memory/health");
    const health = await memoryHealth.json();

    if (health.status !== "healthy") {
      console.warn(`⚠️  Memory system health: ${health.status}`);

      for (const check of health.checks) {
        if (check.status !== "pass") {
          console.warn(`  - ${check.check}: ${check.status}`);
        }
      }
    }
  }

  async handleAgentError(agent) {
    console.error(`❌ Agent ${agent.name} encountered an error`);

    // Attempt automatic recovery
    try {
      await fetch(`/api/v1/config-agents/${agent.id}/restart`, {
        method: "POST",
      });

      console.log(`🔄 Attempted automatic restart for agent ${agent.name}`);
    } catch (error) {
      console.error(
        `❌ Automatic restart failed for agent ${agent.name}:`,
        error,
      );
      // Could trigger alerts to operations team here
    }
  }

  async handleCapabilityIssue(capability) {
    console.warn(`🔍 Investigating capability issue: ${capability.capability}`);

    // Find healthy alternative providers
    const alternativeProviders = await fetch(
      `/api/v1/capabilities/${capability.capability}?include_unhealthy=false`,
    );
    const { providers } = await alternativeProviders.json();

    if (providers.length === 0) {
      console.error(
        `❌ No healthy providers available for ${capability.capability}`,
      );
      // Could trigger emergency deployment of backup agents
    } else {
      console.log(
        `✅ ${providers.length} healthy providers available for ${capability.capability}`,
      );
    }
  }

  logSystemStatus() {
    const totalAgents = this.metrics.agent_health.size;
    const healthyAgents = Array.from(this.metrics.agent_health.values()).filter(
      (h) => h.status === "running",
    ).length;

    console.log(
      `📊 System Status - Agents: ${healthyAgents}/${totalAgents} healthy`,
    );
  }

  stopMonitoring() {
    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
      this.healthCheckInterval = null;
      console.log("Production monitoring stopped");
    }
  }
}

// Usage
const monitor = new ProductionMonitor("production-workspace");
await monitor.startMonitoring(30000); // Check every 30 seconds
```

## Integration Utilities

### Pattern 7: Complete SDK Implementation

```javascript
class CaxtonConfigAgentSDK {
  constructor(baseUrl, apiKey) {
    this.baseUrl = baseUrl;
    this.apiKey = apiKey;
    this.defaultHeaders = {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    };
  }

  // Agent Management
  async createAgent(config) {
    const validation = await this.validateConfig(config.content);
    if (!validation.valid) {
      throw new Error(
        `Configuration invalid: ${JSON.stringify(validation.errors)}`,
      );
    }

    return await this.request("POST", "/api/v1/config-agents", config);
  }

  async getAgent(agentId) {
    return await this.request("GET", `/api/v1/config-agents/${agentId}`);
  }

  async updateAgent(agentId, content) {
    return await this.request("PUT", `/api/v1/config-agents/${agentId}`, {
      content,
    });
  }

  async deleteAgent(agentId, force = false) {
    return await this.request(
      "DELETE",
      `/api/v1/config-agents/${agentId}?force=${force}`,
    );
  }

  // Configuration Validation
  async validateConfig(content, options = {}) {
    return await this.request("POST", "/api/v1/validate/config", {
      content,
      ...options,
    });
  }

  async testConfig(content, scenarios) {
    return await this.request("POST", "/api/v1/test/config", {
      content,
      test_scenarios: scenarios,
    });
  }

  // Template Management
  async listTemplates(filters = {}) {
    const queryString = new URLSearchParams(filters).toString();
    return await this.request("GET", `/api/v1/templates?${queryString}`);
  }

  async generateFromTemplate(templateId, parameters, options = {}) {
    return await this.request(
      "POST",
      `/api/v1/templates/${templateId}/generate`,
      {
        parameters,
        ...options,
      },
    );
  }

  // Capability Management
  async discoverCapabilities(filters = {}) {
    const queryString = new URLSearchParams(filters).toString();
    return await this.request("GET", `/api/v1/capabilities?${queryString}`);
  }

  async findCapabilityProviders(capability, strategy = "priority") {
    return await this.request(
      "GET",
      `/api/v1/capabilities/${capability}?routing_strategy=${strategy}`,
    );
  }

  // Memory Operations
  async storeMemory(agentId, entity) {
    return await this.request("POST", "/api/v1/memory/entities", {
      agent_id: agentId,
      ...entity,
    });
  }

  async searchMemory(agentId, query, options = {}) {
    const params = new URLSearchParams({
      agent_id: agentId,
      query,
      ...options,
    });
    return await this.request("GET", `/api/v1/memory/search?${params}`);
  }

  // Messaging
  async sendMessage(toCapability, content, performative = "REQUEST") {
    return await this.request("POST", "/api/v1/messages", {
      to_capability: toCapability,
      performative,
      content,
    });
  }

  // Health Monitoring
  async getSystemHealth() {
    return await this.request("GET", "/api/v1/capabilities/health");
  }

  async getMemoryHealth() {
    return await this.request("GET", "/api/v1/memory/health");
  }

  // Development Tools
  async hotReload(agentId, content, options = {}) {
    return await this.request(
      "PUT",
      `/api/v1/dev/config-agents/${agentId}/hot-reload`,
      {
        content,
        ...options,
      },
    );
  }

  async compareConfigs(original, updated) {
    return await this.request("POST", "/api/v1/dev/config/compare", {
      original_content: original,
      updated_content: updated,
    });
  }

  // Utility Methods
  async request(method, endpoint, body = null) {
    const config = {
      method,
      headers: this.defaultHeaders,
    };

    if (body) {
      config.body = JSON.stringify(body);
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, config);

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`API Error: ${error.error} (${response.status})`);
    }

    return await response.json();
  }

  async waitForStatus(agentId, targetStatus, timeoutMs = 30000) {
    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      const agent = await this.getAgent(agentId);

      if (agent.status === targetStatus) {
        return agent;
      }

      if (agent.status === "error") {
        throw new Error(
          `Agent entered error state while waiting for ${targetStatus}`,
        );
      }

      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    throw new Error(
      `Timeout waiting for agent ${agentId} to reach status ${targetStatus}`,
    );
  }
}

// Usage
const sdk = new CaxtonConfigAgentSDK(
  "https://caxton.example.com",
  "your-api-key",
);

// Create agent from template
const templates = await sdk.listTemplates({ category: "data-processing" });
const config = await sdk.generateFromTemplate("data-analyzer-basic", {
  AGENT_NAME: "ProductionAnalyzer",
  MEMORY_ENABLED: true,
});

const agent = await sdk.createAgent({
  name: "ProductionAnalyzer",
  content: config.generated_content,
  workspace: "production",
});

await sdk.waitForStatus(agent.id, "running");
console.log("Agent ready for production!");
```

## Error Handling Patterns

### Comprehensive Error Handling

```javascript
class CaxtonError extends Error {
  constructor(message, code, details = null) {
    super(message);
    this.name = "CaxtonError";
    this.code = code;
    this.details = details;
  }
}

async function safeApiCall(apiFunction, retries = 3, backoffMs = 1000) {
  let lastError;

  for (let i = 0; i < retries; i++) {
    try {
      return await apiFunction();
    } catch (error) {
      lastError = error;

      // Don't retry validation errors
      if (error.code === "VALIDATION_ERROR") {
        throw error;
      }

      // Exponential backoff
      if (i < retries - 1) {
        await new Promise((resolve) =>
          setTimeout(resolve, backoffMs * Math.pow(2, i)),
        );
      }
    }
  }

  throw new CaxtonError(
    `API call failed after ${retries} retries: ${lastError.message}`,
    "MAX_RETRIES_EXCEEDED",
    { originalError: lastError, retries },
  );
}

// Usage
try {
  const result = await safeApiCall(() => sdk.createAgent(config), 3);
} catch (error) {
  if (error instanceof CaxtonError) {
    console.error(`Caxton Error [${error.code}]:`, error.message);
    if (error.details) {
      console.error("Details:", error.details);
    }
  } else {
    console.error("Unexpected error:", error);
  }
}
```

## Best Practices Summary

### Development Best Practices

1. **Always validate configurations** before deployment
2. **Use templates** for faster development and consistency
3. **Test configurations** with realistic scenarios
4. **Use hot-reload** for iterative development
5. **Store backups** before making changes

### Operational Best Practices

1. **Monitor agent health** continuously in production
2. **Ensure capability redundancy** for critical services
3. **Use memory scopes** appropriately (agent/workspace/global)
4. **Implement circuit breakers** for capability routing
5. **Plan for graceful degradation** when capabilities are unavailable

### Security Best Practices

1. **Validate all inputs** including configuration content
2. **Use workspace isolation** for multi-tenant scenarios
3. **Implement proper authentication** for API access
4. **Monitor memory usage** to prevent resource exhaustion
5. **Audit configuration changes** in production environments

## Related Documentation

- [Configuration Agent API](config-agents.md) - Core agent management
- [Capability Registration API](capability-registration.md) - Capability system
- [Memory System API](memory-integration.md) - Agent memory operations
- [Configuration Validation API](configuration-validation.md) - Validation and testing
- [Agent Messaging API](fipa-messaging.md) - Agent communication
- [ADR-0028](../adrs/0028-configuration-driven-agent-architecture.md) -
  Architecture foundation
