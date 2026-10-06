import { usageBg, usageColor } from "../../utils/format";
import type { CoreMetrics } from "../../types/metrics";

interface CoreBoxProps {
  core: CoreMetrics;
}

export function CoreBox({ core }: CoreBoxProps) {
  const usage = core.usage;
  const color = usageColor(usage);
  const bg = usageBg(usage);

  return (
    <div className="core-box" style={{ background: bg, borderColor: color }}>
      <div className="core-box__id">C{core.id}</div>
      <div className="core-box__usage" style={{ color }}>
        {usage.toFixed(0)}%
      </div>
      <div className="core-box__bar">
        <div
          className="core-box__fill"
          style={{ height: `${Math.max(4, usage)}%`, background: color }}
        />
      </div>
      {core.frequency_mhz && (
        <div className="core-box__freq">{core.frequency_mhz.toFixed(0)} MHz</div>
      )}
    </div>
  );
}
