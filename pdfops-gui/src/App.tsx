import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { PdfViewer } from "./components/PdfViewer";
import { Toolbar } from "./components/Toolbar";
import type { Annotation, AnnotationMode } from "./types/annotations";
import "./App.css";

interface CompressResult {
  output_path: string;
  original_bytes: number;
  compressed_bytes: number;
}

export default function App() {
  const [pdfData, setPdfData] = useState<Uint8Array | null>(null);
  const [pdfPath, setPdfPath] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [pageCount, setPageCount] = useState(0);
  const [scale, setScale] = useState(1.0);
  const [statusMsg, setStatusMsg] = useState<string>("");
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [annotationMode, setAnnotationMode] = useState<AnnotationMode>("select");

  const loadPdf = useCallback(async (filePath: string) => {
    try {
      const bytes: number[] = await invoke("read_pdf_bytes", { path: filePath });
      setPdfData(new Uint8Array(bytes));
      setPdfPath(filePath);
      setPage(1);
      setStatusMsg(filePath.split(/[\\/]/).pop() ?? filePath);
    } catch (e) {
      setStatusMsg(`Error loading PDF: ${e}`);
    }
  }, []);

  const handleOpen = useCallback(async () => {
    const path: string | null = await invoke("open_pdf_dialog");
    if (path) await loadPdf(path);
  }, [loadPdf]);

  const handleSave = useCallback(async () => {
    if (!pdfPath || !pdfData) return;
    try {
      await invoke("save_pdf_with_annotations", {
        inputPath: pdfPath,
        outputPath: pdfPath,
        annotations: annotations,
      });
      setStatusMsg("Saved.");
    } catch (e) {
      setStatusMsg(`Save error: ${e}`);
    }
  }, [pdfPath, pdfData, annotations]);

  const handleCompress = useCallback(async () => {
    if (!pdfPath) return;
    try {
      const result: CompressResult = await invoke("compress_pdf", { inputPath: pdfPath, level: 6 });
      const ratio = ((1 - result.compressed_bytes / result.original_bytes) * 100).toFixed(1);
      setStatusMsg(
        `Compressed: ${(result.original_bytes / 1024).toFixed(1)} KB → ${(result.compressed_bytes / 1024).toFixed(1)} KB (−${ratio}%)`
      );
      // Reload the compressed file
      await loadPdf(result.output_path);
    } catch (e) {
      setStatusMsg(`Compress error: ${e}`);
    }
  }, [pdfPath, loadPdf]);

  // Drag-and-drop PDF files onto the window
  const handleDrop = useCallback(
    async (e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      const file = e.dataTransfer.files[0];
      if (file && file.name.toLowerCase().endsWith(".pdf")) {
        // Tauri 2: file.path is available via the webview
        const path = (file as File & { path?: string }).path ?? "";
        if (path) await loadPdf(path);
      }
    },
    [loadPdf]
  );

  const clampPage = (n: number) => setPage(Math.max(1, Math.min(n, pageCount)));

  return (
    <div
      style={{ display: "flex", flexDirection: "column", height: "100vh", background: "#1e1e1e" }}
      onDrop={handleDrop}
      onDragOver={(e) => e.preventDefault()}
    >
      <Toolbar
        hasDoc={pdfData !== null}
        page={page}
        pageCount={pageCount}
        scale={scale}
        annotationMode={annotationMode}
        onOpen={handleOpen}
        onSave={handleSave}
        onPageChange={clampPage}
        onScaleChange={setScale}
        onCompress={handleCompress}
        onAnnotationModeChange={setAnnotationMode}
        onUndoAnnotation={() => setAnnotations((a) => a.slice(0, -1))}
        canUndo={annotations.length > 0}
      />

      <PdfViewer
        data={pdfData}
        page={page}
        scale={scale}
        onPageCount={setPageCount}
        onScaleChange={setScale}
        annotations={annotations}
        annotationMode={annotationMode}
        onAnnotationAdd={(a) => setAnnotations((prev) => [...prev, a])}
      />

      {/* Status bar */}
      <div
        style={{
          padding: "3px 12px",
          background: "#333",
          color: "#aaa",
          fontSize: 12,
          flexShrink: 0,
          borderTop: "1px solid #444",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {statusMsg || "Ready"}
      </div>
    </div>
  );
}
