export type RgrStage = "red" | "green" | "refactor";

export type RgrCycle = {
  behavior: string;
  test: string;
  command?: string;
  failingOutput?: string;
  stage: RgrStage;
};

const cycles = new Map<string, RgrCycle>();
const touchedFiles = new Map<string, Set<string>>();
const verification = new Map<string, string>();
const forgejoFeedback = new Map<string, string[]>();

export function normalizePath(path: string): string {
  return path.replaceAll("\\", "/");
}

export function isProductionElixirPath(path: string): boolean {
  const normalized = normalizePath(path);
  return /(^|\/)lib\/.*\.(ex|heex)$/.test(normalized) || /(^|\/)priv\/repo\/migrations\/.*\.exs$/.test(normalized);
}

export function isLikelyTestPath(path: string): boolean {
  const normalized = normalizePath(path);
  return /(^|\/)test\/.*_test\.exs$/.test(normalized) || /(^|\/)test\/support\//.test(normalized) || /(^|\/)test\/test_helper\.exs$/.test(normalized);
}

export function isNonBehavioralPath(path: string): boolean {
  const normalized = normalizePath(path);
  return /(^|\/)(docs|deploy)\//.test(normalized) || /(^|\/)README\.md$/.test(normalized) || /(^|\/)CHANGELOG\.md$/.test(normalized) || /\.md$/.test(normalized);
}

export function commandText(args: unknown): string {
  if (!args || typeof args !== "object") return "";
  const record = args as Record<string, unknown>;
  const command = record.command ?? record.cmd ?? record.script;
  return typeof command === "string" ? command : "";
}

export function setCommandText(args: unknown, command: string): void {
  if (!args || typeof args !== "object") return;
  const record = args as Record<string, unknown>;
  if (typeof record.command === "string") record.command = command;
  else if (typeof record.cmd === "string") record.cmd = command;
  else if (typeof record.script === "string") record.script = command;
}

export function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

export function shouldRunInAmbientShell(command: string): boolean {
  const trimmed = command.trim();
  return (
    trimmed.includes("opencode:ambient-shell") ||
    /^nix(\s|$)/.test(trimmed) ||
    /^opencode(\s|$)/.test(trimmed) ||
    /^command\s+-v\s+nix(\s|$)/.test(trimmed) ||
    /^which\s+nix(\s|$)/.test(trimmed)
  );
}

export function wrapForNixDevelop(command: string): string {
  return `nix develop --command bash -lc ${shellQuote(command)}`;
}

export function blocksForgejoInlineReply(command: string): boolean {
  return /\bgh\s+pr\s+comment\b/.test(command) || /\btea\s+comment\s+\d+\b/.test(command) || /\/pulls\/\d+\/comments\b/.test(command);
}

export function blocksUnsafeToolchainCommand(command: string): boolean {
  const checks = [
    /(^|\s)gh\s+/,
    /(^|\s)mix\s+archive\.install\b/,
    /(^|\s)mix\s+local\.(hex|rebar)\b/,
    /(^|\s)npm\s+(install|i)\b[^\n]*(\s-g|\s--global)(\s|$)/,
    /(^|\s)yarn\s+global\s+add\b/,
    /(^|\s)pnpm\s+(add|install)\b[^\n]*(\s-g|\s--global)(\s|$)/,
    /(^|\s)(asdf|mise|rtx)\s+global\b/,
    /(^|\s)(mise|rtx)\s+use\s+(-g|--global)\b/,
    /(^|\s)git\s+add\s+(-A|-u|\.|--all|--update)(\s|$)/,
    /(^|\s)git\s+commit\s+[^\n]*(\s-a\w*|\s--all)(\s|$)/,
    /--no-verify\b/,
    /--no-gpg-sign\b/,
    /(^|\s)git\s+reset\s+--hard\b/,
    /(^|\s)git\s+checkout\s+--\b/,
    /(^|\s)git\s+push\s+[^\n]*(--force|--force-with-lease)\b/,
  ];
  return checks.some((check) => check.test(command));
}

export function forgejoInlineReplyPayload(comment: { body: string; path: string; position: number }) {
  return {
    body: comment.body,
    path: comment.path,
    new_position: comment.position,
    old_position: 0,
  };
}

export function setCycle(sessionID: string, cycle: RgrCycle): void {
  cycles.set(sessionID, cycle);
}

export function getCycle(sessionID: string): RgrCycle | undefined {
  return cycles.get(sessionID);
}

export function clearCycle(sessionID: string): void {
  cycles.delete(sessionID);
}

export function recordTouchedFile(sessionID: string, path: string): void {
  const files = touchedFiles.get(sessionID) ?? new Set<string>();
  files.add(path);
  touchedFiles.set(sessionID, files);
}

export function recordVerification(sessionID: string, status: string): void {
  verification.set(sessionID, status);
}

export function recordForgejoFeedback(sessionID: string, summary: string): void {
  const items = forgejoFeedback.get(sessionID) ?? [];
  items.push(summary);
  forgejoFeedback.set(sessionID, items);
}

export function sessionContext(sessionID: string): string[] {
  const context: string[] = [];
  const cycle = cycles.get(sessionID);
  if (cycle) context.push(`Active RGR cycle: ${JSON.stringify(cycle)}`);
  const files = touchedFiles.get(sessionID);
  if (files?.size) context.push(`Touched files: ${Array.from(files).sort().join(", ")}`);
  const verify = verification.get(sessionID);
  if (verify) context.push(`Verification status: ${verify}`);
  const feedback = forgejoFeedback.get(sessionID);
  if (feedback?.length) context.push(`Unresolved Forgejo feedback: ${feedback.join("; ")}`);
  return context;
}
