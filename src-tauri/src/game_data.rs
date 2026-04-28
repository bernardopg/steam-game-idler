use crate::command_runner::apply_hidden_command_style;
use crate::utils::{get_cache_dir, get_lib_path};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::fs::{create_dir_all, remove_dir_all, remove_file, OpenOptions};
use std::io::Read;
use std::io::Write;
use tauri::Manager;

#[derive(Serialize, Deserialize)]
struct GameInfo {
    appid: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct GameData {
    appid: u64,
    name: String,
    playtime_forever: u64,
}

#[tauri::command]
pub async fn get_games_list(
    steam_id: String,
    api_key: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let app_data_dir = get_cache_dir(&app_handle)?.join(steam_id.clone());
    create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create app directory: {}", e))?;

    let temp_games_file = app_data_dir.join("temp_owned_games.json");
    let temp_games_file_str = temp_games_file.to_string_lossy().to_string();

    let exe_path = get_lib_path()?;
    let mut command = std::process::Command::new(exe_path);
    command.args(["check_ownership", "--json", &temp_games_file_str]);
    let output = apply_hidden_command_style(&mut command)
        .output()
        .map_err(|e| format!("Failed to execute check_ownership: {}", e))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let error_str = String::from_utf8_lossy(&output.stderr);

    let cs_result: Value = serde_json::from_str(&output_str).map_err(|e| {
        format!(
            "Failed to parse output: {}\nSTDOUT: {}\nSTDERR: {}",
            e, output_str, error_str
        )
    })?;

    if !cs_result["success"].as_bool().unwrap_or(false) {
        return Err(format!(
            "Returned error: {}\n{}",
            cs_result["error"].as_str().unwrap_or("Unknown error"),
            cs_result["suggestion"].as_str().unwrap_or("")
        ));
    }

    let mut file = File::open(&temp_games_file)
        .map_err(|e| format!("Failed to open temp games file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read temp games file: {}", e))?;
    let cs_games_data: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse temp games JSON: {}", e))?;

    let cs_games = cs_games_data["games"]
        .as_array()
        .ok_or_else(|| "Did not return games array".to_string())?;

    let key = api_key.unwrap_or_else(|| {
        std::env::var("KEY")
            .or_else(|_| std::env::var("STEAM_API_KEY"))
            .unwrap_or_default()
    });

    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true&include_free_sub=true&skip_unvetted_apps=false&include_extended_appinfo=false",
        key, steam_id
    );

    let client = Client::new();
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let web_api_body: Value = response.json().await.map_err(|e| e.to_string())?;

    let mut playtime_map: HashMap<u64, u64> = HashMap::new();
    if let Some(web_games) = web_api_body["response"]["games"].as_array() {
        for game in web_games {
            if let (Some(appid), Some(playtime)) =
                (game["appid"].as_u64(), game["playtime_forever"].as_u64())
            {
                playtime_map.insert(appid, playtime);
            }
        }
    }

    let mut merged_games: Vec<GameData> = Vec::new();
    for game in cs_games {
        if let (Some(appid), Some(name)) = (game["appid"].as_u64(), game["name"].as_str()) {
            let playtime_forever = playtime_map.get(&appid).copied().unwrap_or(0);
            merged_games.push(GameData {
                appid,
                name: name.to_string(),
                playtime_forever,
            });
        }
    }

    let cs_appids: std::collections::HashSet<u64> = merged_games.iter().map(|g| g.appid).collect();
    if let Some(web_games) = web_api_body["response"]["games"].as_array() {
        for game in web_games {
            if let (Some(appid), Some(name), Some(playtime)) = (
                game["appid"].as_u64(),
                game["name"].as_str(),
                game["playtime_forever"].as_u64(),
            ) {
                if !cs_appids.contains(&appid) {
                    merged_games.push(GameData {
                        appid,
                        name: name.to_string(),
                        playtime_forever: playtime,
                    });
                }
            }
        }
    }

    let game_data = json!({
        "games_list": merged_games
    });

    let file_name = format!("games_list.json");
    let games_file_path = app_data_dir.join(file_name);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&games_file_path)
        .map_err(|e| format!("Failed to open games list file: {}", e))?;

    let json_string = serde_json::to_string_pretty(&game_data)
        .map_err(|e| format!("Failed to serialize games list: {}", e))?;
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write games list to file: {}", e))?;

    let _ = std::fs::remove_file(&temp_games_file);

    Ok(game_data)
}

#[tauri::command]
pub async fn get_recent_games(
    steam_id: String,
    api_key: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let key = api_key.unwrap_or_else(|| {
        std::env::var("KEY")
            .or_else(|_| std::env::var("STEAM_API_KEY"))
            .unwrap_or_default()
    });

    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetRecentlyPlayedGames/v1/?key={}&steamid={}&count=4",
        key, steam_id
    );

    let client = Client::new();
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let body: Value = response.json().await.map_err(|e| e.to_string())?;

    let file_path = get_cache_dir(&app_handle)?
        .join(steam_id.clone())
        .join("games_list.json");

    if !file_path.exists() {
        return Ok(json!({
            "games_list": [],
            "recent_games": body["response"]["games"].as_array().cloned().unwrap_or_default()
        }));
    }

    let mut file =
        File::open(&file_path).map_err(|e| format!("Failed to open games list file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read games list file: {}", e))?;
    let games_list_json: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse games list JSON: {}", e))?;

    Ok(json!({
        "games_list": games_list_json["games_list"].as_array().cloned().unwrap_or_default(),
        "recent_games": body["response"]["games"].as_array().cloned().unwrap_or_default()
    }))
}

