FROM rust:1.82 AS builder
WORKDIR /usr/src/myapp
COPY . .
ARG github_token 
RUN git config --global credential.helper store && echo "https://zefanjajobse:${github_token}@github.com" > ~/.git-credentials && cargo install --path .

FROM debian:bookworm-slim

ENV token default_token_value
ENV game default_game_value
ENV server_ip default_server_ip_value
ENV server_port default_server_port_value

HEALTHCHECK --interval=5m --timeout=3s --start-period=5s \
  CMD curl -f http://127.0.0.1:3030/ || exit 1

COPY --from=builder /usr/local/cargo/bin/discord_bot /usr/local/bin/discord_bot
RUN apt-get update && apt-get install --assume-yes curl
CMD echo "token = '$token'\ngame = '$game'\nserver_ip = '$server_ip'\nserver_port = $server_port" > config.txt && discord_bot
