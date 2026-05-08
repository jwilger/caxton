# Verification

Run the narrow ExUnit test that proves the current RGR cycle before broader gates. Use an exact `mix test path/to/file_test.exs:line` or focused `mix test --only ...` command when possible.

Use `lefthook run pre-commit` when done with changes and fix any pending issues. Use `lefthook run ci` for the longer manual/CI gate.

For Phoenix/Ecto/Postgres work, use `MIX_ENV=test` and run database setup or migrations when the focused test depends on the database. Before handoff, run the strongest relevant gate feasible for the change: `mix format --check-formatted`, `MIX_ENV=test mix compile --warnings-as-errors --force`, `MIX_ENV=prod mix compile --warnings-as-errors --force`, `mix test --warnings-as-errors`, full Dialyzer with `mix dialyzer --halt-exit-status`, and configured static/security checks such as `mix credo --strict` and `mix sobelow`. If a gate is skipped, state why.
