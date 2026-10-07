import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { EMPTY_METRICS, type MetricsSnapshot } from "../types/metrics";

export function useMetrics(historyLength: number = 60) {
  const [metrics, setMetrics] = useState<MetricsSnapshot | null>(null);
  const [connected, setConnected] = useState(false);
  const [history, setHistory] = useState<number[]>([]);
  const [gpuHistory, setGpuHistory] = useState<number[]>([]);
  const [memHistory, setMemHistory] = useState<number[]>([]);
  const [rxHistory, setRxHistory] = useState<number[]>([]);
  const [txHistory, setTxHistory] = useState<number[]>([]);

  useEffect(() => {
    setHistory((prev) => prev.slice(-historyLength));
    setGpuHistory((prev) => prev.slice(-historyLength));
    setMemHistory((prev) => prev.slice(-historyLength));
    setRxHistory((prev) => prev.slice(-historyLength));
    setTxHistory((prev) => prev.slice(-historyLength));
  }, [historyLength]);

  const pushHistory = useCallback((snap: MetricsSnapshot) => {
    const push = (setter: (fn: (prev: number[]) => number[]) => void, val: number) => {
      setter((prev) => [...prev, val].slice(-historyLength));
    };
    push(setHistory, snap.cpu.overall_usage);
    push(setGpuHistory, snap.gpu.usage_percent);
    push(setMemHistory, snap.memory.usage_percent);
    push(setRxHistory, snap.network.total_rx_bytes_per_sec);
    push(setTxHistory, snap.network.total_tx_bytes_per_sec);
  }, [historyLength]);

  useEffect(() => {
    let cancelled = false;

    invoke<MetricsSnapshot>("get_metrics_now")
      .then((snap) => {
        if (!cancelled) {
          setMetrics(snap);
          pushHistory(snap);
          setConnected(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          invoke<MetricsSnapshot>("get_metrics")
            .then((snap) => {
              if (!cancelled) {
                setMetrics(snap);
                pushHistory(snap);
                setConnected(true);
              }
            })
            .catch(() => {
              setMetrics(EMPTY_METRICS);
              setConnected(false);
            });
        }
      });

    const unlisten = listen<MetricsSnapshot>("metrics-update", (event) => {
      setMetrics(event.payload);
      pushHistory(event.payload);
      setConnected(true);
    });

    return () => {
      cancelled = true;
      unlisten.then((fn) => fn());
    };
  }, [pushHistory]);

  return {
    metrics,
    connected,
    history,
    gpuHistory,
    memHistory,
    rxHistory,
    txHistory,
  };
}
