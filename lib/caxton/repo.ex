defmodule Caxton.Repo do
  use Ecto.Repo,
    otp_app: :caxton,
    adapter: Ecto.Adapters.Postgres
end
