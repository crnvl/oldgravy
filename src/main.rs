use log::{error, info};
use utils::validate::has_duplicates;

mod utils;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    info!("Starting instance");

    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
    let client_secret =
        std::env::var("TWITCH_CLIENT_SECRET").expect("TWITCH_CLIENT_SECRET not set");
    let grant_type = "client_credentials";
    let channel_name = std::env::var("TWITCH_CHANNEL_NAME").expect("TWITCH_CHANNEL_NAME not set");

    let client = utils::api::client_credentials_grant(
        client_id.as_str(),
        client_secret.as_str(),
        grant_type,
    )
    .await;

    info!("Authenticated with Twitch");

    info!("Fetching user ID for channel: {}", channel_name);
    let channel_id = utils::api::get_bcid_from_name(
        client_id.as_str(),
        &client.access_token,
        channel_name.as_str(),
    )
    .await
    .unwrap()
    .id;
    info!("Got channel ID: {}", channel_id);

    let limit = 100;

    let clips = utils::api::get_channel_top_clips(
        client_id.as_str(),
        &client.access_token,
        channel_id.as_str(),
        &limit,
        None,
    )
    .await
    .unwrap();

    info!("Fetched {} clips", clips.data.len());

    let has_duplicates = has_duplicates(clips.data.clone());
    if has_duplicates {
        error!("Duplicates found in clips");
        panic!("Duplicates found in clips");
    }
    info!("No duplicates found in clips");

    // download all clips concurrently
    download_clips(clips.data).await;

    info!("Finished downloading clips");

    // rename all files in /tmp/ to <random_number>.mp4
    let files = std::fs::read_dir("./tmp/").unwrap();
    for file in files {
        let file = file.unwrap();
        let path = file.path();
        let new_path = path.with_file_name(format!("{}.mp4", rand::random::<u32>()));
        std::fs::rename(path, new_path).unwrap();
    }

    info!("Renamed all files in /tmp/");
}

// rayon thread pool to download clips concurrently
use rayon::prelude::*;
async fn download_clips(clips: Vec<utils::api::TwitchClip>) {
    info!("Downloading clips concurrently");

    clips.into_par_iter().for_each(|clip| {
        info!("Starting download for clip: {}", clip.url);
        let future = utils::vidproc::download_clip(clip.url.as_str());

        let _ = tokio::runtime::Runtime::new().unwrap().block_on(future);
        info!("Downloaded clip: {}", clip.url);
    });
}
