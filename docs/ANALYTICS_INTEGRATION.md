**Google Analytics Integration**

This document explains how to integrate GA4 with this codebase while preserving user privacy, minimizing performance impact, and providing verification steps for real-time dashboards.

**Key points**
- Use the server-side Measurement Protocol client at `api/src/analytics/ga4Client.ts` for backend events.
- Frontend should only enable client-side `gtag.js` after explicit user consent.
- IPs are anonymized (`anonymize_ip: true`) and `non_personalized_ads` is enabled to reduce profiling risk.
- Do not send PII to GA (emails, names, raw IPs, addresses, etc.). Use anonymized IDs when necessary.

1) Frontend integration (web)

- Drop the files from `analytics/web/` into your web app build (or import them as modules):
  - `gtag.js` — async loader that sets `anonymize_ip: true`.
  - `consent-manager.js` — minimal consent banner which posts preference to `POST /api/v1/analytics/consent` and sets a cookie.
  - `events.js` — helpers: `trackPageView`, `trackClick`, `trackConversion`, `trackDeposit`.

- Recommendation: expose your GA Measurement ID to the client only after obtaining a consent signal from the user, for example by rendering `window.__GA_MEASUREMENT_ID__` server-side only when consent is saved.

Example usage in an HTML shell:

```html
<script>
  // optional: server-rendered value, otherwise pass via init
  window.__GA_MEASUREMENT_ID__ = 'G-XXXXXXXXXX';
</script>
<script type="module">
  import { mountConsentBanner, getConsentFromCookie } from '/analytics/web/consent-manager.js';
  import { initGtag } from '/analytics/web/gtag.js';
  import { trackPageView } from '/analytics/web/events.js';

  const consent = getConsentFromCookie();
  if (!consent) {
    mountConsentBanner(); // shows banner and loads gtag on accept
  } else if (consent === 'granted') {
    initGtag(window.__GA_MEASUREMENT_ID__);
    trackPageView();
  }
</script>
```

2) Server-side forwarding

- Use the existing Measurement Protocol client (`api/src/analytics/ga4Client.ts`) to send backend events — it already:
  - respects `config.analytics.enabled`
  - uses `non_personalized_ads: true`
  - provides `anonymizeClientId()` to protect identities

- The server also exposes `POST/GET/DELETE /api/v1/analytics/consent` in `api/src/routes/consent.ts` to store user preferences. Client code should call this endpoint after the user makes a choice.

3) GDPR & Privacy

- Consent: show an explicit consent UI before loading client-side analytics. The `consent-manager.js` sample sends the preference to the server and sets a cookie.
- IP Anonymization: the frontend `gtag` loader sets `anonymize_ip: true`. Backend Measurement Protocol does not send raw IPs; it relies on anonymized `client_id` and `non_personalized_ads`.
- Opt-out: clients can send header `X-Analytics-Consent: denied` with API calls to suppress server-side events. The middleware `api/src/middleware/analyticsConsent.ts` reads this header and sets `req.analyticsOptOut`.

4) Instrumentation checklist (implement these events)

- Page views: `trackPageView()` on route changes.
- Clicks (important CTA): `trackClick(action, category, label)`.
- Deposits / Transactions: `trackDeposit(amount, currency)` and server-side record via Measurement Protocol.
- Conversions (sign-ups, feature opt-ins): `trackConversion(name, value)` and duplicate server-side events for reliability.

5) Performance best practices

- Load `gtag.js` asynchronously and only after consent (see `gtag.js`).
- For SPA frameworks, lazy-load the analytics bundle on first meaningful interaction.
- Use server-side forwarding for critical conversion events to avoid client-side blocking and ad-blocker loss.

6) Verification

- To verify events appear in real-time: open GA4 Admin → DebugView and use the browser with the `gtag` loaded. For Measurement Protocol debug requests, enable `GA4_DEBUG=true` in server env to log debug responses.
- Use `api/src/analytics/__tests__` for unit-testing the Measurement Protocol client.

7) Next steps (optional)

- Add automated tests that exercise `consent.ts` endpoints.
- Add E2E test flows that grant consent and verify server's `req.analyticsOptOut` behavior.
