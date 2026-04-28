Google Analytics Integration Samples

This folder contains lightweight, privacy-first frontend samples for integrating
Google Analytics 4 (GA4) with the existing server-side Measurement Protocol
components already present under `api/src/analytics`.

Files:
- `web/gtag.js` — async loader for `gtag.js` that waits for explicit user consent.
- `web/consent-manager.js` — minimal cookie-consent banner and consent relay to the API.
- `web/events.js` — small helper to track pageviews and custom events (fires only after consent).

Design goals:
- GDPR-friendly: explicit opt-in, opt-out header `X-Analytics-Consent: denied`, and anonymize IPs.
- No PII collection: avoid sending user identifiers; prefer anonymized IDs or server-side forwarding.
- Minimal performance impact: scripts load asynchronously and only after consent is granted.

Usage:
1. Add the consent banner snippet to your root HTML or app shell.
2. On consent granted, the consent manager posts the preference to `POST /api/v1/analytics/consent` and loads `gtag.js` via `gtag.js` loader.
3. Use `events.js` helpers to instrument pageviews, clicks, deposits, and conversions.
