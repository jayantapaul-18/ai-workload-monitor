interface SparklineProps {
  data: number[];
  color?: string;
  height?: number;
  max?: number;
}

export function Sparkline({
  data,
  color = "var(--accent)",
  height = 48,
  max,
}: SparklineProps) {
  if (data.length < 2) {
    return <div className="sparkline sparkline--empty" style={{ height }} />;
  }

  const peak = max ?? Math.max(...data, 1);
  const width = 200;
  const step = width / (data.length - 1);

  const points = data
    .map((value, index) => {
      const x = index * step;
      const y = height - (value / peak) * (height - 4) - 2;
      return `${x},${y}`;
    })
    .join(" ");

  const areaPoints = `0,${height} ${points} ${width},${height}`;

  return (
    <svg
      className="sparkline"
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      style={{ height }}
    >
      <defs>
        <linearGradient id={`grad-${color}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.35" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <polygon
        points={areaPoints}
        fill={`url(#grad-${color})`}
      />
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth="2"
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}
