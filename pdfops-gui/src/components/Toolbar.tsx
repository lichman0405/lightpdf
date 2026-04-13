import type { CSSProperties } from "react";
import type { AnnotationMode } from "../types/annotations";

export interface ToolbarProps {
  hasDoc: boolean;
  page: number;
  pageCount: number;
  scale: number;
  annotationMode: AnnotationMode;
  onOpen: () => void;
  onSave: () => void;
  onPageChange: (p: number) => void;
  onScaleChange: (s: number) => void;
  onCompress: () => void;
  onAnnotationModeChange: (m: AnnotationMode) => void;
  onUndoAnnotation: () => void;
  canUndo: boolean;
}

const SCALE_STEPS = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0];

export function Toolbar({
  hasDoc,
  page,
  pageCount,
  scale,
  annotationMode,
  onOpen,
  onSave,
  onPageChange,
  onScaleChange,
  onCompress,
  onAnnotationModeChange,
  onUndoAnnotation,
  canUndo,
}: ToolbarProps) {
  const btnStyle: CSSProperties = {
    padding: "4px 12px",
    cursor: "pointer",
    borderRadius: 4,
    border: "1px solid #555",
    background: "#3a3a3a",
    color: "#eee",
    fontSize: 13,
    marginRight: 4,
  };

  const divider: CSSProperties = {
    width: 1,
    height: 24,
    background: "#555",
    margin: "0 8px",
    alignSelf: "center",
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        padding: "6px 12px",
        background: "#2b2b2b",
        borderBottom: "1px solid #444",
        gap: 4,
        flexShrink: 0,
        overflowX: "auto",
      }}
    >
      {/* File operations */}
      <button style={btnStyle} onClick={onOpen}>
        📂 Open
      </button>
      <button style={{ ...btnStyle, opacity: hasDoc ? 1 : 0.4 }} onClick={onSave} disabled={!hasDoc}>
        💾 Save
      </button>
      <button style={{ ...btnStyle, opacity: hasDoc ? 1 : 0.4 }} onClick={onCompress} disabled={!hasDoc}>
        🗜 Compress
      </button>

      <span style={divider} />

      {/* Navigation */}
      <button
        style={btnStyle}
        onClick={() => onPageChange(page - 1)}
        disabled={page <= 1 || !hasDoc}
      >
        ‹
      </button>
      <input
        type="number"
        min={1}
        max={pageCount || 1}
        value={hasDoc ? page : ""}
        onChange={(e) => {
          const v = parseInt(e.target.value, 10);
          if (!isNaN(v)) onPageChange(v);
        }}
        disabled={!hasDoc}
        style={{
          width: 48,
          textAlign: "center",
          background: "#1e1e1e",
          color: "#eee",
          border: "1px solid #555",
          borderRadius: 4,
          padding: "3px 4px",
          fontSize: 13,
        }}
      />
      <span style={{ color: "#aaa", fontSize: 13 }}>/ {hasDoc ? pageCount : "-"}</span>
      <button
        style={btnStyle}
        onClick={() => onPageChange(page + 1)}
        disabled={page >= pageCount || !hasDoc}
      >
        ›
      </button>

      <span style={divider} />

      {/* Zoom */}
      <select
        value={scale}
        onChange={(e) => onScaleChange(parseFloat(e.target.value))}
        disabled={!hasDoc}
        style={{
          background: "#1e1e1e",
          color: "#eee",
          border: "1px solid #555",
          borderRadius: 4,
          padding: "3px 4px",
          fontSize: 13,
        }}
      >
        {SCALE_STEPS.map((s) => (
          <option key={s} value={s}>
            {Math.round(s * 100)}%
          </option>
        ))}
      </select>

      <span style={divider} />

      {/* Annotation tools */}
      {(["select", "highlight", "draw", "text"] as AnnotationMode[]).map((mode) => {
        const labels: Record<AnnotationMode, string> = {
          select: "↖ Select",
          highlight: "🖊 Highlight",
          draw: "✏ Draw",
          text: "T Text",
        };
        const active = annotationMode === mode;
        return (
          <button
            key={mode}
            style={{
              ...btnStyle,
              background: active ? "#005fb8" : "#3a3a3a",
              borderColor: active ? "#0078d4" : "#555",
            }}
            disabled={!hasDoc}
            onClick={() => onAnnotationModeChange(mode)}
          >
            {labels[mode]}
          </button>
        );
      })}

      <button
        style={{ ...btnStyle, opacity: canUndo ? 1 : 0.4 }}
        disabled={!canUndo}
        onClick={onUndoAnnotation}
        title="Undo last annotation"
      >
        ↩ Undo
      </button>
    </div>
  );
}
