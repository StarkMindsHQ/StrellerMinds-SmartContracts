/*
 * events.js
 *
 * Small wrappers to track pageviews and named events. Designed to be used
 * after consent has been granted. If you prefer server-side forwarding,
 * call your API endpoints and let the server call Measurement Protocol.
 */

import { gtagEvent } from './gtag.js';

export function trackPageView(path, title) {
  if (!path) path = window.location.pathname;
  gtagEvent('page_view', { page_path: path, page_title: title || document.title });
}

export function trackClick(action, category = 'ui', label) {
  gtagEvent('click', { event_category: category, event_action: action, event_label: label });
}

export function trackConversion(name, value, currency = 'USD') {
  gtagEvent('conversion', { event_category: 'conversion', event_action: name, value, currency });
}

export function trackDeposit(amount, currency = 'USD') {
  gtagEvent('deposit', { value: amount, currency });
}
