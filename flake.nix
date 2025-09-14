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
            cargo-nextest
            nodejs
            uv
            git
            gh
            jq
            pre-commit
            pkg-config
            openssl
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          # Configure development environment
          shellHook = ''
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

            # For AI coding assistants
            cargo install --locked cargo-mcp

            pre-commit install
            pre-commit install-hooks
          '';
        };
      }
    );
}
