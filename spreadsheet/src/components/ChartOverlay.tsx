import { useState, useRef, useCallback } from 'preact/hooks';
import { ActionIcon, Text } from '@mantine/core';
import type { ChartData } from '../types/chart';
import { MIN_CHART_WIDTH, MIN_CHART_HEIGHT } from '../types/chart';
import type { CellData } from '../types/spreadsheet';
import { ChartRenderer } from './ChartRenderer';
import { updateChartPosition, updateChartSize, deleteChart } from '../store/spreadsheetStore';

interface ChartOverlayProps {
  chart: ChartData;
  cells: Record<string, CellData>;
  scrollLeft: number;
  scrollTop: number;
  containerOffsetX: number;
  containerOffsetY: number;
}

export function ChartOverlay({
  chart,
  cells,
  scrollLeft,
  scrollTop,
  containerOffsetX,
  containerOffsetY,
}: ChartOverlayProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0, chartX: 0, chartY: 0 });
  const resizeStartRef = useRef({ x: 0, y: 0, width: 0, height: 0 });
  const chartRef = useRef<HTMLDivElement>(null);

  // Handle drag start
  const handleDragStart = useCallback((e: MouseEvent) => {
    if ((e.target as HTMLElement).closest('.resize-handle')) return;
    if ((e.target as HTMLElement).closest('.chart-actions')) return;
    
    e.preventDefault();
    e.stopPropagation();
    
    setIsDragging(true);
    dragStartRef.current = {
      x: e.clientX,
      y: e.clientY,
      chartX: chart.position.x,
      chartY: chart.position.y,
    };

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - dragStartRef.current.x;
      const deltaY = moveEvent.clientY - dragStartRef.current.y;
      
      const newX = Math.max(0, dragStartRef.current.chartX + deltaX);
      const newY = Math.max(0, dragStartRef.current.chartY + deltaY);
      
      updateChartPosition(chart.id, { x: newX, y: newY });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [chart.id, chart.position.x, chart.position.y]);

  // Handle resize start
  const handleResizeStart = useCallback((e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    setIsResizing(true);
    resizeStartRef.current = {
      x: e.clientX,
      y: e.clientY,
      width: chart.size.width,
      height: chart.size.height,
    };

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - resizeStartRef.current.x;
      const deltaY = moveEvent.clientY - resizeStartRef.current.y;
      
      const newWidth = Math.max(MIN_CHART_WIDTH, resizeStartRef.current.width + deltaX);
      const newHeight = Math.max(MIN_CHART_HEIGHT, resizeStartRef.current.height + deltaY);
      
      updateChartSize(chart.id, { width: newWidth, height: newHeight });
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [chart.id, chart.size.width, chart.size.height]);

  // Handle delete
  const handleDelete = useCallback((e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    deleteChart(chart.id);
  }, [chart.id]);

  // Calculate position accounting for scroll
  const left = chart.position.x - scrollLeft + containerOffsetX;
  const top = chart.position.y - scrollTop + containerOffsetY;

  // Don't render if chart is completely outside visible area
  if (
    left + chart.size.width < 0 ||
    top + chart.size.height < 0
  ) {
    return null;
  }

  return (
    <div
      ref={chartRef}
      className={`
        absolute bg-gray-800 rounded-lg shadow-xl border border-gray-600
        ${isDragging ? 'cursor-grabbing opacity-90' : 'cursor-grab'}
        ${isResizing ? 'opacity-90' : ''}
      `}
      style={{
        left,
        top,
        width: chart.size.width,
        height: chart.size.height,
        zIndex: isDragging || isResizing ? 1000 : 100,
      }}
      onMouseDown={handleDragStart}
    >
      {/* Title bar */}
      <div className="flex items-center justify-between px-3 py-2 bg-gray-700 rounded-t-lg border-b border-gray-600">
        <Text size="sm" fw={500} className="text-white truncate flex-1">
          {chart.title}
        </Text>
        <div className="chart-actions flex gap-1 ml-2">
          <ActionIcon
            variant="subtle"
            size="sm"
            color="red"
            onClick={handleDelete}
            title="Delete chart"
          >
            <span className="text-xs">×</span>
          </ActionIcon>
        </div>
      </div>

      {/* Chart content */}
      <div className="p-2" style={{ height: chart.size.height - 40 }}>
        <ChartRenderer
          chart={chart}
          cells={cells}
          width={chart.size.width - 20}
          height={chart.size.height - 60}
        />
      </div>

      {/* Resize handle */}
      <div
        className="resize-handle absolute bottom-0 right-0 w-4 h-4 cursor-se-resize"
        onMouseDown={handleResizeStart}
        style={{
          background: 'linear-gradient(135deg, transparent 50%, rgba(255,255,255,0.3) 50%)',
        }}
      />
    </div>
  );
}
