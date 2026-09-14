# syntax=docker/dockerfile:1.7
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1002 --no-create-home relay
COPY target/release/systemprompt-evaluator-relay /usr/local/bin/systemprompt-evaluator-relay
USER 1002:1002
EXPOSE 8090
ENTRYPOINT ["systemprompt-evaluator-relay"]
