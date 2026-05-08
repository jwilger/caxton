defmodule Caxton.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      CaxtonWeb.Telemetry,
      Caxton.Repo,
      {DNSCluster, query: Application.get_env(:caxton, :dns_cluster_query) || :ignore},
      {Phoenix.PubSub, name: Caxton.PubSub},
      # Start a worker by calling: Caxton.Worker.start_link(arg)
      # {Caxton.Worker, arg},
      # Start to serve requests, typically the last entry
      CaxtonWeb.Endpoint
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Caxton.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    CaxtonWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
