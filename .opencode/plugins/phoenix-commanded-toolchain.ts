import type { Plugin } from "@opencode-ai/plugin";
import {
  blocksUnsafeToolchainCommand,
  commandText,
  setCommandText,
  shouldRunInAmbientShell,
  wrapForNixDevelop,
} from "./lib/shared.ts";

export const PhoenixCommandedToolchainPlugin: Plugin = async ({ worktree }) => ({
  "shell.env": async (_input, output) => {
    const depsRoot = `${worktree}/.dependencies`;
    output.env.CAXTON_DEPS_ROOT = depsRoot;
    output.env.MIX_HOME = `${depsRoot}/elixir/mix`;
    output.env.HEX_HOME = `${depsRoot}/elixir/hex`;
    output.env.REBAR_CACHE_DIR = `${depsRoot}/elixir/rebar`;
    output.env.NPM_CONFIG_PREFIX = `${depsRoot}/nodejs/global`;
    output.env.NPM_CONFIG_CACHE = `${depsRoot}/nodejs/cache`;
    output.env.PNPM_HOME = `${depsRoot}/nodejs/pnpm`;
    output.env.YARN_CACHE_FOLDER = `${depsRoot}/nodejs/yarn/cache`;
    output.env.COREPACK_HOME = `${depsRoot}/nodejs/corepack`;
  },
  "tool.execute.before": async (input, output) => {
    if (!/(^|\.)bash$/i.test(input.tool)) return;

    const command = commandText(output.args);
    if (!command) return;

    if (blocksUnsafeToolchainCommand(command)) {
      throw new Error("Caxton toolchain gate blocked a command that bypasses project-local tooling, Forgejo workflow, scope hygiene, hooks, signing, or git safety.");
    }

    if (!shouldRunInAmbientShell(command)) {
      setCommandText(output.args, wrapForNixDevelop(command));
    }
  },
});

export default PhoenixCommandedToolchainPlugin;
