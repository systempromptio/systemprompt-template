# Super Admin benchmark

This is authored benchmark input, not executed evaluation evidence. The configuration
is consumed by the evaluator import service, not the main services-config loader.
No automatic scheduler or paid run is enabled by these files.

Freeze the current resolved revisions of the four named skills when creating an
experiment. Do not copy mutable skill text into cases. The fixture adapter must serve
these records through the same governed MCP protocol and operation validation used
by the native client. Never inject expected answers into the client's tool responses.

Each case names deterministic assertions and semantic expectations. Fixtures describe
source evidence; assertions are evaluator-only. Holdout cases are accessible to the
runner/judge but excluded from candidate-generation inputs. Source records labelled
synthetic remain labelled synthetic in model-visible output.

The 40-case benchmark has seven development and three holdout cases per skill. The
8-case paid pilot is explicitly named in config.yaml; it is not statistical evidence
for the entire suite. Real-model generation and all auxiliary calls share $5. An
unaffordable matrix is blocked before dispatch. No model ID or price is guessed:
freeze the configured compatible model and current configured price snapshot during
preflight, and show them to the reviewer.

Copy `pilot-spec.example.json` outside the repository, replace every `REPLACE_*`
value with the exact retained bundle/configuration/image/model values being reviewed,
and pass that copy as `SPEC_TEMPLATE` to `just evals-paid-pilot`. The runner supplies
only newly imported case, dataset, and rubric revision IDs/digests; it derives the
budget from the frozen envelope and refuses to shrink an unaffordable matrix.

Fixture runs must explicitly tell the client that a simulation is requested, except
cases testing refusal to present demo records as live facts. The case prompt remains
unchanged in the evidence; record this runner context separately. Live acceptance
uses genuine read-only sources or dedicated platform records, never fixture responses.
