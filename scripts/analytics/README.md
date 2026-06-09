Analytics aggregator

Usage

From repository root run:

```bash
node scripts/analytics/aggregate_search_events.js
```

What it does

- Reads `api/data/search_events.jsonl` (one JSON object per line)
- Produces `api/data/analytics_summary.json` and `api/data/suggestions.json`
- Computes:
  - popular queries
  - zero-result queries
  - low CTR queries
  - top suggestions per query (by clicks)
  - A/B testing summary (per experiment, per variant)

Notes

- This is a minimal, dependency-free prototype. Replace with a proper pipeline (ClickHouse / Kafka / Airflow) for production.
- The script assumes events include fields `type`, `query`, `resultsCount` (or `results_count`), `contentId` (or `content_id`), `experiment`, and `variant`.
