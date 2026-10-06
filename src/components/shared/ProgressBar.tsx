import { usageColor } from "../../utils/format";

interface ProgressBarProps {
  value: number;
  label?: string;
  showValue?: boolean;
  height?: number;
}

export function ProgressBar({
  value,
  label,
  showValue = true,
  height = 8,
}: ProgressBarProps) {
  const clamped = Math.min(100, Math.max(0, value));

  return (
    <div className="progress-bar">
      {(label || showValue) && (
        <div className="progress-bar__header">
          {label && <span className="progress-bar__label">{label}</span>}
          {showValue && (
            <span className="progress-bar__value" style={{ color: usageColor(clamped) }}>
              {clamped.toFixed(0)}%
            </span>
          )}
        </div>
      )}
      <div className="progress-bar__track" style={{ height }}>
        <div
          className="progress-bar__fill"
          style={{
            width: `${clamped}%`,
            background: usageColor(clamped),
          }}
        />
      </div>
    </div>
  );
}
