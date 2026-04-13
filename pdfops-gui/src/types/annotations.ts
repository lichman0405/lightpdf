/** All coordinates are canvas-relative ratios (0–1). */

export type AnnotationMode = "select" | "highlight" | "draw" | "text";

export interface HighlightAnnotation {
  id: string;
  type: "highlight";
  page: number;
  /** top-left x, ratio of canvas width */
  x: number;
  /** top-left y, ratio of canvas height */
  y: number;
  w: number;
  h: number;
  color: string;
}

export interface DrawAnnotation {
  id: string;
  type: "draw";
  page: number;
  /** Sequence of [x_ratio, y_ratio] */
  points: [number, number][];
  color: string;
  lineWidth: number;
}

export interface TextAnnotation {
  id: string;
  type: "text";
  page: number;
  x: number;
  y: number;
  content: string;
  color: string;
}

export type Annotation = HighlightAnnotation | DrawAnnotation | TextAnnotation;
