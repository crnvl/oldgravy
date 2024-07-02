use log::{debug, error};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TwitchAuthResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
}

#[derive(Deserialize, Debug)]
pub struct TwitchGetUsersResponse {
    pub data: Vec<TwitchUser>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: u32,
    pub created_at: String,
}

#[derive(Deserialize, Debug)]
pub struct TwitchGetClipsResponse {
    pub data: Vec<TwitchClip>,
    pub pagination: Option<TwitchPagination>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TwitchClip {
    pub id: String,
    pub url: String,
    pub embed_url: String,
    pub broadcaster_id: String,
    pub broadcaster_name: String,
    pub creator_id: String,
    pub creator_name: String,
    pub video_id: String,
    pub game_id: String,
    pub language: String,
    pub title: String,
    pub view_count: u32,
    pub created_at: String,
    pub thumbnail_url: String,
    pub duration: f32,
    pub vod_offset: Option<f32>,
    pub is_featured: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TwitchPagination {
    pub cursor: Option<String>,
}

pub async fn client_credentials_grant(
    client_id: &str,
    client_secret: &str,
    grant_type: &str,
) -> TwitchAuthResponse {
    let client = reqwest::Client::new();
    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", grant_type),
        ])
        .send()
        .await
        .unwrap();

    let body = res.text().await.unwrap();
    let json: TwitchAuthResponse = serde_json::from_str(&body).unwrap();

    json
}

pub async fn get_bcid_from_name(
    client_id: &str,
    access_token: &str,
    name: &str,
) -> Option<TwitchUser> {
    let client = reqwest::Client::new();
    let res = client
        .get(&format!("https://api.twitch.tv/helix/users?login={}", name))
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Client-Id", client_id)
        .send()
        .await
        .unwrap();

    let body = res.text().await.unwrap();
    let json: TwitchGetUsersResponse = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(e) => {
            error!("Error: {}", e);
            debug!("Body: {}", body);
            error!("Failed to parse JSON");
            return None;
        }
    };

    if json.data.len() == 0 {
        error!("No user found with name: {}", name);
        return None;
    }

    Some(json.data[0].clone())
}

pub async fn get_channel_top_clips(
    client_id: &str,
    access_token: &str,
    channel_id: &str,
    limit: &u32,
    pagination: Option<String>,
) -> Option<TwitchGetClipsResponse> {  
    let client = reqwest::Client::new();
    let res = client
        .get(&format!(
            "https://api.twitch.tv/helix/clips?broadcaster_id={}&first={}{}",
            channel_id, (limit.min(&100)), match pagination {
                Some(p) => format!("&after={}", p),
                None => "".to_string(),
            }
        ))
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Client-Id", client_id)
        .send()
        .await
        .unwrap();

    let body = res.text().await.unwrap();
    let mut json: TwitchGetClipsResponse = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(e) => {
            error!("Error: {}", e);
            debug!("Body: {}", body);
            error!("Failed to parse JSON");
            return None;
        }
    };

    debug!("Duplicate check: {}", json.data[0].id);

    if limit > &100 {
        let cursor = json.pagination.clone().unwrap().cursor.unwrap();

        let remaining = limit - 100;
    
        let mut next_clips = Box::pin(get_channel_top_clips(client_id, access_token, channel_id, &remaining, Some(cursor))).await.unwrap();  
        debug!("Duplicate check: {}", next_clips.data[0].id);
        
        json.data.append(&mut next_clips.data); 
    }

    Some(json)
}
