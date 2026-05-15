use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let env_prod_paths = [Path::new(".env.prod"), Path::new("../.env.prod")];

    let mut found_key = false;

    for env_prod_path in &env_prod_paths {
        if env_prod_path.exists() {
            let env_content =
                fs::read_to_string(env_prod_path).expect("Failed to read .env.prod file");

            for line in env_content.lines() {
                let line = line.trim();
                let Some((name, value)) = line.split_once('=') else {
                    continue;
                };

                if matches!(name.trim(), "KEY" | "STEAM_API_KEY") {
                    let api_key = value.trim().trim_matches('"');
                    if api_key.is_empty() {
                        continue;
                    }

                    println!("cargo:rustc-env=STEAM_API_KEY={}", api_key);
                    found_key = true;
                    break;
                }
            }
            if found_key {
                break;
            }
        }
    }

    if !found_key {
        if let Ok(api_key) = env::var("STEAM_API_KEY") {
            println!("cargo:rustc-env=STEAM_API_KEY={}", api_key);
        } else {
            println!(
                "cargo:warning=No API key found in .env.prod or STEAM_API_KEY environment variable"
            );
        }
    }
    println!("cargo:rerun-if-changed=.env.prod");
    println!("cargo:rerun-if-changed=../.env.prod");

    tauri_build::build()
}
