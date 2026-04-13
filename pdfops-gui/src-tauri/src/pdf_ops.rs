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

/// Mirrors the TypeScript `Annotation` discriminated union.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AnnotationData {
    Highlight {
        id: String,
        page: u32,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        color: String,
    },
    Draw {
        id: String,
        page: u32,
        points: Vec<[f64; 2]>,
        color: String,
        #[serde(rename = "lineWidth")]
        line_width: f64,
    },
    Text {
        id: String,
        page: u32,
        x: f64,
        y: f64,
        content: String,
        color: String,
    },
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

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a CSS "#rrggbb" colour into [r, g, b] values in the 0–1 range.
fn parse_color(hex: &str) -> [f64; 3] {
    let h = hex.trim_start_matches('#');
    let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0) as f64 / 255.0;
    if h.len() >= 6 {
        [parse(&h[0..2]), parse(&h[2..4]), parse(&h[4..6])]
    } else {
        [0.0, 0.0, 0.0]
    }
}

/// Convert a lopdf Object to f64, supporting both Integer and Real variants.
/// lopdf 0.40 uses f32 for Real.
fn obj_to_f64(obj: &lopdf::Object) -> Option<f64> {
    match obj {
        lopdf::Object::Integer(i) => Some(*i as f64),
        lopdf::Object::Real(f) => Some(*f as f64),
        _ => None,
    }
}

/// Return (page_width, page_height) in PDF user-space points from the MediaBox.
fn page_dimensions(doc: &lopdf::Document, page_id: lopdf::ObjectId) -> (f64, f64) {
    let try_get = || -> Option<(f64, f64)> {
        let dict = doc.get_object(page_id).ok()?.as_dict().ok()?;
        let mb = dict.get(b"MediaBox").ok()?.as_array().ok()?;
        let w = obj_to_f64(mb.get(2)?)?;
        let h = obj_to_f64(mb.get(3)?)?;
        Some((w, h))
    };
    try_get().unwrap_or((612.0, 792.0))
}

/// Canvas-relative x (0–1, left→right) → PDF x (points, left→right).
#[inline]
fn to_pdf_x(canvas_x: f64, pw: f64) -> f64 { canvas_x * pw }

/// Canvas-relative y (0–1, top→bottom) → PDF y (points, bottom→top).
#[inline]
fn to_pdf_y(canvas_y: f64, ph: f64) -> f64 { ph * (1.0 - canvas_y) }

/// Shorthand: wrap an f64 as a lopdf Real (lopdf 0.40 stores f32 internally).
#[inline]
fn r(v: f64) -> lopdf::Object { lopdf::Object::Real(v as f32) }

/// Append `annot_id` to the page's /Annots array (creates the array if absent).
fn add_annot_to_page(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    annot_id: lopdf::ObjectId,
) -> Result<(), String> {
    use lopdf::Object;
    let annot_ref = Object::Reference(annot_id);

    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| format!("get page: {e}"))?
        .as_dict_mut()
        .map_err(|e| format!("page as dict: {e}"))?;

    match page_obj.get_mut(b"Annots") {
        Ok(existing) => match existing {
            Object::Array(arr) => arr.push(annot_ref),
            ref other => {
                *existing = Object::Array(vec![(*other).clone(), annot_ref]);
            }
        },
        Err(_) => {
            page_obj.set(b"Annots", Object::Array(vec![annot_ref]));
        }
    }
    Ok(())
}

// ── Command ───────────────────────────────────────────────────────────────────

