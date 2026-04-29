#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const DATA_DIR = path.join(process.cwd(), 'api', 'data');
const INPUT_FILE = path.join(DATA_DIR, 'search_events.jsonl');
const OUT_SUMMARY = path.join(DATA_DIR, 'analytics_summary.json');
const OUT_SUGGESTIONS = path.join(DATA_DIR, 'suggestions.json');

function loadEvents() {
  if (!fs.existsSync(INPUT_FILE)) {
    console.error('No events file found at', INPUT_FILE);
    return [];
  }
  const lines = fs.readFileSync(INPUT_FILE, 'utf8').trim().split('\n');
  const events = [];
  for (const l of lines) {
    if (!l) continue;
    try {
      events.push(JSON.parse(l));
    } catch (err) {
      console.warn('Skipping malformed line');
    }
  }
  return events;
}

function aggregate(events) {
  const queryCounts = new Map();
  const zeroResults = new Map();
  const clicksByQueryContent = new Map();
  const searchesByQuery = new Map();

  const experimentStats = {}; // experiment -> variant -> {searches, clicks}

  for (const ev of events) {
    const q = String(ev.query || '');
    // search events
    if (ev.type === 'search') {
      queryCounts.set(q, (queryCounts.get(q) || 0) + 1);
      searchesByQuery.set(q, (searchesByQuery.get(q) || 0) + 1);
      if (ev.resultsCount === 0 || ev.results_count === 0) {
        zeroResults.set(q, (zeroResults.get(q) || 0) + 1);
      }

      if (ev.experiment) {
        experimentStats[ev.experiment] = experimentStats[ev.experiment] || {};
        const v = ev.variant || 'unknown';
        experimentStats[ev.experiment][v] = experimentStats[ev.experiment][v] || { searches: 0, clicks: 0 };
        experimentStats[ev.experiment][v].searches += 1;
      }
    }

    // click events
    if (ev.type === 'click') {
      const contentId = String(ev.contentId || ev.content_id || ev.contentID || '');
      const key = q + '||' + contentId;
      clicksByQueryContent.set(key, (clicksByQueryContent.get(key) || 0) + 1);

      if (ev.experiment) {
        experimentStats[ev.experiment] = experimentStats[ev.experiment] || {};
        const v = ev.variant || 'unknown';
        experimentStats[ev.experiment][v] = experimentStats[ev.experiment][v] || { searches: 0, clicks: 0 };
        experimentStats[ev.experiment][v].clicks += 1;
      }
    }
  }

  // compute popular queries
  const popular = Array.from(queryCounts.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 200)
    .map(([query, count]) => ({ query, count }));

  // zero-result top
  const zeroTop = Array.from(zeroResults.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 200)
    .map(([query, count]) => ({ query, count }));

  // compute CTR per query-content and top suggestions
  const suggestions = {};
  const ctrs = [];
  const grouped = {};
  for (const [key, clicks] of clicksByQueryContent.entries()) {
    const [query, contentId] = key.split('||');
    const searches = searchesByQuery.get(query) || 0;
    const ctr = searches === 0 ? 0 : clicks / searches;
    ctrs.push({ query, contentId, clicks, searches, ctr });
    grouped[query] = grouped[query] || [];
    grouped[query].push({ contentId, clicks, ctr });
  }

  for (const q of Object.keys(grouped)) {
    suggestions[q] = grouped[q].sort((a, b) => b.clicks - a.clicks).slice(0, 5).map(s => s.contentId);
  }

  // low CTR queries
  const lowCtr = ctrs
    .filter(c => c.searches >= 5) // require minimum searches
    .sort((a, b) => a.ctr - b.ctr)
    .slice(0, 200)
    .map(c => ({ query: c.query, contentId: c.contentId, ctr: c.ctr, searches: c.searches }));

  // experiment analysis: compute CTR per variant
  const ab = {};
  for (const [exp, variants] of Object.entries(experimentStats)) {
    ab[exp] = {};
    for (const [v, st] of Object.entries(variants)) {
      const searches = st.searches || 0;
      const clicks = st.clicks || 0;
      const ctr = searches === 0 ? 0 : clicks / searches;
      ab[exp][v] = { searches, clicks, ctr };
    }
  }

  return { popular, zeroTop, lowCtr, suggestions, ab };
}

function writeOut(summary, suggestions) {
  if (!fs.existsSync(DATA_DIR)) fs.mkdirSync(DATA_DIR, { recursive: true });
  fs.writeFileSync(OUT_SUMMARY, JSON.stringify(summary, null, 2));
  fs.writeFileSync(OUT_SUGGESTIONS, JSON.stringify(suggestions, null, 2));
  console.log('Wrote', OUT_SUMMARY, OUT_SUGGESTIONS);
}

function main() {
  const events = loadEvents();
  const agg = aggregate(events);
  writeOut({ generated_at: new Date().toISOString(), stats: agg }, agg.suggestions);
}

if (require.main === module) main();
