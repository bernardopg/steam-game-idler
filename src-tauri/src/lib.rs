#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub mod achievement_manager;
pub mod automation;
pub mod command_runner;
pub mod crypto;
pub mod custom_lists;
pub mod game_data;
pub mod idling;
pub mod logging;
pub mod process_handler;
pub mod settings;
pub mod steam_utility;
pub mod trading_cards;
pub mod user_data;
pub mod utils;
use achievement_manager::*;
use automation::*;
use custom_lists::*;
use game_data::*;
use idling::*;
use logging::*;
use process_handler::*;
use settings::*;
use trading_cards::*;
use user_data::*;
use utils::*;

use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Listener, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::StateFlags;

pub fn run() {
    // Load environment variables based on the build configuration
    if cfg!(debug_assertions) {
        match crypto::deobfuscate_api_key() {
            key if !key.is_empty() => unsafe {
                std::env::set_var("KEY", key);
            },
            _ => {
                dotenv::from_filename(".env.dev").unwrap().load();
            }
        }
    } else {
        match crypto::deobfuscate_api_key() {
            key if !key.is_empty() => unsafe {
                std::env::set_var("KEY", key);
            },
            _ => {
                // A packaged build must remain usable with the API key saved in
                // Settings. Release automation deliberately does not embed a
                // maintainer secret in public artifacts.
                eprintln!("[App Init] No embedded API key; using the key configured in Settings");
            }
        }
    }

    tauri::Builder::default()
        .manage(DrpClient(Mutex::new(None)))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    StateFlags::SIZE
                        | StateFlags::POSITION
                        | StateFlags::MAXIMIZED
                        | StateFlags::DECORATIONS
                        | StateFlags::FULLSCREEN,
                )
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(move |app| setup_app(app))
        .invoke_handler(tauri::generate_handler![
            is_dev,
            is_steam_running,
            get_users,
            start_idle,
            stop_idle,
            start_farm_idle,
            stop_farm_idle,
            unlock_achievement,
            lock_achievement,
            toggle_achievement,
            unlock_all_achievements,
            lock_all_achievements,
            update_stats,
            reset_all_stats,
            log_event,
            clear_log_file,
            read_log_file,
            get_user_summary,
            get_user_summary_cache,
            delete_user_summary_file,
            get_games_list,
            get_recent_games,
            get_games_list_cache,
            delete_user_games_list_files,
            delete_all_cache_files,
            get_custom_lists,
            add_game_to_custom_list,
            remove_game_from_custom_list,
            update_custom_list,
            get_achievement_data,
            validate_session,
            validate_steam_api_key,
            get_drops_remaining,
            get_games_with_drops,
            open_file_explorer,
            get_free_games,
            anti_away,
            get_running_processes,
            kill_process_by_pid,
            kill_all_steamutil_processes,
            get_user_settings,
            update_user_settings,
            reset_user_settings,
            migrate_local_dev_profile,
            get_trading_cards,
            get_trading_cards_cache,
            update_card_data,
            delete_user_trading_card_file,
            list_trading_cards,
            get_card_price,
            remove_market_listings,
            get_tray_icon,
            is_portable,
            get_cache_dir_path,
            get_achievement_order,
            save_achievement_order,
            start_steam_status_monitor,
            start_processes_monitor,
            open_steam_login_window,
            delete_login_window_cookies,
            open_store_login_window,
            delete_store_cookies,
            redeem_free_game,
            set_zoom,
            quit_app,
            update_tray_menu,
            start_drp,
            update_drp,
            stop_drp
        ])
        .build(tauri::generate_context!())
        .expect("Error while building tauri application")
        .run(move |_, event| match event {
            tauri::RunEvent::Exit => {
                // Kill all SteamUtil processes on app exit
                tauri::async_runtime::block_on(async {
                    let _ = kill_all_steamutil_processes().await;
                });
            }
            _ => {}
        });
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    idling::cleanup_idle_work_root();

    let app_handle = app.handle();
    if let Err(error) = settings::migrate_legacy_app_data(&app_handle) {
        // Never prevent the application from starting because an old, possibly corrupt,
        // profile could not be imported. No migration marker is written on failure, so a
        // later launch can retry after the user fixes filesystem permissions.
        eprintln!("[Settings Migration] Legacy profile migration failed: {error}");
    }
    setup_window(&app_handle)?;
    if should_setup_tray_icon() {
        setup_tray_icon(app)?;
    }

    Ok(())
}

