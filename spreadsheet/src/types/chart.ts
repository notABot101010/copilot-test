export type ChartType = 
  | 'bar'
  | 'line'
  | 'area'
  | 'pie'
  | 'radar'
  | 'scatter'
  | 'heatmap';

export interface ChartPosition {
  x: number;
  y: number;
}

export interface ChartSize {
  width: number;
  height: number;
}

export interface ChartDataRange {
  // Range in A1:B10 format for labels column (optional for some chart types)
  labelsRange: string;
  // Range in A1:B10 format for data values
  dataRange: string;
  // For multi-series charts
  seriesRanges?: string[];
}

export interface ChartData {
  id: string;
  type: ChartType;
  title: string;
  position: ChartPosition;
  size: ChartSize;
  dataRange: ChartDataRange;
  createdAt: number;
  updatedAt: number;
}

// Default chart sizes
export const DEFAULT_CHART_WIDTH = 400;
export const DEFAULT_CHART_HEIGHT = 300;

// Minimum chart sizes
export const MIN_CHART_WIDTH = 200;
export const MIN_CHART_HEIGHT = 150;

// Chart type labels for UI
export const CHART_TYPE_LABELS: Record<ChartType, string> = {
  bar: 'Bar Chart',
  line: 'Line Chart',
  area: 'Area Chart',
  pie: 'Pie Chart',
  radar: 'Radar Chart',
  scatter: 'Scatter Chart',
  heatmap: 'Heatmap',
};

export function generateChartId(): string {
  return `chart-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
}
