# Architecture Overview

LightPDF is a **Cargo workspace** containing three crates that share a common core library.

## Crate relationships

```
┌─────────────────────────────────────────────────────┐
│                  Cargo Workspace                     │
│                                                      │
│  ┌─────────────────┐    ┌─────────────────────────┐ │
│  │  pdfops-gui      │    │       pdfops-mcp        │ │
│  │  (Tauri v2 App)  │    │    (MCP stdio server)   │ │
│  │                  │    │                         │ │
│  │  React + PDF.js  │    │  rmcp + schemars        │ │
│  │  Rust backend    │    │  tokio async             │ │
│  └────────┬─────────┘    └───────────┬─────────────┘ │
│           │                          │               │
│           └──────────────┬───────────┘               │
│                          ▼                           │
│              ┌───────────────────────┐               │
│              │     pdfops-core       │               │
│              │   (pure Rust library) │               │
│              │                       │               │
│              │  • compress.rs        │               │
│              │  • md_to_typst.rs     │               │
│              │  • typst_compiler.rs  │               │
│              │    (feature-gated)    │               │
│              └───────────────────────┘               │
└─────────────────────────────────────────────────────┘
```

## pdfops-core

A **pure Rust library** with no OS-level dependencies. Used by both the GUI backend and the MCP server.

| Module | Purpose |
|--------|---------|
| `compress` | lopdf-based PDF content-stream compression |
| `md_to_typst` | GFM + LaTeX math → Typst source conversion via comrak |
| `typst_compiler` | *(feature: `typst-engine`)* Typst → PDF compilation |

## pdfops-gui

A **Tauri v2** application.

```
pdfops-gui/
├── src/                  # React 19 + TypeScript
│   ├── App.tsx           # Root component, state management
│   ├── components/
│   │   ├── Toolbar.tsx   # Top toolbar: file ops, nav, zoom, annotation modes
│   │   └── PdfViewer.tsx # PDF.js canvas + SVG annotation overlay
│   └── types/
│       └── annotations.ts # TypeScript annotation types
└── src-tauri/src/
    ├── lib.rs            # Tauri Builder + command registration
    └── pdf_ops.rs        # All Tauri commands + lopdf annotation write
```

### Frontend → Backend communication

```
React UI
  │
  │  invoke("command_name", { ...args })
  ▼
Tauri IPC (serde_json serialization)
  │
  ▼
Rust command (pdf_ops.rs)
  │
  ├── File I/O (std::fs)
  └── pdfops-core (compress / annotate)
```

### Tauri commands

| Command | Description |
|---------|-------------|
| `open_pdf_dialog` | Native file picker → returns path |
| `read_pdf_bytes` | Read file bytes for PDF.js rendering |
| `get_pdf_info` | Page count, title, file size, has_forms |
| `compress_pdf` | Compress and write to disk |
| `save_pdf_with_annotations` | Write Highlight / Ink / FreeText objects into PDF |

## pdfops-mcp

A **tokio async** binary implementing the MCP protocol over stdio using the `rmcp` crate.

```
main()
  └─ PdfOpsServer::new().serve(stdio())
        ├─ compress_pdf  ──► pdfops-core::compress
        ├─ get_pdf_info  ──► lopdf
        ├─ merge_pdfs    ──► lopdf
        └─ markdown_to_pdf ► pdfops-core::md_to_typst
```

## Data flow: PDF annotation

```
User draws on SVG overlay (PdfViewer.tsx)
  │
  │  Annotation stored in React state (App.tsx)
  │  { type, page, coordinates as 0–1 ratios, color }
  ▼
User clicks Save → invoke("save_pdf_with_annotations", annotations)
  │
  ▼
Rust: pdf_ops::save_pdf_with_annotations
  │  lopdf::Document::load_from(bytes)
  │  for each annotation:
  │    convert ratio coords → PDF user-space points
  │    build Dictionary { /Type /Annot, /Subtype, /Rect, ... }
  │    doc.add_object(dict)
  │    page /Annots array ← new object id
  └─ doc.save(output_path)
```

Coordinate conversion (canvas → PDF space):

$$
x_{pdf} = x_{ratio} \times W_{page}
$$

$$
y_{pdf} = H_{page} \times (1 - y_{ratio})
$$

PDF origin is **bottom-left**; canvas origin is **top-left**.