fn should_setup_tray_icon() -> bool {
    !(cfg!(debug_assertions)
        && cfg!(target_os = "linux")
        && env::var("SGI_DISABLE_DEV_TRAY").as_deref() == Ok("1"))
}

fn setup_window(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let window = app_handle.get_webview_window("main").unwrap();
    let frontend_ready = Arc::new(AtomicBool::new(false));

    // Keep the startup animation out of sight until the frontend is ready.
    // The native fallback below is deliberately independent of the webview:
    // a failed or delayed frontend must never leave a non-minimized app hidden
    // forever (notably under Wayland compositors).
    let window_clone = window.clone();
    let app_handle_clone = app_handle.clone();
    let frontend_ready_for_event = Arc::clone(&frontend_ready);
    window.listen("ready", move |_| {
        frontend_ready_for_event.store(true, Ordering::Release);
        let app_handle_for_async = app_handle_clone.clone();
        let window_for_async = window_clone.clone();
        tauri::async_runtime::spawn(async move {
            show_window_when_not_minimized(&app_handle_for_async, &window_for_async, false).await;
        });
    });

    let app_handle_for_fallback = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        // A normal startup emits `ready` much sooner. This only affects a
        // stalled frontend, while still keeping intentional tray starts hidden.
        tokio::time::sleep(Duration::from_secs(15)).await;
        if !frontend_ready.load(Ordering::Acquire) {
            show_window_when_not_minimized(&app_handle_for_fallback, &window, true).await;
        }
    });

    Ok(())
}

async fn show_window_when_not_minimized(
    app_handle: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    is_fallback: bool,
) {
    let should_start_minimized = settings::check_start_minimized_setting(app_handle)
        .await
        .unwrap_or(false);
    if should_start_minimized {
        return;
    }

    if is_fallback {
        let _ = logging::log_event(
            "[App Init] Frontend ready timed out; showing window via native fallback".into(),
            app_handle.clone(),
        );
    }

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn setup_tray_icon(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();

    // Create system tray menu
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    // Only add "update" if not portable
    let menu = if !is_portable() {
        let update = MenuItem::with_id(app, "update", "Check for updates..", true, None::<&str>)?;
        Menu::with_items(app, &[&show, &update, &quit])?
    } else {
        Menu::with_items(app, &[&show, &quit])?
    };

    // Load icon directly from binary resources
    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = Image::from_bytes(icon_bytes)?;

    TrayIconBuilder::with_id("1")
        .icon(icon)
        .tooltip("Steam Game Idler")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| match event {
            // Show app window when user left-clicks on the tray icon
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "update" => {
                // Run update check in background to avoid blocking UI
                let app_handle_for_update = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    match check_for_updates(app_handle_for_update.clone()).await {
                        Ok(_) => println!("Update check complete"),
                        Err(e) => println!("Update check failed: {}", e),
                    }
                });
            }
            "quit" => {
                app.exit(0);
            }
            _ => {
                println!("Menu item {:?} not handled", event.id);
            }
        })
        .build(app)?;

    Ok(())
}

async fn check_for_updates(app_handle: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    use tauri::Emitter;

    // Emit a status event to the WebView so the frontend can surface a toast.
    // Native notifications are disabled on Linux, so the emitted event is the
    // only feedback channel there; on other platforms it complements the
    // native notification below.
    let emit_status = |status: &str| {
        let _ = app_handle.emit("update_check_status", status);
    };

    let update = match app_handle.updater()?.check().await {
        Ok(update) => update,
        // On Linux the updater target may be absent (distributed via AUR /
        // deb / rpm / AppImage and updated through the package manager).
        // Treat that as "no in-app update available" instead of failing
        // silently with no user feedback.
        #[cfg(target_os = "linux")]
        Err(tauri_plugin_updater::Error::TargetNotFound(_)) => {
            emit_status("managed_by_package_manager");
            return Ok(());
        }
        Err(error) => {
            emit_status("error");
            return Err(error);
        }
    };

    if let Some(update) = update {
        emit_status("available");
        update
            .download_and_install(|_downloaded, _total| {}, || {})
            .await?;
        app_handle.restart();
    } else {
        emit_status("none");

        #[cfg(not(target_os = "linux"))]
        {
            use tauri_plugin_notification::NotificationExt;
            let _ = app_handle
                .notification()
                .builder()
                .title("No updates available")
                .show();
        }
    }

    Ok(())
}
