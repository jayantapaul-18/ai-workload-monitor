import type { NetworkMetrics } from "../../types/metrics";
import { formatBytes, formatBytesPerSec } from "../../utils/format";
import { Sparkline } from "../shared/Sparkline";
import { StatCard } from "../shared/StatCard";

interface NetworkPanelProps {
  network: NetworkMetrics;
  rxHistory: number[];
  txHistory: number[];
}

export function NetworkPanel({ network, rxHistory, txHistory }: NetworkPanelProps) {
  return (
    <div className="panel network-panel">
      <div className="panel__header">
        <h2>Network</h2>
        <p>Interface throughput, WiFi status, and link details</p>
      </div>

      <div className="stat-grid stat-grid--3">
        <StatCard
          title="Download"
          value={formatBytesPerSec(network.total_rx_bytes_per_sec)}
          accent="var(--cyan)"
        >
          <Sparkline data={rxHistory} color="var(--cyan)" height={40} />
        </StatCard>
        <StatCard
          title="Upload"
          value={formatBytesPerSec(network.total_tx_bytes_per_sec)}
          accent="var(--warning)"
        >
          <Sparkline data={txHistory} color="var(--warning)" height={40} />
        </StatCard>
        {network.wifi && (
          <StatCard
            title="WiFi"
            value={network.wifi.connected ? network.wifi.ssid || "Connected" : "Disconnected"}
            subtitle={
              network.wifi.signal_dbm
                ? `${network.wifi.signal_dbm} dBm · ${network.wifi.frequency_mhz ?? "?"} MHz`
                : network.wifi.interface
            }
            accent="var(--purple)"
          />
        )}
      </div>

      {network.wifi && (
        <div className="wifi-card">
          <div className="wifi-card__header">
            <span className="wifi-card__icon">📶</span>
            <div>
              <h3>{network.wifi.ssid || "Not connected"}</h3>
              <p>{network.wifi.interface} · {network.wifi.bitrate_mbps ?? "?"} Mbps</p>
            </div>
            {network.wifi.signal_percent != null && (
              <div className="wifi-signal">
                <div
                  className="wifi-signal__bar"
                  style={{ width: `${network.wifi.signal_percent}%` }}
                />
                <span>{network.wifi.signal_percent}%</span>
              </div>
            )}
          </div>
        </div>
      )}

      <div className="interface-list">
        {network.interfaces.map((iface) => (
          <div key={iface.name} className="interface-card">
            <div className="interface-card__header">
              <div>
                <span className={`iface-kind iface-kind--${iface.kind}`}>{iface.kind}</span>
                <strong>{iface.name}</strong>
                <span className={`iface-state iface-state--${iface.state}`}>{iface.state}</span>
              </div>
              <div className="interface-card__rates">
                <span className="rate-down">↓ {formatBytesPerSec(iface.rx_bytes_per_sec)}</span>
                <span className="rate-up">↑ {formatBytesPerSec(iface.tx_bytes_per_sec)}</span>
              </div>
            </div>
            <div className="interface-card__details">
              <span>MAC {iface.mac}</span>
              <span>MTU {iface.mtu}</span>
              {iface.speed_mbps && <span>{iface.speed_mbps} Mbps</span>}
              {iface.ipv4.length > 0 && <span>{iface.ipv4.join(", ")}</span>}
              <span>Total ↓ {formatBytes(iface.rx_total_bytes)} · ↑ {formatBytes(iface.tx_total_bytes)}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
