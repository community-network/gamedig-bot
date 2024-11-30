# Gamedig Discord status bot

A Discord bot to check the status of your server in different gamedig supported games, as requested by [onemanarmy](https://www.superinfantryclan.com/)


## Using the bot

You can run it with Docker (Docker Compose):

```docker
services:
  ace-bot-1:
    image: ghcr.io/community-network/gamedig-bot/server-bot-rust:latest
    restart: always
    environment:
      - token=TOKEN
      - game=rust
      - server_ip=194.163.156.99
      - server_port=28016
    healthcheck:
      test: ["CMD", "curl", "-f", "http://127.0.0.1:3030/"]
      interval: "60s"
      timeout: "3s"
      start_period: "5s"
      retries: 3
```

or set the config to the correct info in a config.txt next to this executable to use:

```yaml
# discord bot token
token = ''
# game to track
game = 'rust'
# server ip
server_ip = '194.163.156.99'
# and queryport
server_port = 28016
```