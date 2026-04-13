mod pdf_ops;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            pdf_ops::open_pdf_dialog,
            pdf_ops::read_pdf_bytes,
            pdf_ops::get_pdf_info,
            pdf_ops::compress_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