/// Write a PDF to `output_path` with the supplied annotations baked in as
/// standard PDF annotation objects (Highlight / Ink / FreeText).
#[tauri::command]
pub async fn save_pdf_with_annotations(
    input_path: String,
    output_path: String,
    annotations: Vec<AnnotationData>,
) -> Result<(), String> {
    use lopdf::{Dictionary, Object, ObjectId};
    use std::io::Cursor;

    let data = std::fs::read(&input_path).map_err(|e| format!("read '{input_path}': {e}"))?;
    let mut doc =
        lopdf::Document::load_from(Cursor::new(&data)).map_err(|e| format!("parse PDF: {e}"))?;

    let pages: std::collections::BTreeMap<u32, ObjectId> = doc.get_pages();

    for ann in annotations {
        let (page_num, annot_dict) = match &ann {
            AnnotationData::Highlight { page, x, y, w, h, color, .. } => {
                let page_id = *pages.get(page).ok_or_else(|| format!("page {page} not found"))?;
                let (pw, ph) = page_dimensions(&doc, page_id);

                let x1 = to_pdf_x(*x, pw);
                let y2 = to_pdf_y(*y, ph);
                let x2 = to_pdf_x(x + w, pw);
                let y1 = to_pdf_y(y + h, ph);
                let [rc, gc, bc] = parse_color(color);

                let mut d = Dictionary::new();
                d.set(b"Type",    Object::Name(b"Annot".to_vec()));
                d.set(b"Subtype", Object::Name(b"Highlight".to_vec()));
                d.set(b"Rect",    Object::Array(vec![r(x1), r(y1), r(x2), r(y2)]));
                d.set(b"QuadPoints", Object::Array(vec![
                    r(x1), r(y2), r(x2), r(y2),
                    r(x1), r(y1), r(x2), r(y1),
                ]));
                d.set(b"C",  Object::Array(vec![r(rc), r(gc), r(bc)]));
                d.set(b"CA", r(0.35));
                d.set(b"F",  Object::Integer(4));
                (*page, d)
            }

            AnnotationData::Draw { page, points, color, line_width, .. } => {
                let page_id = *pages.get(page).ok_or_else(|| format!("page {page} not found"))?;
                let (pw, ph) = page_dimensions(&doc, page_id);

                let (mut min_x, mut min_y, mut max_x, mut max_y) =
                    (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
                let mut ink_stroke: Vec<Object> = Vec::with_capacity(points.len() * 2);
                for [cx, cy] in points {
                    let px = to_pdf_x(*cx, pw);
                    let py = to_pdf_y(*cy, ph);
                    ink_stroke.push(r(px));
                    ink_stroke.push(r(py));
                    min_x = min_x.min(px);
                    min_y = min_y.min(py);
                    max_x = max_x.max(px);
                    max_y = max_y.max(py);
                }

                let [rc, gc, bc] = parse_color(color);
                let lw = *line_width;

                let mut d = Dictionary::new();
                d.set(b"Type",    Object::Name(b"Annot".to_vec()));
                d.set(b"Subtype", Object::Name(b"Ink".to_vec()));
                d.set(b"Rect",    Object::Array(vec![
                    r(min_x - lw), r(min_y - lw), r(max_x + lw), r(max_y + lw),
                ]));
                d.set(b"InkList", Object::Array(vec![Object::Array(ink_stroke)]));
                d.set(b"C",  Object::Array(vec![r(rc), r(gc), r(bc)]));
                d.set(b"BS", Object::Dictionary({
                    let mut bs = Dictionary::new();
                    bs.set(b"W", r(lw));
                    bs
                }));
                d.set(b"F",  Object::Integer(4));
                (*page, d)
            }

            AnnotationData::Text { page, x, y, content, color, .. } => {
                let page_id = *pages.get(page).ok_or_else(|| format!("page {page} not found"))?;
                let (pw, ph) = page_dimensions(&doc, page_id);

                let px = to_pdf_x(*x, pw);
                let py = to_pdf_y(*y, ph);
                let [rc, gc, bc] = parse_color(color);

                let mut d = Dictionary::new();
                d.set(b"Type",     Object::Name(b"Annot".to_vec()));
                d.set(b"Subtype",  Object::Name(b"FreeText".to_vec()));
                d.set(b"Rect",     Object::Array(vec![
                    r(px), r(py - 20.0), r(px + 200.0), r(py),
                ]));
                d.set(b"Contents", Object::String(
                    content.as_bytes().to_vec(),
                    lopdf::StringFormat::Literal,
                ));
                d.set(b"C",  Object::Array(vec![r(rc), r(gc), r(bc)]));
                d.set(b"DA", Object::String(
                    b"0 0 0 rg /Helvetica 11 Tf".to_vec(),
                    lopdf::StringFormat::Literal,
                ));
                d.set(b"F",  Object::Integer(4));
                (*page, d)
            }
        };

        let page_id = *pages
            .get(&page_num)
            .ok_or_else(|| format!("page {page_num} not found"))?;
        let annot_id = doc.add_object(Object::Dictionary(annot_dict));
        add_annot_to_page(&mut doc, page_id, annot_id)?;
    }

    doc.save(&output_path)
        .map_err(|e| format!("save PDF: {e}"))?;

    Ok(())
}
