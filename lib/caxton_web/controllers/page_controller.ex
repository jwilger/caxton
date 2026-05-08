defmodule CaxtonWeb.PageController do
  use CaxtonWeb, :controller

  def home(conn, _params) do
    render(conn, :home)
  end
end
