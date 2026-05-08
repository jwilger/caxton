import type { Plugin } from "@opencode-ai/plugin";
import { sessionContext } from "./lib/shared.ts";

function sessionID(input: unknown): string {
  const record = input as Record<string, unknown>;
  return typeof record.sessionID === "string" ? record.sessionID : "global";
}

export const PhoenixCommandedContextPlugin: Plugin = async () => ({
  "experimental.session.compacting": async (input, output) => {
    const context = sessionContext(sessionID(input));
    if (context.length) {
      output.context.push("Phoenix/Commanded project context:", ...context);
    }
  },
  "tool.execute.after": async (input, output) => {
    if (/rgr_|forgejo_/i.test(input.tool)) {
      output.metadata = { ...(output.metadata ?? {}), phoenixCommandedContextPreserved: true };
    }
  },
});

export default PhoenixCommandedContextPlugin;
