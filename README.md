# Caxton

To start your Phoenix server:

* Run `mix setup` to install and setup dependencies
* Start Phoenix endpoint with `mix phx.server` or inside IEx with `iex -S mix phx.server`

Now you can visit [`localhost:4000`](http://localhost:4000) from your browser.

Ready to run in production? Please [check our deployment guides](https://hexdocs.pm/phoenix/deployment.html).

## CI and releases

Forgejo Actions runs the project quality gates on pull requests. Release PRs
opened from same-repository `release/v*` branches with a release-bot-authored
release metadata commit additionally build and publish a release-candidate image at
`git.johnwilger.com/slipstream/caxton-pr:release-v<version>-<sha>` and write
that image link back to the PR body. After the release PR merges to `main`, the
release publisher promotes that reviewed image digest to:

* `git.johnwilger.com/slipstream/caxton:<version>`
* `git.johnwilger.com/slipstream/caxton:latest`

The release workflow requires a repository secret named
`RELEASE_PUBLISH_TOKEN` with permission to push container packages to
`git.johnwilger.com` and a repository variable named `RELEASE_BOT_NAME` for the
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
