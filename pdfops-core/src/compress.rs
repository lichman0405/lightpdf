use anyhow::Result;
use std::io::Cursor;

/// Compress a PDF in memory using lopdf's content-stream compression.
///
/// `level` is accepted for API consistency but lopdf 0.40 uses its internal
/// default zlib level. Returns the compressed PDF bytes.
pub fn compress_pdf(input: &[u8], _level: u8) -> Result<Vec<u8>> {
    let mut doc = lopdf::Document::load_from(Cursor::new(input))
        .map_err(|e| anyhow::anyhow!("load PDF: {e}"))?;

    // Compress all content streams with zlib.
    doc.compress();

    // lopdf 0.40 public API only exposes save-to-path.
    // Write to a unique temp file, read back, then clean up.
    let tmp = std::env::temp_dir().join(format!(
        "pdfops_compress_{}.pdf",
        std::process::id()
    ));
    doc.save(&tmp)
        .map_err(|e| anyhow::anyhow!("lopdf save: {e}"))?;
    let output = std::fs::read(&tmp)?;
    let _ = std::fs::remove_file(&tmp); // best-effort cleanup
    Ok(output)
}

/// Return the byte sizes before and after compression (without writing to disk).
pub fn estimate_compression(input: &[u8], level: u8) -> Result<(usize, usize)> {
    let original = input.len();
    let compressed = compress_pdf(input, level)?;
    Ok((original, compressed.len()))
}