#[tauri::command]
pub async fn get_games_list_cache(
    steam_id: String,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let file_path = get_cache_dir(&app_handle)?
        .join(steam_id.clone())
        .join("games_list.json");

    if !file_path.exists() {
        return Ok(json!({
            "games_list": []
        }));
    }

    let mut file =
        File::open(&file_path).map_err(|e| format!("Failed to open games list file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read games list file: {}", e))?;
    let games_list_json: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse games list JSON: {}", e))?;

    Ok(games_list_json)
}

#[tauri::command]
pub async fn delete_user_games_list_files(
    steam_id: String,
    app_handle: tauri::AppHandle,
) -> Result<Value, String> {
    let app_data_dir = get_cache_dir(&app_handle)?.join(steam_id.clone());
    let games_file_path = app_data_dir.join("games_list.json");
    let temp_games_file_path = app_data_dir.join("temp_owned_games.json");

    let _ = remove_file(games_file_path);
    let _ = remove_file(temp_games_file_path);

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_all_cache_files(app_handle: tauri::AppHandle) -> Result<Value, String> {
    let cache_dir = get_cache_dir(&app_handle)?;

    if cache_dir.exists() {
        remove_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to delete cache directory: {}", e))?;
    }

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn get_free_games() -> Result<serde_json::Value, String> {
    let client = Client::new();
    let url =
        "https://store.steampowered.com/search/?l=english&maxprice=free&specials=1&category1=998";

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

    let html = response.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html);

    let a_selector = Selector::parse("a.search_result_row").unwrap();
    let title_selector = Selector::parse("span.title").unwrap();

    let mut free_games = Vec::new();

    for element in document.select(&a_selector) {
        if let Some(app_id) = element.value().attr("data-ds-appid") {
            if let Some(title_element) = element.select(&title_selector).next() {
                let name = title_element.text().collect::<String>();
                free_games.push(GameInfo {
                    appid: app_id.to_string(),
                    name: name.trim().to_string(),
                });
            }
        }
    }

    Ok(json!({ "games": free_games }))
}

#[tauri::command]
pub async fn redeem_free_game(
    app_handle: tauri::AppHandle,
    app_id: String,
) -> Result<Value, String> {
    use std::time::Duration;

    let url = format!("https://store.steampowered.com/app/{}", app_id);

    let window = tauri::webview::WebviewWindowBuilder::new(
        &app_handle,
        &format!("steam-redeem-{}", app_id),
        tauri::WebviewUrl::External(url.parse().unwrap()),
    )
    .title(&format!("Redeeming Free Game {}", app_id))
    .inner_size(0.0, 0.0)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    tokio::time::sleep(Duration::from_millis(5000)).await;

    for _ in 0..5 {
        if let Some(webview) = window.get_webview(&format!("steam-redeem-{}", app_id)) {
            let js_check = r#"
                (function() {
                    const btn = document.querySelector('.btn_addtocart a[href*="addToCart"]');
                    if (!btn) {
                        throw new Error('Button not found');
                    }

                    const href = btn.getAttribute('href');
                    const match = href.match(/addToCart\(\s*(\d+)\s*\)/);
                    if (!match) {
                        throw new Error('No match for product ID');
                    }

                    addToCart(match[1]);
                    return true;
                })();
            "#;

            match webview.eval(js_check) {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    let _ = window.close();
                    return Ok(serde_json::json!({
                        "success": true,
                        "message": "Free game redeemed successfully"
                    }));
                }
                Err(e) => {
                    println!("JS execution error: {}", e);
                }
            }
        } else {
            return Ok(serde_json::json!({
                "success": false,
                "message": "Redeem window closed unexpectedly"
            }));
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let _ = window.close();
    Ok(serde_json::json!({
        "success": false,
        "message": "Could not find redeem button or game is not free"
    }))
}

#[tauri::command]
pub async fn scrape_game_banner(app_id: u32) -> Result<Value, String> {
    let client = Client::new();
    let url = format!("https://store.steampowered.com/app/{}", app_id);

    let html = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let selector = Selector::parse("img.game_header_image_full")
        .map_err(|e| format!("Failed to create selector: {}", e))?;

    let image_url = document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("src"))
        .unwrap_or_default();

    Ok(json!({ "imageUrl": image_url }))
}

#[tauri::command]
pub async fn scrape_game_description(app_id: u32) -> Result<Value, String> {
    let client = Client::new();
    let url = format!("https://store.steampowered.com/app/{}", app_id);

    let html = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let selector = Selector::parse("div.game_description_snippet")
        .map_err(|e| format!("Failed to create selector: {}", e))?;

    let description = document
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    Ok(json!({ "description": description }))
}
