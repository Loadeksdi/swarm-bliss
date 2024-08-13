use std::time::Duration;

use buttplug::client::{ButtplugClientDevice, ScalarValueCommand};
use reqwest::{Client, Error};
use serde::Deserialize;

use crate::buttplug::vibrate;

#[derive(Debug, Deserialize, PartialEq)]
struct Event {
    #[serde(rename = "EventID")]
    event_id: i32,
    #[serde(rename = "EventName")]
    event_name: String,
    #[serde(default, rename = "GameTime")]
    game_time: f64,
    #[serde(rename = "EventTime")]
    event_time: f64,
    #[serde(default, rename = "Result")]
    result: String,
}

#[derive(Debug, Deserialize)]
struct EventList {
    #[serde(rename = "Events")]
    events: Vec<Event>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Score {
    #[serde(rename = "creepScore")]
    creep_score: i32,
    #[serde(rename = "deaths")]
    deaths: i32,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Player {
    #[serde(rename = "currentGold")]
    gold: f64,
    #[serde(rename = "level")]
    level: i32,
    #[serde(rename = "championStats")]
    stats: ChampionStat,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct GameStat {
    #[serde(default, rename = "gameMode")]
    pub game_mode: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ChampionStat {
    #[serde(rename = "abilityHaste")]
    pub(crate) haste: f64,
    #[serde(rename = "armor")]
    pub(crate) armor: f64,
    #[serde(rename = "maxHealth")]
    pub(crate) health: f64,
    #[serde(rename = "moveSpeed")]
    pub(crate) speed: f64,
}

pub async fn get_event(
    client: &Client,
    events_count: usize,
    url: &str,
    mut is_over: bool,
    device: &ButtplugClientDevice,
) -> Result<(usize, bool), Error> {
    let events = client
        .get(url)
        .send()
        .await?
        .json::<EventList>()
        .await?
        .events;
    if events.len() == 0 {
        log::warn!("No events found");
        return Ok((events_count, is_over));
    }
    if events_count == 0 && events.get(0).unwrap().event_name == "GameStart" {
        log::info!("Game started");
    }
    let last_event = events.get(events.len() - 1).unwrap();
    if !is_over && last_event.event_name == "GameEnd" {
        if last_event.result == "Lose" {
            log::info!("Defeat");
            vibrate(
                device,
                &ScalarValueCommand::ScalarValue(0.75),
                Duration::from_secs(1),
            )
            .await
            .unwrap();
        } else {
            log::info!("Victory");
            vibrate(
                device,
                &ScalarValueCommand::ScalarValue(1.0),
                Duration::from_secs(2),
            )
            .await
            .unwrap();
        }
        is_over = true;
    }
    Ok((events.len(), is_over))
}

pub async fn get_username(client: &Client, url: &str) -> Result<String, Error> {
    return client.get(url).send().await?.json::<String>().await;
}

pub async fn get_score(
    client: &Client,
    url: &str,
    username: &str,
    creep_score: i32,
    mut count: i32,
    deaths: i32,
    device: &ButtplugClientDevice,
) -> Result<(i32, i32, i32), Error> {
    let score = client
        .get(url)
        .query(&[("riotId", username)])
        .send()
        .await?
        .json::<Score>()
        .await?;
    if score.creep_score > creep_score {
        count += 1;
        if count % 2 == 0 {
            log::info!("Creep score updated: {}", score.creep_score);
            vibrate(
                device,
                &ScalarValueCommand::ScalarValue(0.4),
                Duration::from_secs_f32(0.2),
            )
            .await
            .unwrap();
        }
    }
    if score.deaths > deaths {
        log::info!("Death count updated: {}", score.deaths);
        vibrate(
            device,
            &ScalarValueCommand::ScalarValue(0.75),
            Duration::from_secs(1),
        )
        .await
        .unwrap();
    }
    Ok((score.creep_score, count, score.deaths))
}

pub async fn get_active_player(
    client: &Client,
    url: &str,
    gold: f64,
    level: i32,
    stats: ChampionStat,
    device: &ButtplugClientDevice,
) -> Result<(f64, ChampionStat, i32), Error> {
    let active_player = client.get(url).send().await?.json::<Player>().await?;
    if active_player.gold > gold {
        log::info!("Gold updated: {}", active_player.gold);
        vibrate(
            device,
            &ScalarValueCommand::ScalarValue(f64::min(active_player.gold / 15000.0, 1.0)),
            Duration::from_secs_f32(0.2),
        )
        .await
        .unwrap();
    }
    if active_player.level > level {
        log::info!("Level updated: {}", active_player.level);
        vibrate(
            device,
            &ScalarValueCommand::ScalarValue(0.5),
            Duration::from_secs_f32(0.5),
        )
        .await
        .unwrap();
    }
    if active_player.stats.haste > stats.haste {
        log::info!("Ability haste updated: {}", active_player.stats.haste);
        vibrate(
            device,
            &ScalarValueCommand::ScalarValue(f64::min(active_player.stats.haste / 500.0, 1.0)),
            Duration::from_secs_f32(0.1),
        )
        .await
        .unwrap();
        return Ok((
            active_player.gold,
            ChampionStat {
                haste: active_player.stats.haste,
                armor: active_player.stats.armor,
                health: active_player.stats.health,
                speed: active_player.stats.speed,
            },
            active_player.level,
        ));
    }
    if active_player.stats.armor > stats.armor {
        log::info!("Armor updated: {}", active_player.stats.armor);
        vibrate(
            device,
            &ScalarValueCommand::ScalarValue(f64::min(active_player.stats.armor / 300.0, 1.0)),
            Duration::from_secs_f32(0.1),
        )
        .await
        .unwrap();
        return Ok((
            active_player.gold,
            ChampionStat {
                haste: active_player.stats.haste,
                armor: active_player.stats.armor,
                health: active_player.stats.health,
                speed: active_player.stats.speed,
            },
            active_player.level,
        ));
    }
    if active_player.stats.health > stats.health {
        log::info!("Health updated: {}", active_player.stats.health);
        vibrate(
            device,
            &ScalarValueCommand::ScalarValue(f64::min(active_player.stats.health / 10000.0, 1.0)),
            Duration::from_secs_f32(0.1),
        )
        .await
        .unwrap();
        return Ok((
            active_player.gold,
            ChampionStat {
                haste: active_player.stats.haste,
                armor: active_player.stats.armor,
                health: active_player.stats.health,
                speed: active_player.stats.speed,
            },
            active_player.level,
        ));
    }
    if active_player.stats.speed > stats.speed {
        log::info!("Move speed updated: {}", active_player.stats.speed);
        vibrate(
            device,
            &ScalarValueCommand::ScalarValue(f64::min(active_player.stats.speed / 1000.0, 1.0)),
            Duration::from_secs_f32(0.1),
        )
        .await
        .unwrap();
        return Ok((
            active_player.gold,
            ChampionStat {
                haste: active_player.stats.haste,
                armor: active_player.stats.armor,
                health: active_player.stats.health,
                speed: active_player.stats.speed,
            },
            active_player.level,
        ));
    }
    Ok((
        active_player.gold,
        ChampionStat {
            haste: active_player.stats.haste,
            armor: active_player.stats.armor,
            health: active_player.stats.health,
            speed: active_player.stats.speed,
        },
        active_player.level,
    ))
}

pub async fn get_game_type(client: &Client, url: &str) -> Result<GameStat, Error> {
    return client.get(url).send().await?.json::<GameStat>().await;
}
