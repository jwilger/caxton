import { tool, type Plugin } from "@opencode-ai/plugin";
import {
  clearCycle,
  getCycle,
  isLikelyTestPath,
  isNonBehavioralPath,
  isProductionElixirPath,
  recordTouchedFile,
  recordVerification,
  sessionContext,
  setCycle,
} from "./lib/shared.ts";

function sessionID(input: unknown): string {
  const record = input as Record<string, unknown>;
  return typeof record.sessionID === "string" ? record.sessionID : "global";
}

function filePathFromArgs(args: unknown): string | undefined {
  if (!args || typeof args !== "object") return undefined;
  const record = args as Record<string, unknown>;
  const path = record.filePath ?? record.file_path ?? record.path;
  return typeof path === "string" ? path : undefined;
}

function isEditTool(toolID: string): boolean {
  return /(^|\.)(edit|write|apply_patch)$/i.test(toolID) || /apply_patch/i.test(toolID);
}

function rejectsWaterfallTodo(args: unknown): boolean {
  const text = JSON.stringify(args ?? "").toLowerCase();
  const componentWords = [
    "schema",
    "migration",
    "controller",
    "liveview",
    "component",
    "context",
    "command",
    "event",
    "aggregate",
    "projector",
    "process manager",
    "handler",
    "then add tests",
  ];
  const hasComponents = componentWords.filter((word) => text.includes(word)).length >= 2;
  return hasComponents && !text.includes("red") && !text.includes("failing test") && !text.includes("rgr");
}

export const PhoenixCommandedDisciplinePlugin: Plugin = async () => ({
  tool: {
    rgr_start: tool({
      description: "Start a RED-GREEN-REFACTOR cycle for one behavior.",
      args: {
        behavior: tool.schema.string().describe("Observable behavior under test"),
        test: tool.schema.string().describe("Specific failing test name or path"),
      },
      async execute(args, context) {
        setCycle(sessionID(context), { behavior: args.behavior, test: args.test, stage: "red" });
        return `RGR cycle started for ${args.behavior}. Record observed RED output before production edits.`;
      },
    }),
    rgr_record_red: tool({
      description: "Record observed failing test output for the active RGR cycle.",
      args: {
        command: tool.schema.string().describe("Focused test command that failed"),
        output: tool.schema.string().min(1).describe("Copied failing output from the actual run"),
      },
      async execute(args, context) {
        const id = sessionID(context);
        const current = getCycle(id);
        if (!current) throw new Error("Start an RGR cycle before recording RED.");
        setCycle(id, { ...current, command: args.command, failingOutput: args.output, stage: "red" });
        return "RED recorded. Minimum production edits are now allowed for this cycle.";
      },
    }),
    rgr_mark_green: tool({
      description: "Mark the active RGR cycle green after the focused test passes.",
      args: { output: tool.schema.string().describe("Passing test output or concise verification summary") },
      async execute(args, context) {
        const id = sessionID(context);
        const current = getCycle(id);
        if (!current?.failingOutput) throw new Error("Cannot mark GREEN before observed RED is recorded.");
        setCycle(id, { ...current, stage: "green" });
        recordVerification(id, args.output);
        return "GREEN recorded. Refactoring is allowed with tests green.";
      },
    }),
    rgr_mark_refactor: tool({
      description: "Mark refactor completion and clear the active RGR cycle.",
      args: { verification: tool.schema.string().describe("Verification run after refactor") },
      async execute(args, context) {
        const id = sessionID(context);
        recordVerification(id, args.verification);
        clearCycle(id);
        return "REFACTOR recorded. RGR cycle complete.";
      },
    }),
    rgr_status: tool({
      description: "Inspect active RGR and verification context.",
      args: {},
      async execute(_args, context) {
        const items = sessionContext(sessionID(context));
        return items.length ? items.join("\n") : "No active RGR cycle recorded for this session.";
      },
    }),
  },
  "tool.execute.before": async (input, output) => {
    const id = sessionID(input);

    if (isEditTool(input.tool)) {
      const path = filePathFromArgs(output.args);
      if (path) recordTouchedFile(id, path);
      if (path && isProductionElixirPath(path) && !isLikelyTestPath(path) && !isNonBehavioralPath(path)) {
        const current = getCycle(id);
        if (!current?.failingOutput) {
          throw new Error("RGR gate: production Elixir/Phoenix/Commanded edits under lib/** or priv/repo/migrations require observed RED output recorded with rgr_record_red.");
        }
      }
    }

    if (/todo(write|update)?$/i.test(input.tool) && rejectsWaterfallTodo(output.args)) {
      throw new Error("RGR plan gate: behavior work todo lists must name failing tests, not component-waterfall tasks.");
    }
  },
  "experimental.session.compacting": async (input, output) => {
    output.context.push(...sessionContext(sessionID(input)));
  },
});

export default PhoenixCommandedDisciplinePlugin;
