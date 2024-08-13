use std::{
    sync::Arc,
    time::{self, Duration},
};

use buttplug::client::ButtplugClientDevice;
use reqwest::{Client, Error};
use tokio::sync::Mutex;

use crate::request::{get_active_player, get_event, get_score, ChampionStat};

const DELAY: Duration = time::Duration::from_millis(50);

pub fn spawn_event_thread(
    client: Arc<Mutex<Client>>,
    url: &str,
    device: Arc<ButtplugClientDevice>,
) -> tokio::task::JoinHandle<Result<(), Error>> {
    let url = url.to_owned();
    return tokio::spawn(async move {
        let mut events_count = 0;
        let mut is_over = false;
        loop {
            let client = client.lock().await;
            let result = get_event(&client, events_count, &url, is_over, &device)
                .await;
            if result.is_err() {
                return Err(result.unwrap_err())                
            }                
            let (new_count, game_status) = result.unwrap();
            events_count = new_count;
            is_over = game_status;
            tokio::time::sleep(DELAY).await;
        }
    });
}

pub fn spawn_score_thread(
    client: Arc<Mutex<Client>>,
    url: &str,
    username: &str,
    device: Arc<ButtplugClientDevice>,
) -> tokio::task::JoinHandle<Result<(), Error>> {
    let url = url.to_owned();
    let username = username.to_owned();
    return tokio::spawn(async move {
        let mut creep_score = 0;
        let mut count = 0;
        let mut deaths = 0;
        loop {
            let client = client.lock().await;
            let result = get_score(
                &client,
                &url,
                &username,
                creep_score,
                count,
                deaths,
                &device,
            )
            .await;
            if result.is_err() {
                return Err(result.unwrap_err());
            }            
            let (new_creep_score, new_count, new_deaths) = result.unwrap();
            creep_score = new_creep_score;
            count = new_count;
            deaths = new_deaths;
            tokio::time::sleep(DELAY).await;
        }
    });
}

pub fn spawn_active_thread(
    client: Arc<Mutex<Client>>,
    url: &str,
    device: Arc<ButtplugClientDevice>,
) -> tokio::task::JoinHandle<Result<(), Error>> {
    let url = url.to_owned();
    return tokio::spawn(async move {
        let mut gold = 0.0;
        let mut level = 0;
        let mut armor = 0.0;
        let mut haste = 0.0;
        let mut health = 0.0;
        let mut speed = 0.0;
        loop {
            let client = client.lock().await;
            let result = get_active_player(&client, &url, gold, level, ChampionStat { haste: haste, armor: armor, health: health, speed: speed}, &device)
                .await;
            if result.is_err() {
                return Err(result.unwrap_err());
            }                
            let (new_gold, new_stats, new_level) = result.unwrap();
            gold = new_gold;
            armor = new_stats.armor;
            haste = new_stats.haste;
            health = new_stats.health;
            speed = new_stats.speed;
            level = new_level;
            tokio::time::sleep(DELAY).await;
        }
    });
}
