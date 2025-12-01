import { describe, it, expect } from 'vitest';
import type { ChartType, ChartDataRange, ChartPosition, ChartSize } from './chart';
import { 
  generateChartId, 
  DEFAULT_CHART_WIDTH, 
  DEFAULT_CHART_HEIGHT,
  MIN_CHART_WIDTH,
  MIN_CHART_HEIGHT,
  CHART_TYPE_LABELS,
} from './chart';

describe('Chart types and utilities', () => {
  describe('generateChartId', () => {
    it('should generate unique chart IDs', () => {
      const id1 = generateChartId();
      const id2 = generateChartId();
      expect(id1).not.toBe(id2);
    });

    it('should generate IDs with chart prefix', () => {
      const id = generateChartId();
      expect(id.startsWith('chart-')).toBe(true);
    });
  });

  describe('Chart type labels', () => {
    it('should have labels for all chart types', () => {
      const chartTypes: ChartType[] = ['bar', 'line', 'area', 'pie', 'radar', 'scatter', 'heatmap'];
      for (const type of chartTypes) {
        expect(CHART_TYPE_LABELS[type]).toBeDefined();
        expect(typeof CHART_TYPE_LABELS[type]).toBe('string');
      }
    });

    it('should have human-readable labels', () => {
      expect(CHART_TYPE_LABELS.bar).toBe('Bar Chart');
      expect(CHART_TYPE_LABELS.line).toBe('Line Chart');
      expect(CHART_TYPE_LABELS.area).toBe('Area Chart');
      expect(CHART_TYPE_LABELS.pie).toBe('Pie Chart');
      expect(CHART_TYPE_LABELS.radar).toBe('Radar Chart');
      expect(CHART_TYPE_LABELS.scatter).toBe('Scatter Chart');
      expect(CHART_TYPE_LABELS.heatmap).toBe('Heatmap');
    });
  });

  describe('Chart size constants', () => {
    it('should have valid default sizes', () => {
      expect(DEFAULT_CHART_WIDTH).toBeGreaterThan(0);
      expect(DEFAULT_CHART_HEIGHT).toBeGreaterThan(0);
    });

    it('should have valid minimum sizes', () => {
      expect(MIN_CHART_WIDTH).toBeGreaterThan(0);
      expect(MIN_CHART_HEIGHT).toBeGreaterThan(0);
    });

    it('should have default sizes greater than minimum', () => {
      expect(DEFAULT_CHART_WIDTH).toBeGreaterThanOrEqual(MIN_CHART_WIDTH);
      expect(DEFAULT_CHART_HEIGHT).toBeGreaterThanOrEqual(MIN_CHART_HEIGHT);
    });
  });

  describe('ChartDataRange type', () => {
    it('should support basic data range configuration', () => {
      const dataRange: ChartDataRange = {
        labelsRange: 'A1:A10',
        dataRange: 'B1:B10',
      };
      expect(dataRange.labelsRange).toBe('A1:A10');
      expect(dataRange.dataRange).toBe('B1:B10');
    });

    it('should support multi-series configuration', () => {
      const dataRange: ChartDataRange = {
        labelsRange: 'A1:A10',
        dataRange: 'B1:B10',
        seriesRanges: ['C1:C10', 'D1:D10'],
      };
      expect(dataRange.seriesRanges).toHaveLength(2);
    });
  });

  describe('ChartPosition type', () => {
    it('should represent x and y coordinates', () => {
      const position: ChartPosition = { x: 100, y: 200 };
      expect(position.x).toBe(100);
      expect(position.y).toBe(200);
    });
  });

  describe('ChartSize type', () => {
    it('should represent width and height', () => {
      const size: ChartSize = { width: 400, height: 300 };
      expect(size.width).toBe(400);
      expect(size.height).toBe(300);
    });
  });
});
