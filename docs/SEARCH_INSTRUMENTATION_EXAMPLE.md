Frontend Search Instrumentation — Example

Use the TypeScript SDK helper to send events to the API ingestion endpoint.

Example (browser):

```ts
import { trackSearchEvent } from "@strellerminds/sdk";

const API_BASE = process.env.API_BASE || "https://api.strellerminds.com";

async function onSearch(query: string) {
  // perform search request to search backend (omitted)
  const resultsCount = 5; // example

  // Track the search event
  trackSearchEvent(API_BASE, {
    type: "search",
    query,
    resultsCount,
    experiment: "search_suggestions_v2",
    variant: "control",
  });
}

function onResultClick(query: string, contentId: string, rank: number) {
  trackSearchEvent(API_BASE, {
    type: "click",
    query,
    contentId,
    rankPosition: rank,
    experiment: "search_suggestions_v2",
    variant: "control",
  });
}
```

A/B testing notes
- Include `experiment` and `variant` fields in the event payload.
- Ensure the frontend assigns variants deterministically (via feature flag service or hashed user id).

Privacy
- Avoid sending PII. Use pseudonymous user IDs or omit `userId` when not permitted.

Next steps
- Hook an off-chain consumer to `api/data/search_events.jsonl` (or replace with a real ingestion pipeline) to compute popular queries, zero-result lists, CTR, and suggestions.
