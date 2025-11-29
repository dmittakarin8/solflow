/**
 * Flow Sparkline - Client Component
 * Minimal SVG sparkline showing flow trend across 6 windows
 */

'use client';

interface FlowSparklineProps {
  flows: number[]; // [60s, 300s, 900s, 3600s, 7200s, 14400s]
}

export function FlowSparkline({ flows }: FlowSparklineProps) {
  if (!flows || flows.length === 0) {
    return <div className="w-20 h-8" />;
  }

  const width = 80;
  const height = 32;
  const padding = 2;

  // Normalize values
  const maxAbsValue = Math.max(...flows.map(Math.abs), 1);
  const points = flows.map((value, i) => {
    const x = padding + (i / (flows.length - 1)) * (width - 2 * padding);
    const normalizedValue = value / maxAbsValue;
    const y = height / 2 - (normalizedValue * (height / 2 - padding));
    return `${x},${y}`;
  });

  const pathData = `M ${points.join(' L ')}`;

  // Determine overall trend color
  const lastFlow = flows[flows.length - 1];
  const strokeColor =
    lastFlow > 0
      ? 'rgb(34, 197, 94)' // green
      : lastFlow < 0
      ? 'rgb(239, 68, 68)' // red
      : 'rgb(156, 163, 175)'; // gray

  return (
    <svg
      width={width}
      height={height}
      className="inline-block"
      viewBox={`0 0 ${width} ${height}`}
    >
      {/* Zero line */}
      <line
        x1={padding}
        y1={height / 2}
        x2={width - padding}
        y2={height / 2}
        stroke="currentColor"
        strokeWidth="1"
        strokeOpacity="0.1"
      />

      {/* Flow line */}
      <polyline
        points={points.join(' ')}
        fill="none"
        stroke={strokeColor}
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />

      {/* Dots at data points */}
      {points.map((point, i) => {
        const [x, y] = point.split(',').map(Number);
        return (
          <circle
            key={i}
            cx={x}
            cy={y}
            r="2"
            fill={strokeColor}
          />
        );
      })}
    </svg>
  );
}
