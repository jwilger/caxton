import { tool, type Plugin } from "@opencode-ai/plugin";
import { blocksForgejoInlineReply, commandText, forgejoInlineReplyPayload, recordForgejoFeedback } from "./lib/shared.ts";

function sessionID(context: unknown): string {
  const record = context as Record<string, unknown>;
  return typeof record.sessionID === "string" ? record.sessionID : "global";
}

export const PhoenixCommandedForgejoPlugin: Plugin = async () => ({
  tool: {
    forgejo_inline_reply_payload: tool({
      description: "Build the Forgejo inline review reply payload using comment.position as new_position.",
      args: {
        body: tool.schema.string().describe("Reply body"),
        path: tool.schema.string().describe("Original inline comment path"),
        position: tool.schema.number().int().nonnegative().describe("Original inline comment position field"),
      },
      async execute(args) {
        return JSON.stringify(forgejoInlineReplyPayload(args), null, 2);
      },
    }),
    forgejo_feedback_status: tool({
      description: "Record or summarize unresolved Forgejo feedback status for compaction context.",
      args: { summary: tool.schema.string().describe("Feedback status summary") },
      async execute(args, context) {
        recordForgejoFeedback(sessionID(context), args.summary);
        return `Forgejo feedback status recorded: ${args.summary}`;
      },
    }),
    forgejo_review_api_recipe: tool({
      description: "Return the Forgejo API recipe for listing reviews/comments and replying inline.",
      args: {
        owner: tool.schema.string(),
        repo: tool.schema.string(),
        pr: tool.schema.number().int().positive(),
      },
      async execute(args) {
        return [
          `GET /api/v1/repos/${args.owner}/${args.repo}/pulls/${args.pr}/reviews`,
          `GET /api/v1/repos/${args.owner}/${args.repo}/pulls/${args.pr}/reviews/{review_id}/comments`,
          `POST /api/v1/repos/${args.owner}/${args.repo}/pulls/${args.pr}/reviews/{review_id}/comments`,
          "Payload: { body, path: comment.path, new_position: comment.position, old_position: 0 }",
        ].join("\n");
      },
    }),
  },
  "tool.execute.before": async (input, output) => {
    if (/(^|\.)bash$/i.test(input.tool) && blocksForgejoInlineReply(commandText(output.args))) {
      throw new Error("Forgejo review gate: inline feedback replies must use the existing review comment thread before any top-level PR comment.");
    }
  },
});

export default PhoenixCommandedForgejoPlugin;
