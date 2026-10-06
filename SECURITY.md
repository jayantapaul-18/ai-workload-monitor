# Security

AI Workload Monitor reads local hardware and process information and, when enabled, calls the Ollama and vLLM URLs in your settings. Those calls stay on the hosts you configure. The app does not send metrics to a remote service.

Set those URLs only to servers you trust. A metrics endpoint can reveal which models are loaded.

## Reporting

Please report a vulnerability privately through GitHub Security Advisories for this repository once it is published. Do not file a public issue for an exploitable problem.

There is no bug bounty.
