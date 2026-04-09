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

- Both scripts download `hey` (HTTP load testing tool) to `/tmp/hey` on first run
- No AI calls — pure infrastructure benchmarking
