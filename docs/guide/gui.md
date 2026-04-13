# Desktop App

![LightPDF interface](/screenshot.png)

## Opening a PDF

Three ways to open a file:

- Click **📂 Open** in the toolbar → native file picker
- **Drag & drop** a `.pdf` file onto the window
- The app remembers the last file path (status bar shows filename)

## Navigating pages

| Method | Action |
|--------|--------|
| `PageDown` or `→` | Next page |
| `PageUp` or `←` | Previous page |
| Toolbar **‹ ›** buttons | Previous / next page |
| Toolbar page input | Type a page number and press Enter |

## Zooming

| Method | Action |
|--------|--------|
| `Ctrl + Scroll wheel` | Zoom in / out (step 10%) |
| Toolbar dropdown | Choose a preset (25 % – 400 %) |

Rendering uses `devicePixelRatio` scaling so text and images stay sharp on HiDPI displays at every zoom level.

## Annotation tools

Select a mode from the toolbar before interacting with the PDF:

### ↖ Select (default)
No drawing — scroll and click freely.

### 🖊 Highlight
Click and drag a rectangle over text to place a semi-transparent yellow highlight.

- Release the mouse to commit.
- Small movements (< 0.5 % of page) are ignored to avoid accidental marks.

### ✏ Draw
Click and drag to draw freehand ink strokes in red.

- Lift the mouse button to commit the stroke.
- Multiple strokes can be drawn before saving.

### T Text
Click anywhere on the page to place a text annotation.

- A floating input appears at the click position.
- Type your text and press **Enter** to confirm, or **Escape** to cancel.
- The text is also committed if you click elsewhere (blur).

### ↩ Undo
Removes the last committed annotation. Can be pressed repeatedly.

## Saving

Click **💾 Save** to write all annotations back to the original file.

Annotations are saved as standard PDF objects:

| Annotation type | PDF object type |
|-----------------|----------------|
| Highlight | `/Highlight` annotation with `QuadPoints` |
| Draw | `/Ink` annotation with `InkList` |
| Text | `/FreeText` annotation with `Contents` |

These are compatible with Adobe Acrobat, Preview, Foxit, and other PDF viewers.

## Compressing

Click **🗜 Compress** to apply zlib content-stream compression to the currently open file. The status bar shows:

```
Compressed: 4.2 MB → 1.8 MB (−57.1%)
```

The app automatically reloads the compressed file.

## Status bar

The bar at the bottom shows:
- The name of the currently open file
- Results of the last Save / Compress operation
- Error messages if an operation fails
