/*
 * gtag.js
 *
 * Simple GA4 loader that inserts the gtag script asynchronously and
 * configures `anonymize_ip` to true. The loader is a no-op until
 * `initGtag(measurementId)` is called after user consent.
 */

const state = {
  initialized: false,
};

export function initGtag(measurementId) {
  if (typeof window === "undefined") return;
  if (!measurementId) return;
  if (state.initialized) return;

  // Create dataLayer if missing
  window.dataLayer = window.dataLayer || [];
  function gtag(){window.dataLayer.push(arguments);} // eslint-disable-line no-inner-declarations
  window.gtag = gtag;

  // Load gtag.js asynchronously with minimal impact
  const script = document.createElement("script");
  script.async = true;
  script.src = `https://www.googletagmanager.com/gtag/js?id=${encodeURIComponent(measurementId)}`;
  script.referrerPolicy = "no-referrer-when-downgrade";
  script.onload = () => {
    // Configure gtag: anonymize IP and disable personalized ads
    gtag('js', new Date());
    gtag('config', measurementId, {
      anonymize_ip: true,
      allow_ad_personalization_signals: false,
    });
    state.initialized = true;
  };

  // Append as late as possible to avoid blocking critical rendering
  document.head.appendChild(script);
}

export function gtagEvent(name, params = {}) {
  if (!state.initialized || typeof window === "undefined" || !window.gtag) return;
  try {
    window.gtag('event', name, params);
  } catch (e) {
    // No-op: analytics must not crash the app
    // eslint-disable-next-line no-console
    console.warn('gtag event failed', e);
  }
}
