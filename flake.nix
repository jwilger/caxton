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

          # Configure npm to use local directory for global packages
          shellHook = ''
            # Ensure claude code and claude-flow are available
            # First check if claude code is installed
            if ! command -v claude &> /dev/null; then
              npx @anthropic-ai/claude-code install --force latest
            fi
          '';
        };
      }
    );
}
