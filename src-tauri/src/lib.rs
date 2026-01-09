mod commands;
mod crypto;
mod db;
mod models;
mod utils;
mod websocket;

use std::path::PathBuf;
use tauri::Manager;
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the tracing subscriber for structured logging
/// Writes to both stdout and a daily rotating log file
fn init_tracing(log_dir: PathBuf) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("pulse=debug,info"));

    // File appender with daily rotation
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "pulse.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Keep the guard alive for the lifetime of the app by leaking it
    // This is intentional - we want logging to work until the process exits
    Box::leak(Box::new(_guard));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_level(true))
        .with(
            fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing early with a temporary directory
    // We'll log to the app data dir once Tauri gives us the path
    let temp_log_dir = std::env::temp_dir().join("pulse-logs");
    std::fs::create_dir_all(&temp_log_dir).ok();
    init_tracing(temp_log_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle();

            // Log the actual log directory for user reference
            if let Ok(app_dir) = app_handle.path().app_data_dir() {
                info!(path = ?app_dir, "App data directory");
            }

            db::init_database(app_handle).expect("Failed to initialize database");
            info!("Database initialized");

            // Start WebSocket server in background using tauri's async runtime
            tauri::async_runtime::spawn(async {
                match websocket::init_websocket_server().await {
                    Ok(port) => info!(port = port, "WebSocket server started"),
                    Err(e) => error!(error = %e, "Failed to start WebSocket server"),
                }
            });

            // Note: Crypto identity is now initialized from frontend via init_identity command
            // This allows persistent key storage in OS keyring

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Chat commands
            commands::chat::get_chats,
            commands::chat::create_chat,
            // Message commands
            commands::message::get_messages,
            commands::message::send_message,
            commands::message::mark_as_read,
            commands::message::search_messages,
            commands::message::receive_message,
            commands::message::update_message_status,
            // User commands
            commands::user::get_user,
            commands::user::get_current_user,
            commands::user::update_user,
            commands::user::get_contacts,
            commands::user::add_contact,
            // WebSocket commands
            commands::websocket::broadcast_message,
            commands::websocket::get_ws_port,
            commands::websocket::get_local_ip,
            commands::websocket::get_network_status,
            commands::websocket::connect_to_peer,
            commands::websocket::get_ws_auth_token,
            // Crypto commands
            crypto::generate_keys,
            crypto::get_public_key,
            crypto::init_chat_session,
            crypto::encrypt_message,
            crypto::decrypt_message,
            crypto::has_chat_session,
            // Persistent key commands
            crypto::init_identity,
            crypto::store_peer_key,
            crypto::get_peer_key,
            crypto::ensure_chat_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
