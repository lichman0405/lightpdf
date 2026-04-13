# MCP Server

`pdfops-mcp` implements the [Model Context Protocol](https://modelcontextprotocol.io) so that AI assistants can invoke PDF operations as tools.

## Running the server

```bash
cargo run -p pdfops-mcp
# or, after building:
./target/release/pdfops-mcp
```

The server communicates over **stdin / stdout** using the MCP stdio transport.

## Connecting to Claude Desktop

Edit your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "lightpdf": {
      "command": "/absolute/path/to/pdfops-mcp"
    }
  }
}
```

Restart Claude Desktop. You should see the LightPDF tools available in the tool panel.

## Available tools

### `get_pdf_info`

Return metadata about a PDF file.

**Input**
```json
{ "path": "/path/to/file.pdf" }
```

**Output**
```json
{
  "file_name": "paper.pdf",
  "title": "My Paper",
  "page_count": 21,
  "file_size_bytes": 1048576
}
```

---

### `compress_pdf`

Compress a PDF using zlib content-stream compression.

**Input**
```json
{
  "input_path": "/path/to/input.pdf",
  "output_path": "/path/to/output.pdf"
}
```

**Output**
```json
{
  "original_bytes": 4200000,
  "compressed_bytes": 1800000,
  "ratio": 0.571
}
```

---

### `merge_pdfs`

Merge two or more PDFs into a single file.

**Input**
```json
{
  "input_paths": ["/a.pdf", "/b.pdf", "/c.pdf"],
  "output_path": "/merged.pdf"
}
```

**Output**
```json
{ "output_path": "/merged.pdf", "page_count": 42 }
```

---

### `markdown_to_pdf`

Convert a Markdown document (with optional LaTeX math) to a Typst source file, and optionally compile it to PDF.

**Input**
```json
{
  "markdown": "# Hello\n\nSome $x^2$ math.",
  "title": "My Doc",
  "output_path": "/out/doc.typ"
}
```

**Output**
```json
{ "output_path": "/out/doc.typ", "compiled": false }
```

> **Note**: PDF compilation requires building with the `typst-engine` feature flag. Without it, the tool writes the `.typ` source file only.

```bash
cargo build -p pdfops-mcp --features pdfops-core/typst-engine
```

## Example prompts for Claude

Once connected, you can ask Claude things like:

> "Compress all PDFs in my Downloads folder and tell me how much space was saved."

> "Merge my lecture slides and the homework sheet into one PDF."

> "Convert this Markdown note to a PDF: [paste markdown]"

> "How many pages does my thesis have and what is its title?"
