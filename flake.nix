{
  description = "Phoenix development shell with isolated tool caches and PostgreSQL";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          f (import nixpkgs { inherit system; }));
    in
    {
      devShells = forAllSystems (pkgs:
        let
          elixir = pkgs.beam28Packages.elixir_1_19;
          postgresql = pkgs.postgresql;
          nodejs = pkgs.nodejs;

          pgStart = pkgs.writeShellApplication {
            name = "pg-start";
            runtimeInputs = [ postgresql ];
            text = ''
              set -euo pipefail

              deps_root="''${CAXTON_DEPS_ROOT:-$PWD/.dependencies}"
              pgdata="''${PGDATA:-$deps_root/postgres/data}"
              pglog="''${PGLOG:-$deps_root/postgres/postgres.log}"
              pgsocketdir="''${PGSOCKETDIR:-$deps_root/postgres/run}"
              pghost="''${PGHOST:-127.0.0.1}"
              pgport="''${PGPORT:-15432}"
              pguser="''${PGUSER:-postgres}"

              mkdir -p "$(dirname "$pgdata")" "$(dirname "$pglog")" "$pgsocketdir"

              if [ ! -s "$pgdata/PG_VERSION" ]; then
                initdb -D "$pgdata" --encoding=UTF8 --no-locale --auth=trust --username="$pguser"
              fi

              if pg_ctl -D "$pgdata" status >/dev/null 2>&1; then
                echo "PostgreSQL is already running on $pghost:$pgport"
                exit 0
              fi

              pg_ctl -D "$pgdata" -l "$pglog" -o "-c listen_addresses='$pghost' -p $pgport -c unix_socket_directories='$pgsocketdir'" start

              psql -h "$pghost" -p "$pgport" -U "$pguser" -d postgres -v ON_ERROR_STOP=1 \
                -c "ALTER ROLE \"$pguser\" WITH LOGIN SUPERUSER PASSWORD 'postgres';" >/dev/null

              echo "PostgreSQL started on $pghost:$pgport"
              echo "Data directory: $pgdata"
              echo "Log file: $pglog"
            '';
          };

          pgStop = pkgs.writeShellApplication {
            name = "pg-stop";
            runtimeInputs = [ postgresql ];
            text = ''
              set -euo pipefail

              deps_root="''${CAXTON_DEPS_ROOT:-$PWD/.dependencies}"
              pgdata="''${PGDATA:-$deps_root/postgres/data}"

              if [ ! -s "$pgdata/PG_VERSION" ]; then
                echo "PostgreSQL data directory has not been initialized: $pgdata"
                exit 0
              fi

              pg_ctl -D "$pgdata" stop
            '';
          };

          pgStatus = pkgs.writeShellApplication {
            name = "pg-status";
            runtimeInputs = [ postgresql ];
            text = ''
              set -euo pipefail

              deps_root="''${CAXTON_DEPS_ROOT:-$PWD/.dependencies}"
              pgdata="''${PGDATA:-$deps_root/postgres/data}"

              if [ ! -s "$pgdata/PG_VERSION" ]; then
                echo "PostgreSQL data directory has not been initialized: $pgdata"
                exit 1
              fi

              pg_ctl -D "$pgdata" status
            '';
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              elixir
              postgresql
              nodejs
              pgStart
              pgStop
              pgStatus
            ];

            shellHook = ''
              export CAXTON_DEPS_ROOT="$PWD/.dependencies"

              export MIX_HOME="$CAXTON_DEPS_ROOT/elixir/mix"
              export HEX_HOME="$CAXTON_DEPS_ROOT/elixir/hex"
              export REBAR_CACHE_DIR="$CAXTON_DEPS_ROOT/elixir/rebar"

              export NPM_CONFIG_PREFIX="$CAXTON_DEPS_ROOT/nodejs/global"
              export NPM_CONFIG_CACHE="$CAXTON_DEPS_ROOT/nodejs/cache"
              export PNPM_HOME="$CAXTON_DEPS_ROOT/nodejs/pnpm"
              export YARN_CACHE_FOLDER="$CAXTON_DEPS_ROOT/nodejs/yarn/cache"
              export COREPACK_HOME="$CAXTON_DEPS_ROOT/nodejs/corepack"

              export PGHOST="''${PGHOST:-127.0.0.1}"
              export PGPORT="''${PGPORT:-15432}"
              export PGUSER="''${PGUSER:-postgres}"
              export PGPASSWORD="''${PGPASSWORD:-postgres}"
              export PGDATA="''${PGDATA:-$CAXTON_DEPS_ROOT/postgres/data}"

              export PATH="$MIX_HOME/bin:$MIX_HOME/escripts:$NPM_CONFIG_PREFIX/bin:$PNPM_HOME:$PATH"

              mkdir -p \
                "$MIX_HOME" \
                "$HEX_HOME" \
                "$REBAR_CACHE_DIR" \
                "$NPM_CONFIG_PREFIX" \
                "$NPM_CONFIG_CACHE" \
                "$PNPM_HOME" \
                "$YARN_CACHE_FOLDER" \
                "$COREPACK_HOME" \
                "$(dirname "$PGDATA")"

              mix local.hex --force >/dev/null
              mix archive.install hex phx_new --force >/dev/null

              echo "Elixir: $(elixir --version | tail -n 1)"
              echo "PostgreSQL: $(postgres --version)"
              echo "PostgreSQL env: PGHOST=$PGHOST PGPORT=$PGPORT PGUSER=$PGUSER PGDATA=$PGDATA"
              echo "Start database: pg-start"
            '';
          };
        });
    };
}
