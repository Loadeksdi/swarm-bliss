// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod buttplug;
mod request;
mod thread;

use ::buttplug::client::ButtplugClientDevice;
use buttplug::{display_device, start_buttplug};
use dotenv::dotenv;
use request::{get_game_type, get_username};
use reqwest::Client;
use simple_logger::SimpleLogger;
use std::{sync::Arc, time::Duration};
use tauri::{CustomMenuItem, Manager, Menu};
use thread::{spawn_active_thread, spawn_event_thread, spawn_score_thread};
use tokio::sync::Mutex;

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

const ACTIVE_PLAYER_NAME_URL: &str = "https://127.0.0.1:2999/liveclientdata/activeplayername";
const ACTIVE_PLAYER_URL: &str = "https://127.0.0.1:2999/liveclientdata/activeplayer";
const PLAYER_SCORE_URL: &str = "https://127.0.0.1:2999/liveclientdata/playerscores";
const EVENT_DATA_URL: &str = "https://127.0.0.1:2999/liveclientdata/eventdata";
const GAME_STATS_URL: &str = "https://127.0.0.1:2999/liveclientdata/gamestats";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    log::info!("Starting Swarm Bliss");

    let buttplug_client =  Arc::new(Mutex::new(start_buttplug().await?));
    let buttplug_client_clone1 = Arc::clone(&buttplug_client);

    tauri::Builder::default()
        .menu(create_menu())
        .setup(move |app| {
            let window = app.get_window("main").unwrap();
            let _ = window.set_title("Swarm Bliss");
            let buttplug_client_clone2 = Arc::clone(&buttplug_client_clone1);
            let id = app.listen_global("device-confirmed",  move |event|{
                let buttplug_client_clone3 = Arc::clone(&buttplug_client_clone2);
                log::debug!("got device-confirmed with payload {:?}", event.payload());
                tauri::async_runtime::spawn(async move {
                    let buttplug_client_clone4 = Arc::clone(&buttplug_client_clone3);
                    let device = display_device(buttplug_client_clone4).await.unwrap();
                    let client = reqwest::Client::builder()
                        .danger_accept_invalid_certs(true)
                        .build()
                        .unwrap();
                    start_all_threads(client.clone(), device).await;
                });
            });

            //app.unlisten(id);
            

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

fn create_menu() -> Menu {
    let home: CustomMenuItem = CustomMenuItem::new("home".to_string(), "Home");
    let logs = CustomMenuItem::new("logs".to_string(), "Logs");
    let settings = CustomMenuItem::new("settngs".to_string(), "Settings");
    Menu::new().add_item(home).add_item(logs).add_item(settings)
}

async fn wait_game_start(client: &reqwest::Client, url: &str) {
    loop {
        let game_stat = get_game_type(&client, url).await;
        if game_stat.is_err() {
            log::warn!("Game not started yet");
            log::debug!("Error while fetching Swarm game: {:?}", game_stat.err());
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        } else if game_stat.unwrap().game_mode == "STRAWBERRY" {
            break;
        }
    }
}

async fn start_all_threads(client: Client, device: Arc<ButtplugClientDevice> ) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let clone = client.clone();
        wait_game_start(&clone, GAME_STATS_URL).await;

        let username = get_username(&clone, ACTIVE_PLAYER_NAME_URL)
            .await
            .map_err(|e| log::error!("Error while fetching username: {:?}", e))
            .unwrap();
        let cloned_username = username.clone();

        let threadsafe_client = Arc::new(Mutex::new(clone));
        let event_client = Arc::clone(&threadsafe_client);
        let event_device = Arc::clone(&device);
        let event_handle = spawn_event_thread(event_client, EVENT_DATA_URL, event_device);

        let score_client = Arc::clone(&threadsafe_client);
        let score_device = Arc::clone(&device);
        let score_handle = spawn_score_thread(
            score_client,
            PLAYER_SCORE_URL,
            cloned_username.as_str(),
            score_device,
        );

        let active_client = Arc::clone(&threadsafe_client);
        let active_device = Arc::clone(&device);
        let active_handle = spawn_active_thread(active_client, ACTIVE_PLAYER_URL, active_device);

        match tokio::try_join!(event_handle, score_handle, active_handle) {
            Ok(_) => log::info!("Game ended"),
            Err(e) => log::error!(
                "Error while joining threads, game is probably already over: {:?}",
                e
            ),
        }
    }
}
