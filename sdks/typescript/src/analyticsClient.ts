export interface SearchEvent {
    type: "search" | "click";
    query: string;
    userId?: string;
    resultsCount?: number;
    contentId?: string;
    rankPosition?: number;
    experiment?: string; // experiment key
    variant?: string; // experiment variant
    timestamp?: string;
}

/**
 * Send a search-related event to the API ingestion endpoint.
 * Lightweight and resilient: fire-and-forget in browser.
 */
export async function trackSearchEvent(apiBase: string, event: SearchEvent): Promise<void> {
    try {
        const url = new URL("/api/v1/analytics/events", apiBase).toString();

        const payload = Object.assign({}, event, { timestamp: new Date().toISOString() });

        // Fire-and-forget: we await only to catch early errors during development.
        await fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
            keepalive: true,
        });
    } catch (err) {
        // Swallow errors in production; optionally log to console in dev
        if ((globalThis as any).process?.env?.NODE_ENV !== "production") {
            // eslint-disable-next-line no-console
            console.warn("trackSearchEvent failed", err);
        }
    }
}

export default { trackSearchEvent };
