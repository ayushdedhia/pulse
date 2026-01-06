mod commands;
mod crypto;
mod db;
mod websocket;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle();
            db::init_database(app_handle).expect("Failed to initialize database");

            // Start WebSocket server in background using tauri's async runtime
            tauri::async_runtime::spawn(async {
                match websocket::init_websocket_server().await {
                    Ok(port) => println!("WebSocket ready on port {}", port),
                    Err(e) => eprintln!("Failed to start WebSocket server: {}", e),
                }
            });

            // Initialize crypto (generate identity keys if needed)
            let _ = crypto::get_crypto_manager().generate_identity_key();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_chats,
            commands::get_messages,
            commands::send_message,
            commands::mark_as_read,
            commands::create_chat,
            commands::get_user,
            commands::get_current_user,
            commands::update_user,
            commands::search_messages,
            commands::get_contacts,
            // Crypto commands
            crypto::generate_keys,
            crypto::get_public_key,
            crypto::init_chat_session,
            crypto::encrypt_message,
            crypto::decrypt_message,
            crypto::has_chat_session,
            // WebSocket commands
            commands::broadcast_message,
            commands::get_ws_port,
            // Contact commands
            commands::add_contact,
            commands::receive_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
