/*
 * consent-manager.js
 *
 * Minimal cookie-consent banner that asks for opt-in before loading analytics.
 * On grant it POSTs to the server consent endpoint (if available) and loads gtag.
 * Designed to be framework-agnostic — drop into a simple HTML shell.
 */

import { initGtag } from './gtag.js';

const DEFAULT_MEASUREMENT_ID = window.__GA_MEASUREMENT_ID__ || '';

export function mountConsentBanner(opts = {}) {
  const measurementId = opts.measurementId || DEFAULT_MEASUREMENT_ID;
  if (typeof document === 'undefined') return;

  const banner = document.createElement('div');
  banner.setAttribute('id', 'consent-banner');
  banner.style.position = 'fixed';
  banner.style.left = '0';
  banner.style.right = '0';
  banner.style.bottom = '0';
  banner.style.background = '#fff';
  banner.style.borderTop = '1px solid #ddd';
  banner.style.padding = '12px';
  banner.style.zIndex = '9999';
  banner.style.display = 'flex';
  banner.style.justifyContent = 'space-between';
  banner.style.alignItems = 'center';

  banner.innerHTML = `
    <div style="flex:1; margin-right:12px">We use anonymous analytics to improve the app. Do you consent to anonymous tracking?</div>
    <div>
      <button id="consent-deny" style="margin-right:8px">Deny</button>
      <button id="consent-grant">Accept</button>
    </div>
  `;

  document.body.appendChild(banner);

  document.getElementById('consent-deny').addEventListener('click', () => {
    savePreference('denied');
    removeBanner();
  });

  document.getElementById('consent-grant').addEventListener('click', () => {
    savePreference('granted');
    // initialize client-side gtag if measurementId provided
    if (measurementId) initGtag(measurementId);
    removeBanner();
  });

  function removeBanner() {
    if (banner && banner.parentNode) banner.parentNode.removeChild(banner);
  }

  async function savePreference(value) {
    // Persist to server if endpoint available. If it fails, we silently continue.
    try {
      await fetch('/api/v1/analytics/consent', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ consent: value }),
      });
    } catch (e) {
      // swallow error — do not disrupt the app
    }
    // Also set a client cookie for quick local checks (expires 1 year)
    document.cookie = `analytics_consent=${value};path=/;max-age=${60*60*24*365};samesite=lax`;
  }
}

export function getConsentFromCookie() {
  if (typeof document === 'undefined') return null;
  const m = document.cookie.match(/(?:^|; )analytics_consent=([^;]+)/);
  return m ? decodeURIComponent(m[1]) : null;
}
