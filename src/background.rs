use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use async_channel::Sender;
use chrono::Local;
use mpris::PlayerFinder;

use crate::types::{MediaInfo, StatusUpdate};

pub struct BackgroundService {
    pub update_period: Duration,
}

impl BackgroundService {
    pub fn spawn(&self, tx: Sender<StatusUpdate>) -> JoinHandle<()> {
        let period = self.update_period;
        thread::spawn(move || {
            let mut media_info = get_media_info().ok();
            let mut media_mod = 0;
            loop {
                if media_mod % 5 == 0 {
                    media_info = get_media_info().ok();
                    media_mod = 0;
                }

                media_mod += 1;

                let update = StatusUpdate {
                    time: Local::now(),
                    media: media_info.clone(),
                };

                match tx.send_blocking(update) {
                    Ok(_) => {
                        thread::sleep(period);
                        continue;
                    }
                    Err(_) => {
                        eprintln!("Error: failed to send msg through channel");
                        return;
                    }
                }
            }
        })
    }
}

fn get_media_info() -> Result<MediaInfo, &'static str> {
    match PlayerFinder::new() {
        Ok(finder) => match finder.find_active() {
            Err(_) => Err("No players found"),
            Ok(player) => match player.get_metadata() {
                Err(_) => Err("Metadata not found"),
                Ok(data) => {
                    let artist: Option<String> = match data.album_artists() {
                        None => None,
                        Some(mut artists) => {
                            artists.truncate(3);
                            Some(artists.join(", "))
                        }
                    };

                    let info = MediaInfo {
                        title: String::from(data.title().unwrap_or("Unknown")),
                        author: artist,
                        application: String::from("Idk"),
                    };

                    Ok(info)
                }
            },
        },
        Err(_) => Err("Failed to open D-Bus connection"),
    }
}
