use serde::{Deserialize, Serialize};
use tauri::AppHandle;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct PdfInfo {
    pub path: String,
    pub page_count: u32,
    pub file_size: u64,
    pub title: Option<String>,
    pub has_forms: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CompressResult {
    pub output_path: String,
    pub original_bytes: u64,
    pub compressed_bytes: u64,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Open a native file-picker dialog and return the chosen PDF path.
/// The frontend can then call `read_pdf_bytes` to get the file content.
#[tauri::command]
pub async fn open_pdf_dialog(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app
        .dialog()
        .file()
        .add_filter("PDF Documents", &["pdf"])
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned());

    Ok(path)
}

/// Read raw PDF bytes from disk and return them base64-encoded.
///
/// PDF.js on the frontend can decode this back to a `Uint8Array`.
#[tauri::command]
pub async fn read_pdf_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("read '{path}': {e}"))
}

/// Return basic metadata for a PDF file (page count, size, title, forms).
#[tauri::command]
pub async fn get_pdf_info(path: String) -> Result<PdfInfo, String> {
    let data = std::fs::read(&path).map_err(|e| format!("read: {e}"))?;
    let doc = lopdf::Document::load_mem(&data).map_err(|e| format!("parse: {e}"))?;

    let page_count = doc.get_pages().len() as u32;

    let title = doc
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|obj| doc.dereference(obj).ok())
        .and_then(|(_, o)| o.as_dict().ok().cloned())
        .and_then(|d| {
            d.get(b"Title")
                .ok()
                .and_then(|t| t.as_str().ok())
                .map(|s| String::from_utf8_lossy(s).into_owned())
        });

    // A PDF has interactive forms if the AcroForm entry exists in the catalog.
    let has_forms = doc
        .catalog()
        .and_then(|c| c.get(b"AcroForm"))
        .is_ok();

    Ok(PdfInfo {
        path,
        page_count,
        file_size: data.len() as u64,
        title,
        has_forms,
    })
}

/// Compress a PDF file and overwrite (or write to a new path).
#[tauri::command]
pub async fn compress_pdf(
    input_path: String,
    output_path: Option<String>,
    level: Option<u8>,
) -> Result<CompressResult, String> {
    let level = level.unwrap_or(6);
    let out = output_path.unwrap_or_else(|| input_path.clone());

    let data = std::fs::read(&input_path).map_err(|e| format!("read: {e}"))?;
    let original = data.len() as u64;

    let compressed =
        pdfops_core::compress::compress_pdf(&data, level).map_err(|e| e.to_string())?;
    let compressed_size = compressed.len() as u64;

    std::fs::write(&out, &compressed).map_err(|e| format!("write: {e}"))?;

    Ok(CompressResult {
        output_path: out,
        original_bytes: original,
        compressed_bytes: compressed_size,
    })
}
