mod commands;
mod crypto;
mod db;
mod models;
mod utils;
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
         // Chat commands
         commands::chat::get_chats,
         commands::chat::create_chat,
         // Message commands
         commands::message::get_messages,
         commands::message::send_message,
         commands::message::mark_as_read,
         commands::message::search_messages,
         commands::message::receive_message,
         // User commands
         commands::user::get_user,
         commands::user::get_current_user,
         commands::user::update_user,
         commands::user::get_contacts,
         commands::user::add_contact,
         // WebSocket commands
         commands::websocket::broadcast_message,
         commands::websocket::get_ws_port,
         // Crypto commands
         crypto::generate_keys,
         crypto::get_public_key,
         crypto::init_chat_session,
         crypto::encrypt_message,
         crypto::decrypt_message,
         crypto::has_chat_session,
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}
