import { useMemo } from 'preact/hooks';
import {
  BarChart,
  LineChart,
  AreaChart,
  PieChart,
  RadarChart,
  ScatterChart,
} from '@mantine/charts';
import type { ChartData } from '../types/chart';
import type { CellData } from '../types/spreadsheet';
import {
  extractChartData,
  extractScatterData,
  extractHeatmapData,
  type ChartDataPoint,
  type ScatterDataPoint,
  type HeatmapDataPoint,
} from '../utils/chartData';

interface ChartRendererProps {
  chart: ChartData;
  cells: Record<string, CellData>;
  width: number;
  height: number;
}

// Simple heatmap component using recharts
function HeatmapChart({ data, width, height }: { data: HeatmapDataPoint[]; width: number; height: number }) {
  if (data.length === 0) {
    return (
      <div 
        className="flex items-center justify-center text-gray-400"
        style={{ width, height }}
      >
        No data available
      </div>
    );
  }

  // Get unique X and Y values
  const xValues = [...new Set(data.map(d => d.x))];
  const yValues = [...new Set(data.map(d => d.y))];
  
  // Calculate min and max for color scaling
  const values = data.map(d => d.value);
  const minValue = Math.min(...values);
  const maxValue = Math.max(...values);
  const range = maxValue - minValue || 1;
  
  // Calculate cell dimensions
  const cellWidth = (width - 60) / xValues.length;
  const cellHeight = (height - 40) / yValues.length;
  
  // Get color based on value
  const getColor = (value: number) => {
    const normalized = (value - minValue) / range;
    const r = Math.round(59 + normalized * (239 - 59));
    const g = Math.round(130 - normalized * 60);
    const b = Math.round(246 - normalized * 180);
    return `rgb(${r}, ${g}, ${b})`;
  };

  return (
    <div style={{ width, height, overflow: 'hidden' }}>
      <svg width={width} height={height}>
        {/* Y-axis labels */}
        {yValues.map((y, yi) => (
          <text
            key={`y-${yi}`}
            x={55}
            y={yi * cellHeight + cellHeight / 2 + 20}
            textAnchor="end"
            alignmentBaseline="middle"
            fontSize={10}
            fill="#9ca3af"
          >
            {y}
          </text>
        ))}
        
        {/* X-axis labels */}
        {xValues.map((x, xi) => (
          <text
            key={`x-${xi}`}
            x={60 + xi * cellWidth + cellWidth / 2}
            y={height - 5}
            textAnchor="middle"
            fontSize={10}
            fill="#9ca3af"
          >
            {x}
          </text>
        ))}
        
        {/* Cells */}
        {data.map((d, i) => {
          const xi = xValues.indexOf(d.x);
          const yi = yValues.indexOf(d.y);
          return (
            <g key={i}>
              <rect
                x={60 + xi * cellWidth}
                y={yi * cellHeight + 20}
                width={cellWidth - 2}
                height={cellHeight - 2}
                fill={getColor(d.value)}
                rx={2}
              />
              <text
                x={60 + xi * cellWidth + cellWidth / 2}
                y={yi * cellHeight + cellHeight / 2 + 20}
                textAnchor="middle"
                alignmentBaseline="middle"
                fontSize={10}
                fill="#fff"
              >
                {d.value.toFixed(1)}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}

export function ChartRenderer({ chart, cells, width, height }: ChartRendererProps) {
  const chartData = useMemo(() => {
    const { labelsRange, dataRange } = chart.dataRange;
    
    switch (chart.type) {
      case 'scatter':
        // For scatter, labels range is Y values, data range is X values
        return extractScatterData(dataRange, labelsRange, cells);
      case 'heatmap':
        return extractHeatmapData(dataRange, cells);
      default:
        return extractChartData(labelsRange, dataRange, cells);
    }
  }, [chart.type, chart.dataRange.labelsRange, chart.dataRange.dataRange, cells]);

  // Chart dimensions with some padding for title
  const chartWidth = width - 20;
  const chartHeight = height - 40;

  if (chartData.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        No data available
      </div>
    );
  }

  switch (chart.type) {
    case 'bar':
      return (
        <BarChart
          h={chartHeight}
          w={chartWidth}
          data={chartData as ChartDataPoint[]}
          dataKey="name"
          series={[{ name: 'value', color: 'blue.6' }]}
          tickLine="y"
        />
      );

    case 'line':
      return (
        <LineChart
          h={chartHeight}
          w={chartWidth}
          data={chartData as ChartDataPoint[]}
          dataKey="name"
          series={[{ name: 'value', color: 'blue.6' }]}
          curveType="linear"
          tickLine="y"
        />
      );

    case 'area':
      return (
        <AreaChart
          h={chartHeight}
          w={chartWidth}
          data={chartData as ChartDataPoint[]}
          dataKey="name"
          series={[{ name: 'value', color: 'blue.6' }]}
          curveType="linear"
          tickLine="y"
        />
      );

    case 'pie':
      return (
        <PieChart
          h={chartHeight}
          w={chartWidth}
          data={(chartData as ChartDataPoint[]).map(d => ({
            name: d.name,
            value: d.value,
            color: `hsl(${Math.random() * 360}, 70%, 50%)`,
          }))}
          withLabels
          labelsType="value"
        />
      );

    case 'radar':
      return (
        <RadarChart
          h={chartHeight}
          w={chartWidth}
          data={chartData as ChartDataPoint[]}
          dataKey="name"
          series={[{ name: 'value', color: 'blue.6' }]}
        />
      );

    case 'scatter':
      return (
        <ScatterChart
          h={chartHeight}
          w={chartWidth}
          data={(chartData as ScatterDataPoint[]).map(d => ({
            ...d,
            color: 'blue.6',
          }))}
          dataKey={{ x: 'x', y: 'y' }}
        />
      );

    case 'heatmap':
      return (
        <HeatmapChart
          data={chartData as HeatmapDataPoint[]}
          width={chartWidth}
          height={chartHeight}
        />
      );

    default:
      return (
        <div className="flex items-center justify-center h-full text-gray-400">
          Unsupported chart type
        </div>
      );
  }
}
