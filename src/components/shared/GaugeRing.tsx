import { usageColor } from "../../utils/format";

interface GaugeRingProps {
  value: number;
  label: string;
  size?: number;
  sublabel?: string;
}

export function GaugeRing({
  value,
  label,
  size = 120,
  sublabel,
}: GaugeRingProps) {
  const stroke = 10;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = Math.min(100, Math.max(0, value));
  const offset = circumference - (clamped / 100) * circumference;
  const color = usageColor(clamped);

  return (
    <div className="gauge-ring" style={{ width: size, height: size }}>
      <svg width={size} height={size}>
        <circle
          className="gauge-ring__track"
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth={stroke}
        />
        <circle
          className="gauge-ring__fill"
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth={stroke}
          stroke={color}
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
      </svg>
      <div className="gauge-ring__center">
        <span className="gauge-ring__value" style={{ color }}>
          {clamped.toFixed(0)}%
        </span>
        <span className="gauge-ring__label">{label}</span>
        {sublabel && <span className="gauge-ring__sublabel">{sublabel}</span>}
      </div>
    </div>
  );
}
