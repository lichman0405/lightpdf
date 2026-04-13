# Annotation System

## Type definitions

All annotation data is defined in `pdfops-gui/src/types/annotations.ts` on the frontend, and mirrored as `AnnotationData` in `pdf_ops.rs` on the Rust backend.

### TypeScript

```typescript
type AnnotationMode = "select" | "highlight" | "draw" | "text";

interface HighlightAnnotation {
  id: string;
  type: "highlight";
  page: number;
  x: number;  // top-left x, ratio 0–1
  y: number;  // top-left y, ratio 0–1
  w: number;  // width ratio
  h: number;  // height ratio
  color: string; // CSS hex e.g. "#ffff00"
}

interface DrawAnnotation {
  id: string;
  type: "draw";
  page: number;
  points: [number, number][]; // [x_ratio, y_ratio] sequence
  color: string;
  lineWidth: number;
}

interface TextAnnotation {
  id: string;
  type: "text";
  page: number;
  x: number;  // anchor x ratio
  y: number;  // anchor y ratio
  content: string;
  color: string;
}

type Annotation = HighlightAnnotation | DrawAnnotation | TextAnnotation;
```

All coordinates are **canvas-relative ratios** (0.0–1.0), where `(0,0)` is the top-left corner of the rendered page. This makes annotations resolution-independent and scale-independent.

### Rust (serde mirror)

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AnnotationData {
    Highlight { id, page, x, y, w, h, color },
    Draw      { id, page, points, color, line_width },
    Text      { id, page, x, y, content, color },
}
```

The `#[serde(tag = "type")]` discriminant matches the TypeScript `type` field.

---

## Frontend rendering (SVG overlay)

An absolutely-positioned `<svg>` element sits on top of the PDF canvas:

```
<div style="position: relative">
  <canvas ref={canvasRef} />          ← PDF.js renders here
  <svg viewBox="0 0 100 100"          ← annotation overlay
       preserveAspectRatio="none"
       style="position: absolute; inset: 0">
    <!-- committed annotations -->
    <!-- live draft while drawing -->
  </svg>
  <!-- floating text input (text mode only) -->
</div>
```

The SVG uses a `viewBox="0 0 100 100"` coordinate space, so all annotation coordinates are multiplied by 100 for SVG rendering (e.g., `x_ratio * 100 = percentage of width`).

`preserveAspectRatio="none"` ensures the SVG stretches exactly over the canvas regardless of aspect ratio.

### Pointer events

| Mode | SVG `pointerEvents` | Cursor |
|------|---------------------|--------|
| `select` | `none` (pass-through) | `default` |
| `highlight` | `all` | `crosshair` |
| `draw` | `all` | `crosshair` |
| `text` | `all` | `text` |

---

## Drawing interaction

### Highlight
```
mousedown → record start (x, y)
mousemove → update draft rect (w = current_x - start_x, h = current_y - start_y)
mouseup   → normalise negative w/h → commit if size > 0.5% of page
```

### Draw
```
mousedown → isDrawing = true, initialise points[]
mousemove → push current (x, y) to points[]
mouseup   → isDrawing = false, commit if points.length > 1
```

### Text
```
mousedown → record (x, y), show <input> at position
Enter/blur → commit TextAnnotation, hide input
Escape     → cancel, hide input
```

A `committingRef` flag prevents the `onBlur` handler from double-firing when Enter is used (Enter → commit → input unmounts → blur fires after commit).

---

## Backend: writing annotations to PDF

`save_pdf_with_annotations` in `pdf_ops.rs`:

1. Load PDF from disk into `lopdf::Document`
2. Get the page ID map: `BTreeMap<u32, ObjectId>` from `doc.get_pages()`
3. For each annotation:
   - Look up the page's `MediaBox` to get `(page_width, page_height)` in PDF points
   - Convert ratio coordinates to PDF user space (origin bottom-left):

     ```
     pdf_x = ratio_x * page_width
     pdf_y = page_height * (1 - ratio_y)
     ```

   - Build a `lopdf::Dictionary` with the appropriate `/Subtype`, `/Rect`, and subtype-specific keys
   - Add the dictionary as an indirect object: `annot_id = doc.add_object(dict)`
   - Append `annot_id` to the page's `/Annots` array
4. `doc.save(output_path)`

### PDF object mapping

| TypeScript type | PDF `/Subtype` | Key fields |
|----------------|----------------|------------|
| `highlight` | `/Highlight` | `/Rect`, `/QuadPoints`, `/C`, `/CA` |
| `draw` | `/Ink` | `/Rect`, `/InkList`, `/C`, `/BS` |
| `text` | `/FreeText` | `/Rect`, `/Contents`, `/C`, `/DA` |

### Coordinate note

lopdf 0.40 uses `f32` for `Object::Real`. All internal calculations use `f64` and are cast to `f32` at the point of constructing PDF objects via the helper:

```rust
fn r(v: f64) -> lopdf::Object { lopdf::Object::Real(v as f32) }
```
