import { describe, it, expect } from 'vitest';
import {
  extractChartData,
  extractScatterData,
  extractHeatmapData,
  extractRadarData,
  getCellValuesFromRange,
  getNumericValuesFromRange,
} from './chartData';
import type { CellData } from '../types/spreadsheet';

// Helper to create cell data
function createCells(data: Record<string, string>): Record<string, CellData> {
  const cells: Record<string, CellData> = {};
  for (const [key, value] of Object.entries(data)) {
    cells[key] = { value };
  }
  return cells;
}

describe('chartData utilities', () => {
  describe('getCellValuesFromRange', () => {
    it('should extract values from a vertical range', () => {
      const cells = createCells({
        '0:0': 'Apple',
        '1:0': 'Banana',
        '2:0': 'Cherry',
      });
      const values = getCellValuesFromRange('A1:A3', cells);
      expect(values).toEqual(['Apple', 'Banana', 'Cherry']);
    });

    it('should extract values from a horizontal range', () => {
      const cells = createCells({
        '0:0': '10',
        '0:1': '20',
        '0:2': '30',
      });
      const values = getCellValuesFromRange('A1:C1', cells);
      expect(values).toEqual(['10', '20', '30']);
    });

    it('should return empty strings for empty cells', () => {
      const cells = createCells({
        '0:0': 'Apple',
        '2:0': 'Cherry',
      });
      const values = getCellValuesFromRange('A1:A3', cells);
      expect(values).toEqual(['Apple', '', 'Cherry']);
    });
  });

  describe('getNumericValuesFromRange', () => {
    it('should extract numeric values from a range', () => {
      const cells = createCells({
        '0:1': '10',
        '1:1': '20',
        '2:1': '30',
      });
      const values = getNumericValuesFromRange('B1:B3', cells);
      expect(values).toEqual([10, 20, 30]);
    });

    it('should return 0 for non-numeric values', () => {
      const cells = createCells({
        '0:1': '10',
        '1:1': 'text',
        '2:1': '30',
      });
      const values = getNumericValuesFromRange('B1:B3', cells);
      expect(values).toEqual([10, 0, 30]);
    });

    it('should handle decimal numbers', () => {
      const cells = createCells({
        '0:1': '10.5',
        '1:1': '20.25',
      });
      const values = getNumericValuesFromRange('B1:B2', cells);
      expect(values).toEqual([10.5, 20.25]);
    });
  });

  describe('extractChartData', () => {
    it('should extract chart data with labels and values', () => {
      const cells = createCells({
        '0:0': 'Apple',
        '1:0': 'Banana',
        '2:0': 'Cherry',
        '0:1': '10',
        '1:1': '20',
        '2:1': '30',
      });
      const data = extractChartData('A1:A3', 'B1:B3', cells);
      expect(data).toEqual([
        { name: 'Apple', value: 10 },
        { name: 'Banana', value: 20 },
        { name: 'Cherry', value: 30 },
      ]);
    });

    it('should use default labels when labels are empty', () => {
      const cells = createCells({
        '0:1': '10',
        '1:1': '20',
      });
      const data = extractChartData('A1:A2', 'B1:B2', cells);
      expect(data[0].name).toBe('Item 1');
      expect(data[1].name).toBe('Item 2');
    });

    it('should update when cell values change', () => {
      const cells = createCells({
        '0:0': 'Apple',
        '0:1': '10',
      });
      
      // Initial data
      let data = extractChartData('A1:A1', 'B1:B1', cells);
      expect(data[0]).toEqual({ name: 'Apple', value: 10 });
      
      // Update cell value
      cells['0:1'] = { value: '50' };
      data = extractChartData('A1:A1', 'B1:B1', cells);
      expect(data[0]).toEqual({ name: 'Apple', value: 50 });
      
      // Update label
      cells['0:0'] = { value: 'Orange' };
      data = extractChartData('A1:A1', 'B1:B1', cells);
      expect(data[0]).toEqual({ name: 'Orange', value: 50 });
    });
  });

  describe('extractScatterData', () => {
    it('should extract scatter plot data', () => {
      const cells = createCells({
        '0:0': '1',
        '1:0': '2',
        '2:0': '3',
        '0:1': '10',
        '1:1': '20',
        '2:1': '30',
      });
      const data = extractScatterData('A1:A3', 'B1:B3', cells);
      expect(data).toEqual([
        { x: 1, y: 10 },
        { x: 2, y: 20 },
        { x: 3, y: 30 },
      ]);
    });

    it('should handle mismatched range lengths', () => {
      const cells = createCells({
        '0:0': '1',
        '1:0': '2',
        '0:1': '10',
      });
      const data = extractScatterData('A1:A2', 'B1:B1', cells);
      // Should only include pairs where both values exist
      expect(data).toHaveLength(1);
    });
  });

  describe('extractHeatmapData', () => {
    it('should extract heatmap data from a rectangular range', () => {
      const cells = createCells({
        '0:0': '1',
        '0:1': '2',
        '1:0': '3',
        '1:1': '4',
      });
      const data = extractHeatmapData('A1:B2', cells);
      expect(data).toHaveLength(4);
      expect(data).toContainEqual({ x: 'Col 1', y: 'Row 1', value: 1 });
      expect(data).toContainEqual({ x: 'Col 2', y: 'Row 1', value: 2 });
      expect(data).toContainEqual({ x: 'Col 1', y: 'Row 2', value: 3 });
      expect(data).toContainEqual({ x: 'Col 2', y: 'Row 2', value: 4 });
    });

    it('should handle empty cells as 0', () => {
      const cells = createCells({
        '0:0': '1',
        // 0:1 is empty
        '1:0': '3',
        '1:1': '4',
      });
      const data = extractHeatmapData('A1:B2', cells);
      const emptyCell = data.find(d => d.x === 'Col 2' && d.y === 'Row 1');
      expect(emptyCell?.value).toBe(0);
    });
  });

  describe('extractRadarData', () => {
    it('should extract radar chart data (same as chart data)', () => {
      const cells = createCells({
        '0:0': 'Speed',
        '1:0': 'Power',
        '2:0': 'Defense',
        '0:1': '80',
        '1:1': '65',
        '2:1': '90',
      });
      const data = extractRadarData('A1:A3', 'B1:B3', cells);
      expect(data).toEqual([
        { name: 'Speed', value: 80 },
        { name: 'Power', value: 65 },
        { name: 'Defense', value: 90 },
      ]);
    });
  });

  describe('chart data reactivity to cell changes', () => {
    it('should reflect updated cell values immediately', () => {
      const cells = createCells({
        '0:0': 'Q1',
        '1:0': 'Q2',
        '2:0': 'Q3',
        '3:0': 'Q4',
        '0:1': '100',
        '1:1': '150',
        '2:1': '200',
        '3:1': '250',
      });

      // Initial chart data
      let chartData = extractChartData('A1:A4', 'B1:B4', cells);
      expect(chartData[0].value).toBe(100);
      expect(chartData[1].value).toBe(150);

      // Simulate updating Q1 value
      cells['0:1'] = { value: '500' };
      chartData = extractChartData('A1:A4', 'B1:B4', cells);
      expect(chartData[0].value).toBe(500);

      // Simulate updating Q2 label
      cells['1:0'] = { value: 'Second Quarter' };
      chartData = extractChartData('A1:A4', 'B1:B4', cells);
      expect(chartData[1].name).toBe('Second Quarter');
    });

    it('should handle adding new data points', () => {
      const cells = createCells({
        '0:0': 'A',
        '0:1': '10',
      });

      let chartData = extractChartData('A1:A3', 'B1:B3', cells);
      expect(chartData).toHaveLength(3);
      expect(chartData[0]).toEqual({ name: 'A', value: 10 });
      expect(chartData[1].name).toBe('Item 2'); // Empty label

      // Add new data
      cells['1:0'] = { value: 'B' };
      cells['1:1'] = { value: '20' };
      cells['2:0'] = { value: 'C' };
      cells['2:1'] = { value: '30' };

      chartData = extractChartData('A1:A3', 'B1:B3', cells);
      expect(chartData).toEqual([
        { name: 'A', value: 10 },
        { name: 'B', value: 20 },
        { name: 'C', value: 30 },
      ]);
    });

    it('should handle removing data (setting to empty)', () => {
      const cells = createCells({
        '0:0': 'A',
        '1:0': 'B',
        '0:1': '10',
        '1:1': '20',
      });

      let chartData = extractChartData('A1:A2', 'B1:B2', cells);
      expect(chartData[1].value).toBe(20);

      // Remove B's value
      cells['1:1'] = { value: '' };
      chartData = extractChartData('A1:A2', 'B1:B2', cells);
      expect(chartData[1].value).toBe(0);
    });

    it('should handle formula results in data range', () => {
      const cells = createCells({
        '0:0': 'Sum',
        '0:1': '=10+20', // Formula that equals 30
      });

      // Note: The chart data extractor uses getComputedValue which evaluates formulas
      const chartData = extractChartData('A1:A1', 'B1:B1', cells);
      expect(chartData[0].value).toBe(30);
    });
  });
});
