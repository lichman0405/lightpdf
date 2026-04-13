import { useEffect, useRef, useState } from "react";
import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";
import type { Annotation, AnnotationMode, DrawAnnotation, HighlightAnnotation, TextAnnotation } from "../types/annotations";

// PDF.js is loaded via a CDN worker; we need to tell it where the worker is.
import * as pdfjsLib from "pdfjs-dist";
pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url
).toString();

export interface PdfViewerProps {
  /** Raw PDF bytes to display */
  data: Uint8Array | null;
  /** Current page (1-based) */
  page: number;
  /** Zoom level: 0.25 – 4.0, default 1.0 */
  scale: number;
  onPageCount?: (n: number) => void;
  onScaleChange?: (s: number) => void;
  /** Current annotations (all pages) */
  annotations?: Annotation[];
  /** Active annotation drawing mode */
  annotationMode?: AnnotationMode;
  /** Called when user finishes drawing a new annotation */
  onAnnotationAdd?: (a: Annotation) => void;
}

export function PdfViewer({
  data, page, scale, onPageCount, onScaleChange,
  annotations = [], annotationMode = "select", onAnnotationAdd,
}: PdfViewerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const docRef = useRef<PDFDocumentProxy | null>(null);
  const renderTaskRef = useRef<ReturnType<PDFPageProxy["render"]> | null>(null);
  // keep scale and callback in refs so the wheel handler always sees the latest value
  const scaleRef = useRef(scale);
  const onScaleChangeRef = useRef(onScaleChange);
  useEffect(() => { scaleRef.current = scale; }, [scale]);
  useEffect(() => { onScaleChangeRef.current = onScaleChange; }, [onScaleChange]);

  // Ctrl+wheel zoom — must use non-passive listener to call preventDefault
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const handler = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      const step = e.deltaY < 0 ? 0.1 : -0.1;
      const next = Math.min(4, Math.max(0.25, Math.round((scaleRef.current + step) * 100) / 100));
      onScaleChangeRef.current?.(next);
    };
    el.addEventListener("wheel", handler, { passive: false });
    return () => el.removeEventListener("wheel", handler);
  }, []);

  // Load document whenever data changes
  useEffect(() => {
    if (!data) return;

    let cancelled = false;
    (async () => {
      // Cancel any ongoing render
      if (renderTaskRef.current) {
        renderTaskRef.current.cancel();
      }

      const loadingTask = pdfjsLib.getDocument({ data: data.slice(0) });
      const newDoc = await loadingTask.promise;
      if (cancelled) {
        newDoc.destroy();
        return;
      }
      docRef.current?.destroy();
      docRef.current = newDoc;
      onPageCount?.(newDoc.numPages);
    })().catch(console.error);

    return () => {
      cancelled = true;
    };
  }, [data]);

  // Render the requested page whenever page/scale changes
  useEffect(() => {
    const doc = docRef.current;
    if (!doc || !canvasRef.current) return;
    if (page < 1 || page > doc.numPages) return;

    let cancelled = false;
    (async () => {
      if (renderTaskRef.current) {
        renderTaskRef.current.cancel();
      }

      const pdfPage = await doc.getPage(page);
      if (cancelled) return;

      // Multiply by devicePixelRatio so the canvas has physical pixels,
      // then shrink it back with CSS — eliminates blur on HiDPI displays.
      const dpr = window.devicePixelRatio || 1;
      const viewport = pdfPage.getViewport({ scale: scale * dpr });
      const canvas = canvasRef.current!;
      const ctx = canvas.getContext("2d")!;

      canvas.width = viewport.width;
      canvas.height = viewport.height;
      canvas.style.width  = `${viewport.width  / dpr}px`;
      canvas.style.height = `${viewport.height / dpr}px`;

      const task = pdfPage.render({ canvasContext: ctx, viewport });
      renderTaskRef.current = task;
      try {
        await task.promise;
      } catch (e: unknown) {
        // RenderingCancelled is expected on rapid page switches
        if ((e as { name?: string }).name !== "RenderingCancelledException") {
          console.error("Render error:", e);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [data, page, scale]);

  // ── Annotation drawing state ───────────────────────────────────────────────
  const [hlDraft, setHlDraft] = useState<{ x: number; y: number; w: number; h: number } | null>(null);
  const [drawPoints, setDrawPoints] = useState<[number, number][]>([]);
  const isDrawingRef = useRef(false);
  const [textPos, setTextPos] = useState<{ x: number; y: number } | null>(null);
  const [textInput, setTextInput] = useState("");
  const svgRef = useRef<SVGSVGElement>(null);

  const getRatio = (e: React.MouseEvent<SVGSVGElement>): [number, number] => {
    const svg = svgRef.current!;
    const rect = svg.getBoundingClientRect();
    return [
      (e.clientX - rect.left) / rect.width,
      (e.clientY - rect.top) / rect.height,
    ];
  };

  const newId = () => `${Date.now()}-${Math.random().toString(36).slice(2)}`;

  const onSvgMouseDown = (e: React.MouseEvent<SVGSVGElement>) => {
    if (!data) return;
    const [rx, ry] = getRatio(e);
    if (annotationMode === "highlight") {
      setHlDraft({ x: rx, y: ry, w: 0, h: 0 });
    } else if (annotationMode === "draw") {
      isDrawingRef.current = true;
      setDrawPoints([[rx, ry]]);
    } else if (annotationMode === "text") {
      setTextPos({ x: rx, y: ry });
      setTextInput("");
    }
  };

  const onSvgMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const [rx, ry] = getRatio(e);
    if (annotationMode === "highlight" && hlDraft) {
      setHlDraft((d) => d && { ...d, w: rx - d.x, h: ry - d.y });
    } else if (annotationMode === "draw" && isDrawingRef.current) {
      setDrawPoints((pts) => [...pts, [rx, ry]]);
    }
  };

  const onSvgMouseUp = () => {
    if (annotationMode === "highlight" && hlDraft) {
      const x = hlDraft.w < 0 ? hlDraft.x + hlDraft.w : hlDraft.x;
      const y = hlDraft.h < 0 ? hlDraft.y + hlDraft.h : hlDraft.y;
      const w = Math.abs(hlDraft.w);
      const h = Math.abs(hlDraft.h);
      if (w > 0.005 && h > 0.005) {
        onAnnotationAdd?.({ id: newId(), type: "highlight", page, x, y, w, h, color: "#ffff00" } satisfies HighlightAnnotation);
      }
      setHlDraft(null);
    } else if (annotationMode === "draw" && isDrawingRef.current) {
      isDrawingRef.current = false;
      if (drawPoints.length > 1) {
        onAnnotationAdd?.({ id: newId(), type: "draw", page, points: drawPoints, color: "#e74c3c", lineWidth: 2 } satisfies DrawAnnotation);
      }
      setDrawPoints([]);
    }
  };

  const commitText = () => {
    if (textPos && textInput.trim()) {
      onAnnotationAdd?.({ id: newId(), type: "text", page, x: textPos.x, y: textPos.y, content: textInput.trim(), color: "#2c3e50" } satisfies TextAnnotation);
    }
    setTextPos(null);
    setTextInput("");
  };

  const ratiosToPath = (pts: [number, number][]) =>
    pts.map(([x, y], i) => `${i === 0 ? "M" : "L"} ${x * 100} ${y * 100}`).join(" ");

  const pageAnnotations = annotations.filter((a) => a.page === page);

  const modeCursor: Record<AnnotationMode, string> = {
    select: "default",
    highlight: "crosshair",
    draw: "crosshair",
    text: "text",
  };

  return (
    <div
      ref={containerRef}
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "flex-start",
        overflow: "auto",
        flex: 1,
        background: "#525659",
        padding: "16px",
      }}
    >
      {data ? (
        <div style={{ position: "relative", display: "inline-block" }}>
          <canvas
            ref={canvasRef}
            style={{ boxShadow: "0 4px 20px rgba(0,0,0,0.5)", display: "block" }}
          />
          {/* SVG annotation overlay */}
          <svg
            ref={svgRef}
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              height: "100%",
              cursor: modeCursor[annotationMode],
              pointerEvents: annotationMode === "select" ? "none" : "all",
            }}
            onMouseDown={onSvgMouseDown}
            onMouseMove={onSvgMouseMove}
            onMouseUp={onSvgMouseUp}
            onMouseLeave={onSvgMouseUp}
          >
            {/* Committed annotations for this page */}
            {pageAnnotations.map((ann) => {
              if (ann.type === "highlight") {
                return (
                  <rect
                    key={ann.id}
                    x={ann.x * 100}
                    y={ann.y * 100}
                    width={ann.w * 100}
                    height={ann.h * 100}
                    fill={ann.color}
                    fillOpacity={0.35}
                    stroke={ann.color}
                    strokeWidth={0.3}
                  />
                );
              }
              if (ann.type === "draw") {
                return (
                  <path
                    key={ann.id}
                    d={ratiosToPath(ann.points)}
                    fill="none"
                    stroke={ann.color}
                    strokeWidth={ann.lineWidth * 0.3}
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                );
              }
              if (ann.type === "text") {
                return (
                  <text
                    key={ann.id}
                    x={ann.x * 100}
                    y={ann.y * 100}
                    fill={ann.color}
                    fontSize={3}
                    style={{ userSelect: "none" }}
                  >
                    {ann.content}
                  </text>
                );
              }
              return null;
            })}

            {/* Live highlight draft */}
            {hlDraft && (
              <rect
                x={(hlDraft.w < 0 ? hlDraft.x + hlDraft.w : hlDraft.x) * 100}
                y={(hlDraft.h < 0 ? hlDraft.y + hlDraft.h : hlDraft.y) * 100}
                width={Math.abs(hlDraft.w) * 100}
                height={Math.abs(hlDraft.h) * 100}
                fill="#ffff00"
                fillOpacity={0.35}
                stroke="#ffff00"
                strokeWidth={0.3}
              />
            )}

            {/* Live draw draft */}
            {drawPoints.length > 1 && (
              <path
                d={ratiosToPath(drawPoints)}
                fill="none"
                stroke="#e74c3c"
                strokeWidth={0.6}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            )}
          </svg>

          {/* Floating text input for text annotations */}
          {textPos && (
            <input
              autoFocus
              value={textInput}
              onChange={(e) => setTextInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitText();
                if (e.key === "Escape") { setTextPos(null); setTextInput(""); }
              }}
              onBlur={commitText}
              style={{
                position: "absolute",
                left: `${textPos.x * 100}%`,
                top: `${textPos.y * 100}%`,
                transform: "translateY(-100%)",
                background: "rgba(255,255,255,0.9)",
                border: "1px solid #aaa",
                borderRadius: 3,
                padding: "2px 6px",
                fontSize: 14,
                color: "#2c3e50",
                outline: "none",
                minWidth: 120,
                zIndex: 10,
              }}
            />
          )}
        </div>
      ) : (
        <div
          style={{
            color: "#ccc",
            fontSize: 18,
            marginTop: 80,
            textAlign: "center",
          }}
        >
          <p>Drop a PDF here or click&nbsp;<b>Open</b></p>
        </div>
      )}
    </div>
  );
}
