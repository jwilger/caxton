defmodule CaxtonWeb.PageControllerTest do
  use CaxtonWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn, ~p"/")
    assert html_response(conn, 200) =~ "Peace of mind from prototype to production"
  end

  test "GET / includes the explicit Content-Security-Policy response header", %{conn: conn} do
    conn = get(conn, ~p"/")

    assert [
             "default-src 'self'; " <>
               "script-src 'self'; " <>
               "style-src 'self' 'unsafe-inline'; " <>
               "img-src 'self' data:; " <>
               "font-src 'self' data:; " <>
               "connect-src 'self'; " <>
               "frame-ancestors 'self'; " <>
               "base-uri 'self'; " <>
               "form-action 'self'"
           ] = get_resp_header(conn, "content-security-policy")
  end

  test "GET / keeps external app script without inline script bodies", %{conn: conn} do
    conn = get(conn, ~p"/")
    html = html_response(conn, 200)

    assert html =~ ~r/<script[^>]+src="[^"]*\/assets\/js\/app(?:-[^"]+)?\.js"[^>]*>\s*<\/script>/

    inline_script_bodies =
      for [_, attrs, body] <- Regex.scan(~r/<script\b([^>]*)>(.*?)<\/script>/s, html),
          not String.contains?(attrs, "src="),
          String.trim(body) != "" do
        body
      end

    assert inline_script_bodies == []
  end

  test "app JavaScript installs the executable theme handlers" do
    app_js = File.read!(Path.expand("assets/js/app.js", File.cwd!()))

    assert app_js =~ "import {installThemeHandlers} from \"./theme\""
    assert app_js =~ "installThemeHandlers({document, localStorage, window})"
  end
end
