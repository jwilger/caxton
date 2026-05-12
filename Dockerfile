# syntax=docker/dockerfile:1

ARG ELIXIR_VERSION=1.19.5
ARG BUILDER_IMAGE=docker.io/elixir:${ELIXIR_VERSION}-slim

FROM ${BUILDER_IMAGE} AS builder

RUN apt-get update \
  && apt-get install -y --no-install-recommends build-essential ca-certificates git \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN mix local.hex --force \
  && mix local.rebar --force

ENV MIX_ENV=prod

COPY mix.exs mix.lock ./
RUN mix deps.get --only prod

RUN mkdir config
COPY config/config.exs config/prod.exs config/
RUN mix deps.compile

COPY priv priv
COPY lib lib
RUN mix compile

COPY assets assets
RUN mix assets.deploy

COPY config/runtime.exs config/
RUN mix release

FROM ${BUILDER_IMAGE} AS final

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libncurses6 libstdc++6 locales openssl \
  && rm -rf /var/lib/apt/lists/*

RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen \
  && locale-gen

ENV LANG=en_US.UTF-8 \
  LANGUAGE=en_US:en \
  LC_ALL=en_US.UTF-8 \
  MIX_ENV=prod \
  PHX_SERVER=true

WORKDIR /app
RUN chown nobody:nogroup /app

COPY --from=builder --chown=nobody:nogroup /app/_build/prod/rel/caxton ./

USER nobody

CMD ["/app/bin/caxton", "start"]
