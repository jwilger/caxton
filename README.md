# Caxton

To start your Phoenix server:

* Run `mix setup` to install and setup dependencies
* Start Phoenix endpoint with `mix phx.server` or inside IEx with `iex -S mix phx.server`

Now you can visit [`localhost:4000`](http://localhost:4000) from your browser.

Ready to run in production? Please [check our deployment guides](https://hexdocs.pm/phoenix/deployment.html).

## CI and releases

Forgejo Actions runs the project quality gates on pull requests. Pushing a tag
that starts with `v` skips the quality gates, pulls the already-tested image for
the tagged commit, and promotes it to:

* `git.johnwilger.com/slipstream/caxton:<tag>`
* `git.johnwilger.com/slipstream/caxton:latest`

The release workflow requires a repository secret named
`RELEASE_PUBLISH_TOKEN` with permission to push container packages to
`git.johnwilger.com`. If that token belongs to a service account instead of the
tag-pushing actor, also set `RELEASE_PUBLISH_USERNAME` to the token owner's
username. Protect `v*` tags in Forgejo so only release maintainers can publish
images.

The image starts the Phoenix release with `PHX_SERVER=true`. At runtime, provide
the production environment variables required by `config/runtime.exs`, including
`DATABASE_URL`, `SECRET_KEY_BASE`, and `PHX_HOST`.

## Learn more

* Official website: https://www.phoenixframework.org/
* Guides: https://hexdocs.pm/phoenix/overview.html
* Docs: https://hexdocs.pm/phoenix
* Forum: https://elixirforum.com/c/phoenix-forum
* Source: https://github.com/phoenixframework/phoenix
