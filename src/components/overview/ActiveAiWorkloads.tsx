import type { AiRuntimeMetrics } from "../../types/metrics";
import { formatBytes } from "../../utils/format";

interface ActiveAiWorkloadsProps {
  aiRuntimes: AiRuntimeMetrics;
}

function runtimeLabel(runtime: string): string {
  if (runtime === "ollama") return "Ollama";
  if (runtime === "vllm") return "vLLM";
  return runtime;
}

function statusClass(status: string): string {
  if (status === "running") return "ai-workload-card__status--running";
  if (status === "loaded") return "ai-workload-card__status--loaded";
  return "ai-workload-card__status--idle";
}

export function ActiveAiWorkloads({ aiRuntimes }: ActiveAiWorkloadsProps) {
  const { workloads, bottleneck, runtimes_online } = aiRuntimes;
  const hasRuntimes = runtimes_online.length > 0;
  const hasWorkloads = workloads.length > 0;

  return (
    <section className="ai-intelligence">
      <div className="ai-intelligence__header">
        <div>
          <h3>Active AI Workloads</h3>
          <p>
            {hasRuntimes
              ? `Connected: ${runtimes_online.map(runtimeLabel).join(" · ")}`
              : "Connect Ollama or vLLM to see model-level metrics"}
          </p>
        </div>
        {hasWorkloads && (
          <span className="ai-intelligence__count">{workloads.length} model{workloads.length !== 1 ? "s" : ""}</span>
        )}
      </div>

      {bottleneck && (
        <div className={`bottleneck-banner bottleneck-banner--${bottleneck.severity}`}>
          <div className="bottleneck-banner__title">Bottleneck: {bottleneck.message}</div>
          <div className="bottleneck-banner__hint">{bottleneck.suggestion}</div>
        </div>
      )}

      {hasWorkloads ? (
        <div className="ai-workload-grid">
          {workloads.map((workload) => (
            <article key={`${workload.runtime}-${workload.model}`} className="ai-workload-card">
              <div className="ai-workload-card__top">
                <span className={`ai-workload-card__runtime ai-workload-card__runtime--${workload.runtime}`}>
                  {runtimeLabel(workload.runtime)}
                </span>
                <span className={`ai-workload-card__status ${statusClass(workload.status)}`}>
                  {workload.status}
                </span>
              </div>
              <h4 className="ai-workload-card__model">{workload.model}</h4>
              <dl className="ai-workload-card__meta">
                <div>
                  <dt>Accelerator</dt>
                  <dd>{workload.accelerator}</dd>
                </div>
                <div>
                  <dt>Throughput</dt>
                  <dd>
                    {workload.tokens_per_sec != null && workload.tokens_per_sec > 0
                      ? `${workload.tokens_per_sec.toFixed(1)} tok/s${workload.throughput_kind === "estimated" ? " ~" : ""}`
                      : workload.throughput_kind === "active"
                        ? "Generating…"
                        : workload.status === "loaded"
                          ? "Idle (loaded)"
                          : "—"}
                  </dd>
                </div>
                <div>
                  <dt>{workload.memory_on_gpu ? "VRAM" : "Model RAM"}</dt>
                  <dd>{workload.vram_bytes ? formatBytes(workload.vram_bytes) : "—"}</dd>
                </div>
                <div>
                  <dt>Context</dt>
                  <dd>
                    {workload.context_length != null
                      ? workload.context_length.toLocaleString()
                      : "—"}
                  </dd>
                </div>
                <div>
                  <dt>Params</dt>
                  <dd>
                    {workload.parameter_size ?? "—"}
                    {workload.quantization ? ` · ${workload.quantization}` : ""}
                  </dd>
                </div>
              </dl>
            </article>
          ))}
        </div>
      ) : (
        <div className="ai-intelligence__empty">
          <p>No loaded models detected.</p>
          <p className="ai-intelligence__empty-hint">
            Start Ollama (<code>ollama serve</code>) or vLLM, then load a model to see live workload cards here.
          </p>
        </div>
      )}
    </section>
  );
}
