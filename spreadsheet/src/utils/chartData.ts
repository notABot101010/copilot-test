import type { CellData } from '../types/spreadsheet';
import { parseRange, getComputedValue, type CellData as FormulaCellData } from './formulaEngine';

export interface ChartDataPoint {
  name: string;
  value: number;
  [key: string]: string | number;
}

export interface ScatterDataPoint {
  x: number;
  y: number;
}

export interface HeatmapDataPoint {
  x: string;
  y: string;
  value: number;
}

/**
 * Get raw cell values from a range
 */
export function getCellValuesFromRange(
  range: string,
  cells: Record<string, CellData>
): string[] {
  const parsed = parseRange(range);
  if (!parsed) return [];
  
  const values: string[] = [];
  const minRow = Math.min(parsed.start.row, parsed.end.row);
  const maxRow = Math.max(parsed.start.row, parsed.end.row);
  const minCol = Math.min(parsed.start.col, parsed.end.col);
  const maxCol = Math.max(parsed.start.col, parsed.end.col);
  
  // Determine if it's a row or column range
  if (minRow === maxRow) {
    // Horizontal range
    for (let col = minCol; col <= maxCol; col++) {
      const computed = getComputedValue(minRow, col, cells as Record<string, FormulaCellData>);
      values.push(computed);
    }
  } else {
    // Vertical range or rectangular (use first column)
    for (let row = minRow; row <= maxRow; row++) {
      const computed = getComputedValue(row, minCol, cells as Record<string, FormulaCellData>);
      values.push(computed);
    }
  }
  
  return values;
}

/**
 * Get numeric values from a range
 */
export function getNumericValuesFromRange(
  range: string,
  cells: Record<string, CellData>
): number[] {
  const stringValues = getCellValuesFromRange(range, cells);
  return stringValues.map(v => {
    const num = parseFloat(v);
    return isNaN(num) ? 0 : num;
  });
}

/**
 * Extract data for a simple chart (bar, line, area, pie)
 */
export function extractChartData(
  labelsRange: string,
  dataRange: string,
  cells: Record<string, CellData>
): ChartDataPoint[] {
  const labels = getCellValuesFromRange(labelsRange, cells);
  const values = getNumericValuesFromRange(dataRange, cells);
  
  const data: ChartDataPoint[] = [];
  const length = Math.max(labels.length, values.length);
  
  for (let i = 0; i < length; i++) {
    data.push({
      name: labels[i] || `Item ${i + 1}`,
      value: values[i] || 0,
    });
  }
  
  return data;
}

/**
 * Extract data for a multi-series chart
 */
export function extractMultiSeriesData(
  labelsRange: string,
  seriesRanges: string[],
  cells: Record<string, CellData>
): ChartDataPoint[] {
  const labels = getCellValuesFromRange(labelsRange, cells);
  const seriesData = seriesRanges.map(range => getNumericValuesFromRange(range, cells));
  
  const data: ChartDataPoint[] = [];
  const length = Math.max(labels.length, ...seriesData.map(s => s.length));
  
  for (let i = 0; i < length; i++) {
    const point: ChartDataPoint = {
      name: labels[i] || `Item ${i + 1}`,
      value: seriesData[0]?.[i] || 0,
    };
    
    // Add each series as a named property
    seriesData.forEach((series, idx) => {
      point[`series${idx + 1}`] = series[i] || 0;
    });
    
    data.push(point);
  }
  
  return data;
}

/**
 * Extract data for a scatter chart
 */
export function extractScatterData(
  xRange: string,
  yRange: string,
  cells: Record<string, CellData>
): ScatterDataPoint[] {
  const xValues = getNumericValuesFromRange(xRange, cells);
  const yValues = getNumericValuesFromRange(yRange, cells);
  
  const data: ScatterDataPoint[] = [];
  const length = Math.min(xValues.length, yValues.length);
  
  for (let i = 0; i < length; i++) {
    data.push({
      x: xValues[i],
      y: yValues[i],
    });
  }
  
  return data;
}

/**
 * Extract data for a heatmap
 * Expects a rectangular range where rows are Y, columns are X
 */
export function extractHeatmapData(
  range: string,
  cells: Record<string, CellData>
): HeatmapDataPoint[] {
  const parsed = parseRange(range);
  if (!parsed) return [];
  
  const data: HeatmapDataPoint[] = [];
  const minRow = Math.min(parsed.start.row, parsed.end.row);
  const maxRow = Math.max(parsed.start.row, parsed.end.row);
  const minCol = Math.min(parsed.start.col, parsed.end.col);
  const maxCol = Math.max(parsed.start.col, parsed.end.col);
  
  for (let row = minRow; row <= maxRow; row++) {
    for (let col = minCol; col <= maxCol; col++) {
      const computed = getComputedValue(row, col, cells as Record<string, FormulaCellData>);
      const value = parseFloat(computed);
      data.push({
        x: `Col ${col - minCol + 1}`,
        y: `Row ${row - minRow + 1}`,
        value: isNaN(value) ? 0 : value,
      });
    }
  }
  
  return data;
}

/**
 * Extract radar chart data
 * Each category (row/column) becomes a spoke
 */
export function extractRadarData(
  labelsRange: string,
  dataRange: string,
  cells: Record<string, CellData>
): ChartDataPoint[] {
  // Radar uses same format as simple charts
  return extractChartData(labelsRange, dataRange, cells);
}
