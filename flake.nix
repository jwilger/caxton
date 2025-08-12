{
  description = "Caxton development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            nodejs
            python3
            uv
            git
            gh
            pre-commit
            cargo-nextest
            cargo-watch
            cargo-expand
            cargo-edit
            just
            pkg-config
            openssl
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          # Configure development environment
          shellHook = ''
            echo "🦀 Caxton Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "Available tools: cargo-nextest, cargo-watch, cargo-expand, cargo-edit"
            echo ""

            # Set up git safety wrapper
            chmod +x "$PWD/scripts/git" 2>/dev/null || true
            export PATH="$PWD/scripts:$PATH"

            echo "🛡️  Git safety wrapper enabled"
            echo "   - git commands now go through quality enforcement"
            echo "   - --no-verify is blocked (use /usr/bin/git for emergencies)"
            echo ""

            # Ensure claude code is available
            # First check if claude code is installed
            if ! command -v claude &> /dev/null; then
              npx @anthropic-ai/claude-code install --force latest
            fi

            # MCP Memory Server configured via 'claude mcp' command
            echo "🧠 MCP Memory Server configured via Claude Code"
            echo "   - Check status: claude mcp list"
            echo "   - Memory file: .claude/sparc-memory.jsonl"
            echo ""

            echo "📋 Common commands:"
            echo "  cargo nextest run    # Run tests with nextest"
            echo "  cargo watch -x test  # Auto-run tests on changes"
            echo "  cargo clippy         # Lint code"
            echo "  cargo fmt           # Format code"
            echo ""
          '';
        };
      }
    );
}
