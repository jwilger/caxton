# Caxton

To start your Phoenix server:

* Run `mix setup` to install and setup dependencies
* Start Phoenix endpoint with `mix phx.server` or inside IEx with `iex -S mix phx.server`

Now you can visit [`localhost:4000`](http://localhost:4000) from your browser.

Ready to run in production? Please [check our deployment guides](https://hexdocs.pm/phoenix/deployment.html).

## CI and releases

GitHub Actions runs the project quality gates on pull requests. Release PRs
opened from same-repository `release/v*` branches with a release-bot-authored
release metadata commit additionally build and publish a release-candidate image
to the `ghcr.io/jwilger/caxton` package with both the reviewed
commit SHA tag and a `<version>-rc.<build-number>` tag, then write those image
links back to the PR body. After the release PR merges to `main`, the release
publisher promotes that reviewed image digest to:

* `ghcr.io/jwilger/caxton:<version>`
* `ghcr.io/jwilger/caxton:latest`

The release workflow uses the built-in `GITHUB_TOKEN` with `packages: write` to
publish GHCR images and a repository variable named `RELEASE_BOT_NAME` for the
trusted release identity.

The image starts the Phoenix release with `PHX_SERVER=true`. At runtime, provide
the production environment variables required by `config/runtime.exs`, including
`DATABASE_URL`, `SECRET_KEY_BASE`, and `PHX_HOST`.

## Learn more

* Official website: https://www.phoenixframework.org/
* Guides: https://hexdocs.pm/phoenix/overview.html
* Docs: https://hexdocs.pm/phoenix
* Forum: https://elixirforum.com/c/phoenix-forum
* Source: https://github.com/phoenixframework/phoenix
