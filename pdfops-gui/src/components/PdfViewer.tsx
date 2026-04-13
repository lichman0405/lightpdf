import { useEffect, useRef, useState } from "react";
import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";

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
}

export function PdfViewer({ data, page, scale, onPageCount }: PdfViewerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const docRef = useRef<PDFDocumentProxy | null>(null);
  const renderTaskRef = useRef<ReturnType<PDFPageProxy["render"]> | null>(null);

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

      const viewport = pdfPage.getViewport({ scale });
      const canvas = canvasRef.current!;
      const ctx = canvas.getContext("2d")!;

      canvas.width = viewport.width;
      canvas.height = viewport.height;

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
        <canvas
          ref={canvasRef}
          style={{
            boxShadow: "0 4px 20px rgba(0,0,0,0.5)",
            display: "block",
          }}
        />
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
