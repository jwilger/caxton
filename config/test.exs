import Config

# Configure your database
#
# The MIX_TEST_PARTITION environment variable can be used
# to provide built-in test partitioning in CI environment.
# Run `mix help test` for more information.
config :caxton, Caxton.Repo,
  username: System.get_env("PGUSER") || "postgres",
  password: System.get_env("PGPASSWORD") || "postgres",
  hostname: System.get_env("PGHOST") || "127.0.0.1",
  port: String.to_integer(System.get_env("PGPORT") || "15432"),
  database: "caxton_test#{System.get_env("MIX_TEST_PARTITION")}",
  pool: Ecto.Adapters.SQL.Sandbox,
  pool_size: System.schedulers_online() * 2

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :caxton, CaxtonWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4002],
  secret_key_base: "NRbjsAMzCPg+bJRMtBgrKYXzOUu0Y3MAnkh4jrjBUlSKzTnEM29uTPtHnHGn21F7",
  server: false

# In test we don't send emails
config :caxton, Caxton.Mailer, adapter: Swoosh.Adapters.Test

# Disable swoosh api client as it is only required for production adapters
config :swoosh, :api_client, false

# Print only warnings and errors during test
config :logger, level: :warning

# Initialize plugs at runtime for faster test compilation
config :phoenix, :plug_init_mode, :runtime

# Enable helpful, but potentially expensive runtime checks
config :phoenix_live_view,
  enable_expensive_runtime_checks: true

# Sort query params output of verified routes for robust url comparisons
config :phoenix,
  sort_verified_routes_query_params: true
