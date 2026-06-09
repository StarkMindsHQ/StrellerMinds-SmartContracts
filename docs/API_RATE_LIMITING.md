# API Rate Limiting Documentation

StrellerMinds employs a multi-layered rate limiting strategy to ensure platform stability, prevent abuse, and provide fair resource allocation across different user tiers.

## 1. Overview
Rate limits are applied at two levels:
1. **Global IP-based Limiting**: Applied to all requests based on the client's IP address.
2. **Tiered User Limiting**: Applied to authenticated requests based on the account's subscription tier.

Limits are calculated using a **sliding window** of 60 seconds.

---

## 2. Global Limits (IP-Based)
These limits apply to all unauthenticated traffic and serve as a first line of defense.

| Endpoint Group | Limit | Description |
|----------------|-------|-------------|
| **General API** | 60 requests / min | Applies to most endpoints (`/auth`, `/students`, etc.) |
| **Verify Endpoint** | 100 requests / min | Specifically for `GET /api/v1/certificates/:id/verify` |

---

## 3. Tiered User Limits (Authenticated)
Authenticated requests are subject to higher limits based on the user's assigned tier. Authenticated users benefit from a **Token Bucket** algorithm that allows for short bursts of traffic.

| Tier | Limit (RPM) | Burst Allowance | Use Case |
|------|-------------|-----------------|----------|
| **Free** | 30 | 10 | Individual students and researchers |
| **Pro** | 120 | 30 | Professional educators and small institutions |
| **Enterprise**| 600 | 100 | Large universities and corporate partners |
| **Internal** | 6,000 | 500 | StrellerMinds internal services and high-trust partners |

---

## 4. Response Headers
Every API response includes headers to help you track your current usage.

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | The maximum number of requests allowed in the current window. |
| `X-RateLimit-Remaining` | The number of requests remaining in the current window. |
| `X-RateLimit-Reset` | The Unix timestamp when the current rate limit window resets. |
| `X-RateLimit-Tier` | The subscription tier applied to the current request. |
| `X-RateLimit-Burst-Limit` | (Authenticated only) The size of your burst bucket. |
| `Retry-After` | (Only on 429 errors) Number of seconds to wait before retrying. |

---

## 5. Error Responses
When a limit is exceeded, the API returns a `429 Too Many Requests` status code.

### 5.1. Global Limit Exceeded
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Please slow down and try again later.",
    "details": {
      "retryAfter": 60
    }
  }
}
```

### 5.2. User Tier Limit Exceeded
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "USER_RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded for tier 'free'. Limit: 30 req/min, burst: 10 req/10s.",
    "details": {
      "tier": "free",
      "limit": 30,
      "burstLimit": 10,
      "retryAfter": 45,
      "upgradeUrl": "https://strellerminds.com/pricing"
    }
  }
}
```

---

## 6. Monitoring Usage
You can programmatically check your current status using the following endpoints:

- **Check Current Status**: `GET /api/v1/rate-limit/status` (Requires Auth)
- **List Tier Definitions**: `GET /api/v1/rate-limit/tiers` (Public)

---

## 7. Best Practices
1. **Honor `Retry-After`**: Always respect the `Retry-After` header. Hard-polling after a 429 may result in temporary IP blocking.
2. **Exponential Backoff**: Implement an exponential backoff strategy for retries to smooth out traffic spikes.
3. **Caching**: Cache immutable resources (like certificate details) locally to reduce redundant API calls.
4. **Header Monitoring**: Monitor `X-RateLimit-Remaining` to proactively slow down requests as you approach your limit.

---

## 8. Code Example (JavaScript)
```javascript
async function fetchWithRetry(url, options = {}) {
  const response = await fetch(url, options);
  
  if (response.status === 429) {
    const retryAfter = response.headers.get('Retry-After') || 5;
    console.warn(`Rate limited. Retrying after ${retryAfter}s...`);
    
    await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
    return fetchWithRetry(url, options);
  }
  
  return response.json();
}
```
