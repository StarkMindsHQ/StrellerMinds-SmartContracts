# Certificate Verification API

Public REST API for verifying educational certificates issued on the Stellar blockchain via the StrellerMinds smart contract platform.

## Architecture

```
api/
├── src/
│   ├── server.ts           # Entry point, graceful shutdown
│   ├── app.ts              # Express app, middleware stack
│   ├── config.ts           # Environment config
│   ├── logger.ts           # Winston structured logger
│   ├── metrics.ts          # Prometheus metrics
│   ├── openapi.ts          # OpenAPI 3.0 spec
│   ├── soroban-client.ts   # Stellar contract client
│   ├── types.ts            # Shared TypeScript types
│   ├── middleware/
│   │   ├── auth.ts         # JWT authentication
│   │   ├── rateLimiter.ts  # express-rate-limit
│   │   ├── requestId.ts    # X-Request-ID injection
│   │   └── metricsMiddleware.ts
│   ├── routes/
│   │   ├── auth.ts         # POST /api/v1/auth/token
│   │   ├── certificates.ts # GET /api/v1/certificates/*
│   │   ├── students.ts     # GET /api/v1/students/*
│   │   ├── analytics.ts    # GET /api/v1/analytics
│   │   └── health.ts       # GET /health/*
│   └── utils/
│       ├── response.ts     # Envelope helpers
│       └── validate.ts     # Zod schemas
```

## Setup

```bash
cp .env.example .env
# Edit .env with your contract ID and secrets

npm install
npm run build
npm start
```

For development:
```bash
npm run dev
```

## Environment Variables

| Variable | Description | Default |
|---|---|---|
| `STELLAR_RPC_URL` | Soroban RPC endpoint | testnet |
| `STELLAR_NETWORK_PASSPHRASE` | Network passphrase | testnet |
| `CERTIFICATE_CONTRACT_ID` | Deployed contract address | — |
| `PORT` | HTTP port | 3000 |
| `JWT_SECRET` | Secret for signing JWTs | — |
| `JWT_EXPIRES_IN` | Token lifetime | 1h |
| `DEMO_API_KEY` | API key for token issuance | — |
| `RATE_LIMIT_WINDOW_MS` | Rate limit window | 60000 |
| `RATE_LIMIT_MAX_REQUESTS` | General limit per window | 60 |
| `RATE_LIMIT_VERIFY_MAX` | Verify endpoint limit | 100 |
| `CORS_ORIGINS` | Comma-separated allowed origins | localhost:3000 |

## API Endpoints

### Authentication

```
POST /api/v1/auth/token
```
Exchange an API key for a JWT. No auth required.

```json
{ "apiKey": "your-api-key" }
```

Response:
```json
{
  "success": true,
  "data": {
    "accessToken": "eyJ...",
    "tokenType": "Bearer",
    "expiresIn": "1h",
    "scope": ["verify", "read"]
  }
}
```

### Certificate Verification (Public)

```
GET /api/v1/certificates/:id/verify
```

No authentication required. Rate-limited to 100 req/min.

`:id` — 64-character hex certificate ID.

```json
{
  "success": true,
  "data": {
    "certificateId": "abc123...",
    "isValid": true,
    "status": "Active",
    "verifiedAt": 1714000000,
    "certificate": { ... },
    "revocationRecord": null,
    "message": "Certificate is valid and active"
  }
}
```

### Certificate Details (Auth Required)

```
GET /api/v1/certificates/:id
GET /api/v1/certificates/:id/revocation
```

### Student Certificates (Auth Required)

```
GET /api/v1/students/:address/certificates
```

`:address` — Stellar public key (G...).

### Analytics (Auth Required)

```
GET /api/v1/analytics
```

### Health & Monitoring

```
GET /health          # Liveness probe
GET /health/ready    # Readiness probe (checks contract connectivity)
GET /health/metrics  # Prometheus metrics scrape endpoint
```

## API Documentation

Interactive Swagger UI available at:
```
http://localhost:3000/api/docs
```

## Response Envelope

All responses follow a consistent envelope:

```json
{
  "success": true | false,
  "data": { ... } | null,
  "error": { "code": "...", "message": "..." } | null,
  "meta": {
    "requestId": "uuid",
    "timestamp": "ISO-8601",
    "version": "1.0.0"
  }
}
```

## Error Codes

| Code | HTTP | Description |
|---|---|---|
| `AUTH_REQUIRED` | 401 | Missing Authorization header |
| `TOKEN_EXPIRED` | 401 | JWT has expired |
| `TOKEN_INVALID` | 401 | JWT is malformed or invalid |
| `INSUFFICIENT_SCOPE` | 403 | Token lacks required scope |
| `INVALID_CERTIFICATE_ID` | 400 | ID is not a 64-char hex string |
| `INVALID_ADDRESS` | 400 | Not a valid Stellar address |
| `CERTIFICATE_NOT_FOUND` | 404 | No certificate with that ID |
| `REVOCATION_NOT_FOUND` | 404 | No revocation record found |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `CONTRACT_ERROR` | 502 | Soroban RPC call failed |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

## Security

- All responses include security headers via `helmet`
- JWT tokens are short-lived (default 1h)
- Rate limiting on all endpoints (stricter on public verify endpoint)
- Input validation via `zod` on all parameters
- CORS restricted to configured origins
- No private keys stored — all reads use simulation (no signing)

## Monitoring

Prometheus metrics are exposed at `GET /health/metrics`:

| Metric | Type | Description |
|---|---|---|
| `cert_api_http_request_duration_seconds` | Histogram | Request latency by route |
| `cert_api_http_requests_total` | Counter | Total requests by route/status |
| `cert_api_verifications_total` | Counter | Verifications by result |
| `cert_api_contract_call_duration_seconds` | Histogram | Soroban call latency |
| `cert_api_rate_limit_hits_total` | Counter | Rate limit hits by endpoint |
| Default Node.js metrics | — | CPU, memory, event loop |
