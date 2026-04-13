use std::collections::BTreeMap;

use anyhow::Result;
use lopdf::{Document, Object, ObjectId};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
    ServiceExt, transport::stdio,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ── Parameter structs ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct CompressParams {
    /// Absolute path to the input PDF file.
    input_path: String,
    /// Absolute path for the output file (omit to overwrite input).
    output_path: Option<String>,
    /// Compression level 1–9 (default: 6).
    level: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct MarkdownToPdfParams {
    /// Markdown source text (GFM + math extensions supported).
    markdown: String,
    /// Absolute path where the output PDF should be written.
    output_path: String,
    /// Document title (optional).
    title: Option<String>,
    /// Template: "default" | "academic" | "report" (default: "default").
    template: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GetPdfInfoParams {
    /// Absolute path to the PDF file.
    path: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct MergePdfsParams {
    /// List of absolute paths to input PDF files (in order).
    input_paths: Vec<String>,
    /// Absolute path for the merged output PDF.
    output_path: String,
}

// ── Server struct ──────────────────────────────────────────────────────────

#[derive(Clone)]
struct PdfOpsServer {
    tool_router: ToolRouter<Self>,
}

impl PdfOpsServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

// ── Tool implementations ───────────────────────────────────────────────────

#[tool_router(router = tool_router)]
impl PdfOpsServer {
    /// Compress a PDF file and write the result to disk.
    #[tool(description = "Compress a PDF file. Returns original and compressed byte sizes.")]
    async fn compress_pdf(&self, Parameters(p): Parameters<CompressParams>) -> String {
        let level = p.level.unwrap_or(6);
        let out = p.output_path.unwrap_or_else(|| p.input_path.clone());

        let result: Result<String> = (|| {
            let input = std::fs::read(&p.input_path)?;
            let original = input.len();
            let compressed = pdfops_core::compress::compress_pdf(&input, level)?;
            let compressed_size = compressed.len();
            std::fs::write(&out, &compressed)?;
            Ok(format!(
                "Compressed '{}' → '{}'.\nOriginal: {} bytes | Compressed: {} bytes | Ratio: {:.1}%",
                p.input_path,
                out,
                original,
                compressed_size,
                100.0 * compressed_size as f64 / original as f64,
            ))
        })();

        match result {
            Ok(msg) => msg,
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Convert Markdown (with math) to a well-formatted PDF via Typst.
    #[tool(description = "Convert Markdown (with math) to a beautifully typeset PDF.")]
    async fn markdown_to_pdf(&self, Parameters(p): Parameters<MarkdownToPdfParams>) -> String {
        let typst_doc = pdfops_core::md_to_typst::markdown_to_typst(
            &p.markdown,
            p.title.as_deref(),
        );

        #[cfg(feature = "typst-engine")]
        {
            match pdfops_core::typst_compiler::compile_to_pdf(&typst_doc.source) {
                Ok(pdf_bytes) => match std::fs::write(&p.output_path, &pdf_bytes) {
                    Ok(()) => return format!(
                        "PDF written to '{}' ({} bytes).",
                        p.output_path,
                        pdf_bytes.len()
                    ),
                    Err(e) => return format!("Failed to write PDF: {e}"),
                },
                Err(e) => return format!("Typst compilation error: {e}"),
            }
        }

        let typst_path = format!("{}.typ", p.output_path);
        match std::fs::write(&typst_path, &typst_doc.source) {
            Ok(()) => format!(
                "PDF compilation not yet enabled (Phase 7).\n\
                 Typst source written to '{typst_path}'.\n\
                 Compile with: typst compile \"{typst_path}\" \"{}\"",
                p.output_path
            ),
            Err(e) => format!("Error writing Typst source: {e}"),
        }
    }

    /// Return metadata about a PDF file.
    #[tool(description = "Get metadata of a PDF file (page count, file size, etc.).")]
    async fn get_pdf_info(&self, Parameters(p): Parameters<GetPdfInfoParams>) -> String {
        let result: Result<String> = (|| {
            let data = std::fs::read(&p.path)?;
            let doc = Document::load_mem(&data)
                .map_err(|e| anyhow::anyhow!("load: {e}"))?;
            let page_count = doc.get_pages().len();
            let file_size = data.len();

            let title = doc
                .trailer
                .get(b"Info")
                .ok()
                .and_then(|obj| doc.dereference(obj).ok())
                .and_then(|(_, obj)| obj.as_dict().ok())
                .and_then(|d| d.get(b"Title").ok())
                .and_then(|t| t.as_str().ok())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .unwrap_or_else(|| "(untitled)".into());

            Ok(format!(
                "File: {}\nTitle: {title}\nPages: {page_count}\nSize: {file_size} bytes",
                p.path,
            ))
        })();

        match result {
            Ok(info) => info,
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Merge multiple PDF files into one.
    #[tool(description = "Merge multiple PDF files into a single PDF.")]
    async fn merge_pdfs(&self, Parameters(p): Parameters<MergePdfsParams>) -> String {
        let result: Result<String> = (|| {
            if p.input_paths.is_empty() {
                anyhow::bail!("input_paths must not be empty");
            }

            let mut max_id: u32 = 1;
            let mut all_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
            let mut all_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();

            for path in &p.input_paths {
                let data = std::fs::read(path)
                    .map_err(|e| anyhow::anyhow!("read '{path}': {e}"))?;
                let mut doc = Document::load_mem(&data)
                    .map_err(|e| anyhow::anyhow!("load '{path}': {e}"))?;
                doc.renumber_objects_with(max_id);
                max_id = doc.max_id + 1;

                doc.get_pages()
                    .into_values()
                    .for_each(|oid| {
                        if let Ok(obj) = doc.get_object(oid) {
                            all_pages.insert(oid, obj.to_owned());
                        }
                    });
                all_objects.extend(doc.objects);
            }

            let mut merged = Document::with_version("1.5");

            // Insert non-structural objects
            for (oid, obj) in &all_objects {
                match obj.type_name().unwrap_or(b"") {
                    b"Catalog" | b"Pages" | b"Page" | b"Outlines" | b"Outline" => {}
                    _ => { merged.objects.insert(*oid, obj.clone()); }
                }
            }

            // Collect the first Catalog and Pages objects
            let catalog_entry = all_objects.iter()
                .find(|(_, o)| o.type_name().unwrap_or(b"") == b"Catalog")
                .map(|(id, o)| (*id, o.clone()));
            let pages_entry = all_objects.iter()
                .find(|(_, o)| o.type_name().unwrap_or(b"") == b"Pages")
                .map(|(id, o)| (*id, o.clone()));

            let (catalog_id, catalog_obj) = catalog_entry
                .ok_or_else(|| anyhow::anyhow!("no Catalog in input documents"))?;
            let (pages_id, pages_obj) = pages_entry
                .ok_or_else(|| anyhow::anyhow!("no Pages root in input documents"))?;

            // Insert all Page objects with corrected Parent pointer
            for (oid, obj) in &all_pages {
                if let Ok(dict) = obj.as_dict() {
                    let mut d = dict.clone();
                    d.set("Parent", pages_id);
                    merged.objects.insert(*oid, Object::Dictionary(d));
                }
            }

            // Build updated Pages dictionary
            if let Ok(dict) = pages_obj.as_dict() {
                let mut d = dict.clone();
                d.set("Count", all_pages.len() as u32);
                d.set(
                    "Kids",
                    all_pages.keys().map(|id| Object::Reference(*id)).collect::<Vec<_>>(),
                );
                merged.objects.insert(pages_id, Object::Dictionary(d));
            }

            // Build updated Catalog dictionary
            if let Ok(dict) = catalog_obj.as_dict() {
                let mut d = dict.clone();
                d.set("Pages", pages_id);
                d.remove(b"Outlines");
                merged.objects.insert(catalog_id, Object::Dictionary(d));
            }

            merged.trailer.set("Root", catalog_id);
            merged.max_id = merged.objects.len() as u32;
            merged.renumber_objects();

            merged
                .save(&p.output_path)
                .map_err(|e| anyhow::anyhow!("save: {e}"))?;

            let out_size = std::fs::metadata(&p.output_path)?.len();
            Ok(format!(
                "Merged {} files → '{}' ({out_size} bytes).",
                p.input_paths.len(),
                p.output_path,
            ))
        })();

        match result {
            Ok(msg) => msg,
            Err(e) => format!("Error: {e}"),
        }
    }
}

// ── ServerHandler impl (delegates to tool_router) ─────────────────────────

#[tool_handler(router = self.tool_router)]
impl ServerHandler for PdfOpsServer {}

// ── Entry point ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("pdfops-mcp server starting…");
    let service = PdfOpsServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}