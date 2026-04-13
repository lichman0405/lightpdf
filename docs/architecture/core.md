# Core Library (`pdfops-core`)

`pdfops-core` is a pure Rust library shared by both the desktop app backend and the MCP server. It has no Tauri or UI dependencies.

## `compress` module

```rust
pub fn compress_pdf(input: &[u8], level: u8) -> Result<Vec<u8>>
pub fn estimate_compression(input: &[u8], level: u8) -> Result<(usize, usize)>
```

### How it works

1. Load the PDF from memory using `lopdf::Document::load_from`
2. Call `doc.compress()` — replaces uncompressed content streams with zlib-deflated versions
3. Write to a temporary file via `doc.save(&tmp_path)` (lopdf 0.40 public API)
4. Read the file back into memory and delete the temp file

The `level` parameter is accepted for API consistency but lopdf uses its own default zlib level internally.

### Typical compression ratios

| PDF type | Reduction |
|----------|-----------|
| Scanned (already compressed images) | 0–5% |
| Text-heavy academic paper | 40–65% |
| Mixed content | 20–50% |

---

## `md_to_typst` module

```rust
pub fn markdown_to_typst(markdown: &str, title: &str) -> TypstDocument
```

Converts **GitHub Flavored Markdown with LaTeX math** to a Typst source document.

### Supported Markdown elements

| Markdown | Typst output |
|----------|-------------|
| `# H1` … `###### H6` | `= H1` … `====== H6` |
| `**bold**` | `*bold*` |
| `*italic*` | `_italic_` |
| `` `code` `` | `` `code` `` |
| ` ```lang ``` ` | `#raw(lang: "lang", ...)` |
| `$x^2$` (inline math) | `$x^2$` |
| `$$...$$` (block math) | `$ ... $` |
| `[text](url)` | `#link("url")[text]` |
| `![alt](src)` | `#figure(image("src"), caption: [alt])` |
| `> quote` | `#quote[...]` |
| `- item` / `1. item` | `- item` / `+ item` |
| `- [x] task` | `- [x] task` (Typst checkbox) |
| Tables | `#table(...)` |
| Footnotes | `#footnote[...]` |

### Parser

Uses [comrak](https://github.com/kivikakk/comrak) with these extensions enabled:
- `COMMONMARK` (baseline)
- `GFM` (GitHub Flavored Markdown — tables, strikethrough, task lists)
- `MATH_DOLLARS` (inline `$` and block `$$` math)
- `FOOTNOTES`

---

## `typst_compiler` module *(feature-gated)*

Available only when built with the `typst-engine` feature:

```bash
cargo build --features typst-engine
```

```rust
pub fn compile_typst(source: &str) -> Result<Vec<u8>>
```

Compiles a Typst source string to PDF bytes using the embedded Typst engine. No external `typst` binary required.

> **Status**: The feature flag and dependency declarations are in place; the implementation file (`typst_compiler.rs`) is planned for a future release.
