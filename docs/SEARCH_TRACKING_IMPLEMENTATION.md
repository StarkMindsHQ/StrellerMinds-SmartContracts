Search Tracking — Implementation Notes

Scope
- Track search events and clicks on-chain for immutable audit; emit rich off-chain events for aggregation.

What changed
- Per-event storage keys now include timestamp so events are not overwritten (search & click events).
- Off-chain consumers should aggregate events to compute popular queries, zero-result queries, CTR, and session funnels.

Next steps (implementation)
1. Update off-chain collector to ingest `search` and `click` events and store in analytics DB (Postgres/ClickHouse).
2. Compute popular searches: aggregate counts, weekly trends, and spikes.
3. Identify gaps: surface zero-result queries and low-CTR queries for content creation.
4. Suggest improvements: surface suggested content and query rewrites using top-N suggestions from analytics + lightweight ML.
5. A/B test: create feature flag in search payload to annotate events with experiment group; measure CTR and conversion.

Acceptance criteria
- Events include query, user (if available), results_count, rank_position for clicks.
- Aggregation produces accurate popular-queries and zero-result lists within acceptable latency.
- Suggestions are A/B testable and show measurable CTR uplift in experiment window.
- On-chain changes add minimal compute/storage; heavy aggregation happens off-chain.

Performance notes
- Keep on-chain storage minimal (store raw events only; avoid expensive aggregates on-chain).
- Emit rich events for off-chain processing rather than computing heavy metrics in-contract.

Deployment
- Implement off-chain consumer and dashboard before enabling suggestion rollout.
- Run A/B tests with small user percentage; monitor latency and success metrics.
