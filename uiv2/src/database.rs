use crate::gamelist::GameList;
use reqwasm::http::Request;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// pub async fn get_gamedata() -> Result<GameData, anyhow::Error> {
//     // consider making a Client in yew and passing it in here to prevent reopening channels
//     let gamedata_json = reqwest::get("/gamedata").await?.text().await?;
//     let gamedata: GameData = serde_json::from_str(&gamedata_json)?;
//     Ok(gamedata)
// }

// pub async fn post_gamedata(gamedata: &GameData) -> Result<(), anyhow::Error> {
//     let client = reqwest::Client::new();
//     let gamedata_json = serde_json::to_string(gamedata)?;
//     let _ = client
//         .post("/post_gamedata")
//         .body(gamedata_json)
//         .send()
//         .await?;
//     Ok(())
// }

pub async fn get_object<T>(url: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let object_json = Request::get(url)
        .send()
        .await
        .map_err(|_| {
            log::info!("Request failed");
            "Request failed".to_owned()
        })?
        .text()
        .await
        .map_err(|_| {
            log::info!("Failed to get response body");
            "Failed to get response body".to_owned()
        })?;

    let gamelist: T = serde_json::from_str(&object_json).map_err(|_| {
        log::info!("Failed to deserialize JSON: {}", &object_json);
        "Failed to deserialize JSON".to_owned()
    })?;
    return Ok(gamelist);
}

pub async fn post_object<T>(url: &str, object: T) -> Result<(), String>
where
    T: Serialize,
{
    let object_json = serde_json::to_string(&object).map_err(|_| "Serializing failed")?;
    Request::post(url)
        .body(object_json)
        .send()
        .await
        .map_err(|_| "Post request failed");
    Ok(())
}

pub async fn join_game(game_id: usize) {
    Request::get(&format!("joingame/{}", game_id));
}
