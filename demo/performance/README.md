<div align="center">
  <a href="https://systemprompt.io">
    <img src="https://systemprompt.io/logo.svg" alt="systemprompt.io" width="150" />
  </a>
  <p><strong>Production infrastructure for AI agents</strong></p>
  <p><a href="https://systemprompt.io">systemprompt.io</a> • <a href="https://systemprompt.io/documentation">Documentation</a> • <a href="https://github.com/systempromptio/systemprompt-core">Core</a> • <a href="https://github.com/systempromptio/systemprompt-template">Template</a></p>
</div>

---

# Performance & Load Demos

Request tracing, benchmarks, and load testing.

## Prerequisites

Run `../00-preflight.sh` first. Run some governance demos first for trace data.

## Scripts

| # | Script | What it proves | Cost |
|---|--------|---------------|------|
| 01 | request-tracing.sh | Typed data, IDs, logs, flow maps, 200-request benchmark | Free |
| 02 | load-test.sh | 2000-request load test with throughput and latency benchmarks | Free |

## Notes

- Both scripts download `hey` (HTTP load testing tool) to `/tmp/hey` on first run, picking the `hey_darwin_amd64` or `hey_linux_amd64` build for the host. On Apple Silicon the amd64 build runs under Rosetta 2 (`softwareupdate --install-rosetta`); alternatively `brew install hey`.
- No AI calls — pure infrastructure benchmarking


---

## License

MIT - See [LICENSE](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE) for details.
