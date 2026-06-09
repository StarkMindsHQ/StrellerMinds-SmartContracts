# Visual Regression Testing

This suite provides screenshot-based UI regression protection for repository-facing dashboard surfaces.

## Coverage

- Full-page baseline screenshot (`dashboard-shell.png`)
- Component-level screenshot (`component-cards.png`)
- Cross-browser execution on Chromium, Firefox, and WebKit
- HTML diff report output via Playwright

## Run Locally

From repository root:

```bash
npm install
npm run visual:test:update   # update committed baseline screenshots when UI changes are intentional
npm run visual:test
```

To inspect diffs:

```bash
npm run visual:test:report
```

> Note: CI runs visual tests against the committed baselines and does not update snapshots automatically.

## CI Integration

The `CI` workflow runs `visual-regression` with:

- A 10-minute timeout
- Browser installation + screenshot assertions
- Uploaded artifacts for screenshots, diffs, and HTML report
