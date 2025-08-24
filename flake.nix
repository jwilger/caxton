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

        # Python environment with numpy support and Qdrant dependencies
        pythonEnv = pkgs.python3.withPackages (
          ps: with ps; [
            pip
            setuptools
            wheel
            numpy
            scipy
            grpcio
            protobuf
            httpx
            pydantic
            typing-extensions
            # Additional packages that mcp-server-qdrant might need
          ]
        );
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            nodejs
            pythonEnv # Use the Python environment with numpy
            uv
            git
            gh
            pre-commit
            cargo-nextest
            cargo-watch
            cargo-expand
            cargo-edit
            bacon
            just
            pkg-config
            openssl

            # Build dependencies for Python C extensions
            gcc
            gfortran
            blas
            lapack
            zlib
            stdenv.cc.cc.lib
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          # Environment variables for Python C extensions
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.stdenv.cc.cc.lib
            pkgs.zlib
            pkgs.blas
            pkgs.lapack
          ];

          # Help Python find numpy and other compiled packages
          PYTHONPATH = "${pythonEnv}/lib/python3.11/site-packages";

          # Configure development environment
          shellHook = ''
            # Validate required environment variables
            if [ -z "$GITHUB_MCP_TOKEN" ]; then
              echo "❌ ERROR: GITHUB_MCP_TOKEN environment variable is required"
              echo "   Please set your GitHub Personal Access Token:"
              echo "   export GITHUB_MCP_TOKEN=your_github_token_here"
              echo ""
              echo "   Token requirements:"
              echo "   - GitHub Personal Access Token with repo permissions"
              echo "   - Used for GitHub API access via MCP server"
              exit 1
            fi

            # Create local dependency directories
            mkdir -p .dependencies/nodejs
            mkdir -p .dependencies/rust

            # Configure Node.js/npm to use local directory
            export NPM_CONFIG_PREFIX="$PWD/.dependencies/nodejs"
            export NPM_CONFIG_CACHE="$PWD/.dependencies/nodejs/cache"
            export NODE_PATH="$PWD/.dependencies/nodejs/lib/node_modules"
            export PATH="$PWD/.dependencies/nodejs/bin:$PATH"

            # Configure Cargo to use local directory
            export CARGO_HOME="$PWD/.dependencies/rust/cargo"
            export RUSTUP_HOME="$PWD/.dependencies/rust/rustup"
            export PATH="$PWD/.dependencies/rust/cargo/bin:$PATH"

            # Ensure cargo-mcp is installed in local directory
            cargo install --locked cargo-mcp

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

            # Ensure claude code is available (install to local directory)
            if ! command -v claude &> /dev/null; then
              npx @anthropic-ai/claude-code install --force latest
            fi

            # Configure MCP servers for project
            echo "🔧 Configuring MCP servers..."

            export QDRANT_URL="http://localhost:6333"
            export COLLECTION_NAME="caxton-memory"

            pre-commit install
            pre-commit install-hooks

            # Add MCP servers with proper configuration

            npm install -g uuid-mcp
            claude mcp add sparc-memory npx @modelcontextprotocol/server-memory
            claude mcp add cargo cargo-mcp serve
            claude mcp add --transport=http --header="Authorization: Bearer $GITHUB_MCP_TOKEN" github https://api.githubcopilot.com/mcp/
            claude mcp add git npx @cyanheads/git-mcp-server

            # Configure Qdrant MCP server to use Nix Python with pre-built numpy
            # This avoids numpy compilation issues on NixOS

            # Create a wrapper script for the Qdrant MCP server
            mkdir -p .dependencies/scripts
            cat > .dependencies/scripts/qdrant-mcp-wrapper <<EOF
#!/usr/bin/env bash
# Use uv with the Nix-provided Python that has numpy pre-installed
export UV_PYTHON="${pythonEnv}/bin/python"
export PYTHONPATH="${pythonEnv}/lib/python3.11/site-packages:\$PYTHONPATH"
export LD_LIBRARY_PATH="${
              pkgs.lib.makeLibraryPath [
                pkgs.stdenv.cc.cc.lib
                pkgs.zlib
                pkgs.blas
                pkgs.lapack
              ]
            }:\$LD_LIBRARY_PATH"
uv run --with mcp-server-qdrant --python "${pythonEnv}/bin/python" -- python -m mcp_server_qdrant
EOF
            chmod +x .dependencies/scripts/qdrant-mcp-wrapper

            # Add the MCP server using our wrapper
            claude mcp add qdrant .dependencies/scripts/qdrant-mcp-wrapper

            claude mcp add uuid node .dependencies/nodejs/lib/node_modules/uuid-mcp/build/index.js

            echo "✅ MCP servers configured successfully"
            echo ""

            echo "🧠 MCP servers configured:"
            echo "   - sparc-memory: SPARC workflow knowledge storage (graph)"
            echo "   - qdrant: Semantic memory storage (dual-memory system)"
            echo "   - uuid: UUID generation for memory tracking"
            echo "   - cargo: Rust/Cargo integration"
            echo "   - github: GitHub API with Bearer token auth"
            echo "   - git: Enhanced git operations"
            echo "   - Check status: claude mcp list"
            echo ""

            echo "📦 Dependency directories:"
            echo "   - Node.js packages: .dependencies/nodejs/"
            echo "   - Rust/Cargo packages: .dependencies/rust/"
            echo ""

            echo "📋 Common commands:"
            echo "  cargo nextest run    # Run tests with nextest"
            echo "  cargo watch -x test  # Auto-run tests on changes"
            echo "  cargo clippy         # Lint code"
            echo "  cargo fmt           # Format code"
            echo "  bacon --headless     # Continuous testing"
            echo ""
          '';
        };
      }
    );
}
