use anyhow::Result;
use chrono::Utc;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serenity::{
    client::{Client, Context, EventHandler},
    model::gateway::Activity,
    model::gateway::Ready,
    prelude::GatewayIntents,
};
use std::{
    sync::{atomic, Arc},
    time,
};
use warp::Filter;

struct Handler;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Static {
    pub token: String,
    pub game: String,
    pub server_ip: String,
    pub server_port: i32,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Static {
    fn default() -> Self {
        Self {
            token: "".into(),
            game: "".into(),
            server_ip: "".into(),
            server_port: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Players {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub map: String,
    pub password: bool,
    pub players: Vec<Players>,
    pub maxplayers: i32,
    pub connect: String,
    pub ping: i32,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        let user = ctx.cache.current_user();
        log::info!("Logged in as {:#?}", user.name);

        let last_update = Arc::new(atomic::AtomicI64::new(0));
        let last_update_clone = Arc::clone(&last_update);

        let cfg: Static = confy::load_path("config.txt").unwrap();

        log::info!(
            "Started monitoring server {}:{}",
            cfg.server_ip,
            cfg.server_port
        );

        tokio::spawn(async move {
            let hello = warp::any().map(move || {
                let last_update_i64 = last_update_clone.load(atomic::Ordering::Relaxed);
                let now_minutes = Utc::now().timestamp() / 60;
                if (now_minutes - last_update_i64) > 5 {
                    warp::reply::with_status(
                        format!("{}", now_minutes - last_update_i64),
                        warp::http::StatusCode::SERVICE_UNAVAILABLE,
                    )
                } else {
                    warp::reply::with_status(
                        format!("{}", now_minutes - last_update_i64),
                        warp::http::StatusCode::OK,
                    )
                }
            });
            warp::serve(hello).run(([0, 0, 0, 0], 3030)).await;
        });

        // loop in seperate async
        tokio::spawn(async move {
            loop {
                match status(ctx.clone(), cfg.clone()).await {
                    Ok(item) => item,
                    Err(e) => {
                        log::error!("cant get new stats: {}", e);
                    }
                };
                last_update.store(Utc::now().timestamp() / 60, atomic::Ordering::Relaxed);
                // wait 2 minutes before redo
                tokio::time::sleep(time::Duration::from_secs(60)).await;
            }
        });
    }
}

async fn get(statics: &Static) -> Result<ServerInfo> {
    let client = reqwest::Client::new();

    let url = Url::parse(
        &format!(
            "https://gamedig.gametools.network/game/{}/{}/{}",
            statics.game, statics.server_ip, statics.server_port
        )[..],
    )
    .unwrap();

    Ok(client.get(url).send().await?.json::<ServerInfo>().await?)
}

async fn status(ctx: Context, statics: Static) -> Result<()> {
    match get(&statics).await {
        Ok(status) => {
            let server_info = format!(
                "{}/{} - {}",
                status.players.len(),
                status.maxplayers,
                status.map,
            );
            // change game activity
            ctx.set_activity(Activity::playing(server_info)).await;
        }
        Err(e) => {
            let server_info = "¯\\_(ツ)_/¯ server not found";
            ctx.set_activity(Activity::playing(server_info)).await;

            anyhow::bail!(format!("Failed to get new serverinfo: {}", e))
        }
    };
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::set_max_level(log::LevelFilter::Info);
    flexi_logger::Logger::try_with_str("warn,discord_bot=info")
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e))
        .start()?;

    let cfg: Static = match confy::load_path("config.txt") {
        Ok(config) => config,
        Err(e) => {
            log::error!("error in config.txt: {}", e);
            log::warn!("changing back to default..");
            Static {
                token: "".into(),
                game: "rust".into(),
                server_ip: "".into(),
                server_port: 0,
            }
        }
    };
    confy::store_path("config.txt", cfg.clone()).unwrap();

    // Login with a bot token from the environment
    let intents = GatewayIntents::non_privileged();
    let mut client = Client::builder(cfg.token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        log::error!("Client error: {:?}", why);
    }
    Ok(())
}
