# Google Analytics 4 (GA4) Integration

## Overview

The StrellerMinds Certificate Verification API includes a GDPR-compliant Google Analytics 4 integration powered by the [Measurement Protocol](https://developers.google.com/analytics/devguides/collection/protocol/ga4). This enables tracking of user behavior and key conversion events while maintaining strict privacy standards.

## Features

### Event Tracking

All key user actions are tracked as GA4 events:

| Event Name | Trigger | Conversion | Description |
|------------|---------|------------|-------------|
| `auth_token_issued` | JWT token issued | ✅ Yes | Authentication/login conversion |
| `certificate_verified` | Certificate verification | ✅ Yes | Primary verification conversion |
| `certificate_detail_fetched` | Full cert details viewed | ❌ No | Page view equivalent |
| `revocation_checked` | Revocation record lookup | ❌ No | Verification detail |
| `student_certs_listed` | Student certs retrieved | ❌ No | Navigation flow |
| `analytics_queried` | Analytics endpoint accessed | ❌ No | Dashboard usage |
| `api_rate_limited` | Rate limit exceeded | ❌ No | Abuse detection |

### GDPR Compliance

1. **Cookie Consent**: `X-Analytics-Consent` header controls tracking per-request
   - `"granted"` or absent → tracking enabled (default)
   - `"denied"` → tracking suppressed

2. **Data Anonymization**: All client IDs are SHA-256 hashed (32-char hex)
   - No PII is ever sent to GA4
   - IP addresses are hashed for public endpoints
   - JWT `sub` claims are hashed for authenticated endpoints

3. **Opt-Out API**: REST endpoints for managing consent
   - `POST /api/v1/analytics/consent` – Set preference
   - `GET /api/v1/analytics/consent` – Retrieve preference
   - `DELETE /api/v1/analytics/consent` – Withdraw consent

4. **Non-Personalized Ads**: `non_personalized_ads=true` by default

### Performance Optimization

- **Fire-and-Forget**: GA4 requests are never awaited
- **5-Second Timeout**: Aborts slow requests to prevent blocking
- **Graceful Degradation**: GA4 outages cannot affect API availability
- **Async Spawn**: Uses `void (async () => { ... })()` pattern

## Configuration

### Environment Variables (.env)

```bash
# GA4 Measurement ID (format: G-XXXXXXXXXX)
GA4_MEASUREMENT_ID=G-XXXXXXXXXX

# GA4 Measurement Protocol API Secret
GA4_API_SECRET=your-mp-api-secret-here

# Enable/disable GA4 tracking (default: true)
GA4_ENABLED=true

# Enable debug mode (logs validation responses)
GA4_DEBUG=false
```

### Setup Steps

1. **Create GA4 Property**
   - Go to Google Analytics → Admin → Create Property
   - Select "Web" platform
   - Note the **Measurement ID** (e.g., `G-XXXXXXXXXX`)

2. **Create Data Stream**
   - In GA4 → Admin → Data Streams
   - Add new Web stream
   - Note the **Measurement ID** from the stream

3. **Get API Secret**
   - In GA4 → Admin → Data Streams → Your Stream
   - Scroll to "Measurement Protocol API secrets"
   - Click "Create" → Note the **API Secret**

4. **Configure Environment**
   ```bash
   cd /home/ezekiel001/StrellerMinds-SmartContracts/api
   cp .env.example .env
   # Edit .env with your GA4 credentials
   ```

5. **Update CORS (if needed)**
   - Add authorized origins to `CORS_ORIGINS`
   - Default allows GA4 endpoints via CSP

## API Endpoints

### Consent Management

#### Set Consent Preference
```bash
POST /api/v1/analytics/consent
Authorization: Bearer <token>
Content-Type: application/json

{
  "consent": "granted"  # or "denied"
}
```

#### Get Consent Preference
```bash
GET /api/v1/analytics/consent
Authorization: Bearer <token>
```

#### Withdraw Consent
```bash
DELETE /api/v1/analytics/consent
Authorization: Bearer <token>
```

### Per-Request Opt-Out

Clients can control tracking per-request via header:

```bash
# Enable tracking (default)
X-Analytics-Consent: granted

# Disable tracking
X-Analytics-Consent: denied
```

## Architecture

### File Structure

```
api/src/
├── analytics/
│   ├── ga4Client.ts      # GA4 Measurement Protocol client
│   ├── events.ts         # Typed event factories
│   └── index.ts          # Barrel exports
├── middleware/
│   └── analyticsConsent.ts  # GDPR consent middleware
├── routes/
│   ├── analytics.ts      # Aggregate analytics endpoint
│   ├── consent.ts        # Consent management
│   ├── auth.ts           # Auth with GA4 tracking
│   ├── certificates.ts   # Cert endpoints with tracking
│   └── students.ts       # Student endpoints with tracking
└── app.ts                # Middleware configuration
```

### Data Flow

1. **Request Arrives** → `analyticsConsent` middleware reads `X-Analytics-Consent`
2. **Route Handler** → Calls `trackX` function with `req.analyticsOptOut`
3. **Event Factory** → Creates typed `Ga4Event` object
4. **GA4 Client** → Sends via Measurement Protocol (fire-and-forget)
5. **GA4** → Processes and reports in real-time dashboards

## GA4 Dashboards

### Real-Time Reports

Access via Google Analytics → Reports → Realtime

- **Active Users**: Currently active sessions
- **Event Count**: Events per second
- **Conversions**: Track auth and verification conversions

### Engagement Reports

- **Events**: All tracked events with parameters
- **Conversions**: Mark `auth_token_issued` and `certificate_verified` as conversions
- **User Journey**: Navigation flow analysis

### Creating Custom Reports

1. **Exploration Report**
   - Add `event_name` dimension
   - Add `certificate_id` parameter (for verification events)
   - Add `verification_result` parameter

2. **Funnel Analysis**
   - Step 1: `auth_token_issued`
   - Step 2: `certificate_verified`
   - Step 3: `certificate_detail_fetched`

3. **Cohort Analysis**
   - Track user retention via `student_certs_listed`
   - Analyze certificate verification patterns

## Testing

### Unit Tests

The API uses integration testing. No test mocks for GA4:
- Network calls are fire-and-forget
- Errors are logged but never thrown
- Test with `GA4_ENABLED=false` to disable during tests

### Manual Testing

```bash
# 1. Start the API
cd /home/ezekiel001/StrellerMinds-SmartContracts/api
npm run dev

# 2. Get a token
curl -X POST http://localhost:3000/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"apiKey": "your-api-key"}'

# 3. Verify a certificate
curl http://localhost:3000/api/v1/certificates/0000000000000000000000000000000000000000000000000000000000000000/verify

# 4. Check GA4 Realtime dashboard
# Events should appear within seconds
```

### Debug Mode

Enable debug to see GA4 validation responses:

```bash
GA4_DEBUG=true npm run dev
```

Check logs for:
- Event names and parameters
- GA4 validation status
- Any warnings or errors

## Privacy & Security

### No PII Sent

- ❌ No email addresses
- ❌ No names
- ❌ No IP addresses (hashed)
- ❌ No personal identifiers

✅ All IDs are SHA-256 hashed  
✅ Only first 8-16 chars used for analysis  
✅ `non_personalized_ads=true`  
✅ Consent-based tracking  

### Data Retention

GA4 default retention: **2 months**  
Configurable in: GA4 → Admin → Data Settings → Data Retention

### User Rights

Users can:
1. Opt-out per-request (`X-Analytics-Consent: denied`)
2. Revoke consent (`DELETE /consent`)
3. Request data deletion (via GA4 deletion requests)

## Troubleshooting

### Events Not Appearing in GA4

1. **Check credentials**
   ```bash
   echo $GA4_MEASUREMENT_ID
   echo $GA4_API_SECRET
   ```

2. **Verify enabled**
   ```bash
   echo $GA4_ENABLED  # Should be "true"
   ```

3. **Check debug logs**
   ```bash
   GA4_DEBUG=true npm run dev
   ```

4. **Test endpoint manually**
   ```bash
   curl "https://www.google-analytics.com/debug/mp/collect?measurement_id=G-XXXXXXXXXX&api_secret=YOUR_SECRET" \
     -H "Content-Type: application/json" \
     -d '{"client_id":"test","events":[{"name":"test_event"}]}'
   ```

5. **Verify network access**
   - Ensure outbound HTTPS to `www.google-analytics.com` is allowed
   - Check firewall rules

### Validation Errors

Common issues:
- ❌ Invalid Measurement ID format (must start with `G-`)
- ❌ Expired API secret (regenerate in GA4)
- ❌ Wrong property (ensure GA4, not UA)

## Monitoring

### Prometheus Metrics

The API tracks:
- `cert_api_http_requests_total` – Requests by route
- `cert_api_http_request_duration_seconds` – Latency
- `cert_api_verifications_total` – Verification results

### Logs

Structured logging via Winston:
```json
{
  "level": "info",
  "message": "Certificate verified",
  "certificateId": "abc123...",
  "isValid": true,
  "requestId": "uuid"
}
```

## Best Practices

### DO ✅

- Use descriptive event names (snake_case)
- Keep event params minimal (avoid PII)
- Mark key conversions in GA4 UI
- Test with debug mode first
- Monitor GA4 quota limits

### DON'T ❌

- ❌ Send PII in event parameters
- ❌ Block on GA4 requests (always fire-and-forget)
- ❌ Hardcode secrets in code
- ❌ Track without consent (respect opt-out)
- ❌ Use client-side GA4 (use Measurement Protocol)

## Limits & Quotas

- **500,000 events/day** (free tier)
- **Requests/second**: Varies by property
- **Event size**: Max 128 parameters, 256 bytes each
- **Client ID**: Max 256 bytes

See: [GA4 Quotas](https://developers.google.com/analytics/devguides/collection/protocol/ga4/quotas)

## References

- [GA4 Measurement Protocol](https://developers.google.com/analytics/devguides/collection/protocol/ga4)
- [GA4 Event Builder](https://ga-dev-tools.google/ga4/event-builder/)
- [GDPR Compliance Guide](https://support.google.com/analytics/answer/10700670)
- [StrellerMinds API Docs](http://localhost:3000/api/docs)