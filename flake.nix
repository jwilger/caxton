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
            git
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
            export NPM_CONFIG_PREFIX="$PWD/.npm-global"
            export PATH="$PWD/.npm-global/bin:$PATH"

            # Create the directory if it doesn't exist
            mkdir -p "$PWD/.npm-global"

            # Set npm cache to also be local
            export NPM_CONFIG_CACHE="$PWD/.npm-cache"
            mkdir -p "$PWD/.npm-cache"
          '';
        };
      }
    );
}
