---
layout: home

hero:
  name: LightPDF
  text: Fast. Lightweight. Open.
  tagline: A PDF toolkit for humans — desktop app and AI-ready MCP server, powered by Rust.
  image:
    src: /screenshot.png
    alt: LightPDF desktop app
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/lichman0405/lightpdf

features:
  - icon: 📄
    title: Crisp PDF Viewer
    details: Renders PDFs via PDF.js with full HiDPI / Retina support. No blur at any zoom level.
  - icon: 🖊
    title: Annotation Tools
    details: Highlight, freehand draw, and text annotations — saved as standard PDF objects readable by any viewer.
  - icon: 🗜
    title: PDF Compression
    details: Shrink PDFs with zlib content-stream compression via lopdf, without quality loss.
  - icon: 🤖
    title: MCP Server
    details: Expose PDF operations (compress, merge, info, Markdown→PDF) to Claude Desktop or any MCP-compatible AI.
  - icon: ⌨️
    title: Keyboard-First
    details: PageUp/PageDown, arrow keys for navigation, Ctrl+scroll to zoom — no mouse required.
  - icon: 🦀
    title: Rust Core
    details: Pure-Rust library shared between the GUI and MCP server. Fast, safe, no runtime dependencies.
---
