# Employer Credential Verification API

## Overview

The Employer Credential Verification API provides secure, authenticated endpoints for employers to verify student credentials issued on the StrellerMinds platform. This API includes comprehensive security features, rate limiting, audit logging, and batch processing capabilities.

## Features

- **Employer Authentication**: Multi-method authentication (JWT, API Key, OAuth2)
- **Tiered Rate Limiting**: Different limits based on subscription tiers
- **Comprehensive Audit Logging**: Full audit trail for compliance
- **Batch Verification**: Process multiple credentials efficiently
- **Enhanced Verification Levels**: Basic, Enhanced, and Comprehensive verification
- **Real-time Metrics**: Performance and usage analytics

## Authentication

### Supported Authentication Methods

1. **JWT Bearer Token**
   ```
   Authorization: Bearer <jwt_token>
   ```

2. **API Key**
   ```
   Authorization: <api_key>
   ```

3. **OAuth2 Bearer Token** (Future)
   ```
   Authorization: Bearer <oauth2_token>
   ```

### Getting API Credentials

#### Demo API Keys
For development and testing, use these demo API keys:

| Tier | API Key | Rate Limits |
|------|---------|-------------|
| Basic | `emp_demo_key_basic` | 100 verifications/hour |
| Premium | `emp_demo_key_premium` | 500 verifications/hour |
| Enterprise | `emp_demo_key_enterprise` | 10,000 verifications/hour |

#### JWT Authentication
1. Obtain a JWT token from `/api/v1/auth/token` endpoint
2. Include it in the Authorization header

## API Endpoints

### 1. Single Credential Verification

**POST** `/api/v1/employer/verify`

Verify a single student credential with enhanced security and logging.

#### Request Body

```json
{
  "certificateId": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "studentAddress": "GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
  "verificationLevel": "basic|enhanced|comprehensive",
  "includeMetadata": false
}
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `certificateId` | string | Yes | 64-character hex certificate ID |
| `studentAddress` | string | Yes | Stellar public key of student |
| `verificationLevel` | string | No | Verification level (default: "basic") |
| `includeMetadata` | boolean | No | Include verification metadata (default: false) |

#### Verification Levels

- **Basic**: Standard certificate validity check
- **Enhanced**: Additional student address verification and cross-references
- **Comprehensive**: Deep analytics, risk assessment, and historical verification data

#### Response

```json
{
  "success": true,
  "data": {
    "isValid": true,
    "certificate": {
      "id": "0x1234...",
      "studentAddress": "GABCD...",
      "issuer": "StrellerMinds",
      "issueDate": "2024-01-15T10:30:00Z",
      "expiryDate": "2025-01-15T10:30:00Z",
      "courseName": "Blockchain Development",
      "grade": "A"
    },
    "enhancedVerification": {
      "studentAddressMatch": true,
      "additionalChecks": "passed",
      "verificationTimestamp": "2024-04-28T15:30:00Z"
    },
    "comprehensiveVerification": {
      "riskScore": "low",
      "historicalVerificationCount": 5,
      "complianceChecks": "passed",
      "comprehensiveTimestamp": "2024-04-28T15:30:00Z"
    }
  },
  "meta": {
    "requestId": "req_123456789",
    "timestamp": "2024-04-28T15:30:00Z",
    "version": "1.0.0"
  }
}
```

#### Example Request

```bash
curl -X POST https://api.strellerminds.com/api/v1/employer/verify \
  -H "Authorization: emp_demo_key_premium" \
  -H "Content-Type: application/json" \
  -d '{
    "certificateId": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "studentAddress": "GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "verificationLevel": "enhanced",
    "includeMetadata": true
  }'
```

### 2. Batch Credential Verification

**POST** `/api/v1/employer/verify/batch`

Verify multiple credentials in a single request for efficiency.

#### Request Body

```json
{
  "verifications": [
    {
      "certificateId": "0x1234...",
      "studentAddress": "GABCD..."
    },
    {
      "certificateId": "0x5678...",
      "studentAddress": "GEFGH..."
    }
  ],
  "verificationLevel": "basic|enhanced|comprehensive",
  "includeMetadata": false
}
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `verifications` | array | Yes | Array of verification objects (max 50) |
| `verificationLevel` | string | No | Verification level for all items (default: "basic") |
| `includeMetadata` | boolean | No | Include verification metadata (default: false) |

#### Response

```json
{
  "success": true,
  "data": {
    "results": [
      {
        "certificateId": "0x1234...",
        "studentAddress": "GABCD...",
        "success": true,
        "result": {
          "isValid": true,
          "certificate": { /* certificate data */ }
        }
      },
      {
        "certificateId": "0x5678...",
        "studentAddress": "GEFGH...",
        "success": false,
        "error": "Certificate not found"
      }
    ],
    "summary": {
      "total": 2,
      "successful": 1,
      "failed": 1,
      "successRate": "50.00%"
    },
    "metadata": {
      "verificationLevel": "enhanced",
      "duration": 2500,
      "timestamp": "2024-04-28T15:30:00Z",
      "employerId": "emp_002"
    }
  },
  "meta": {
    "requestId": "req_123456790",
    "timestamp": "2024-04-28T15:30:00Z",
    "version": "1.0.0"
  }
}
```

#### Example Request

```bash
curl -X POST https://api.strellerminds.com/api/v1/employer/verify/batch \
  -H "Authorization: emp_demo_key_enterprise" \
  -H "Content-Type: application/json" \
  -d '{
    "verifications": [
      {
        "certificateId": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "studentAddress": "GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ"
      },
      {
        "certificateId": "0x567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456",
        "studentAddress": "GBCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXY"
      }
    ],
    "verificationLevel": "comprehensive",
    "includeMetadata": true
  }'
```

### 3. Verification History

**GET** `/api/v1/employer/verification-history`

Retrieve verification history for the authenticated employer.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | No | Number of records to return (max 1000, default 100) |
| `offset` | integer | No | Number of records to skip (default 0) |

#### Response

```json
{
  "success": true,
  "data": {
    "history": [
      {
        "type": "verification_success",
        "data": {
          "employerId": "emp_002",
          "certificateId": "0x1234...",
          "studentAddress": "GABCD...",
          "verificationLevel": "enhanced",
          "result": "valid",
          "duration": 1200,
          "timestamp": "2024-04-28T15:30:00Z"
        },
        "timestamp": "2024-04-28T15:30:00Z",
        "id": "audit_1714324200_abc123def"
      }
    ],
    "pagination": {
      "limit": 100,
      "offset": 0,
      "total": 1
    }
  },
  "meta": {
    "requestId": "req_123456791",
    "timestamp": "2024-04-28T15:30:00Z",
    "version": "1.0.0"
  }
}
```

#### Example Request

```bash
curl -X GET "https://api.strellerminds.com/api/v1/employer/verification-history?limit=50&offset=0" \
  -H "Authorization: emp_demo_key_premium"
```

## Rate Limiting

### Rate Limit Tiers

| Subscription Tier | Single Verifications | Batch Verifications | Read Operations |
|------------------|---------------------|---------------------|-----------------|
| Basic | 100/hour | 10/hour | 1,000/hour |
| Premium | 500/hour | 50/hour | 5,000/hour |
| Enterprise | 10,000/hour | 1,000/hour | 100,000/hour |

### Rate Limit Headers

All API responses include rate limit headers:

```
X-RateLimit-Limit: 500
X-RateLimit-Remaining: 499
X-RateLimit-Reset: 2024-04-28T16:30:00Z
```

### Rate Limit Exceeded

When rate limits are exceeded, the API returns:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again in 300 seconds.",
    "retryAfter": 300
  },
  "meta": {
    "requestId": "req_123456792",
    "timestamp": "2024-04-28T15:30:00Z",
    "version": "1.0.0"
  }
}
```

## Error Handling

### Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": [ /* validation errors if applicable */ ]
  },
  "meta": {
    "requestId": "req_123456793",
    "timestamp": "2024-04-28T15:30:00Z",
    "version": "1.0.0"
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Authentication required or failed |
| `INSUFFICIENT_PERMISSIONS` | 403 | Permission denied for operation |
| `INSUFFICIENT_SUBSCRIPTION` | 403 | Higher subscription tier required |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `INVALID_CERTIFICATE_ID` | 400 | Invalid certificate ID format |
| `BATCH_TOO_LARGE` | 400 | Batch size exceeds maximum |
| `CERTIFICATE_NOT_FOUND` | 404 | Certificate not found on blockchain |
| `RATE_LIMIT_EXCEEDED` | 429 | Rate limit exceeded |
| `CONTRACT_ERROR` | 502 | Blockchain query failed |
| `INTERNAL_ERROR` | 500 | Internal server error |

## Security Features

### Authentication Security

- **Timing-safe comparison** for API key validation
- **JWT token expiration** and validation
- **Rate limiting on authentication attempts**
- **IP-based tracking** for security monitoring

### Data Protection

- **HTTPS enforcement** for all API calls
- **Input validation** and sanitization
- **SQL injection prevention** (if database used)
- **Cross-Site Scripting (XSS) prevention**

### Audit Logging

All verification activities are logged with:

- Employer identification
- Certificate IDs (hashed for privacy)
- Timestamps
- IP addresses
- User agents
- Success/failure status
- Performance metrics

## Performance Metrics

### Available Metrics

The API exposes Prometheus metrics at `/metrics`:

- `cert_api_employer_verifications_total` - Total verification attempts
- `cert_api_batch_verifications_total` - Total batch verifications
- `cert_api_verification_duration_seconds` - Verification duration histogram
- `cert_api_employer_authentications_total` - Authentication attempts
- `cert_api_employer_rate_limit_hits_total` - Rate limit hits

### Performance Benchmarks

| Operation | Average Duration | P95 Duration | P99 Duration |
|-----------|------------------|--------------|--------------|
| Basic Verification | 200ms | 400ms | 800ms |
| Enhanced Verification | 500ms | 1s | 2s |
| Comprehensive Verification | 1.2s | 2.5s | 5s |
| Batch Verification (10 items) | 2s | 4s | 8s |

## SDK and Integration

### JavaScript/TypeScript SDK

```typescript
import { EmployerVerificationClient } from '@strellerminds/employer-sdk';

const client = new EmployerVerificationClient({
  apiKey: 'emp_demo_key_premium',
  baseUrl: 'https://api.strellerminds.com'
});

// Single verification
const result = await client.verify({
  certificateId: '0x1234...',
  studentAddress: 'GABCD...',
  verificationLevel: 'enhanced'
});

// Batch verification
const batchResult = await client.verifyBatch({
  verifications: [
    { certificateId: '0x1234...', studentAddress: 'GABCD...' },
    { certificateId: '0x5678...', studentAddress: 'GEFGH...' }
  ],
  verificationLevel: 'comprehensive'
});
```

### Python SDK

```python
from strellerminds import EmployerVerificationClient

client = EmployerVerificationClient(
    api_key='emp_demo_key_premium',
    base_url='https://api.strellerminds.com'
)

# Single verification
result = client.verify(
    certificate_id='0x1234...',
    student_address='GABCD...',
    verification_level='enhanced'
)

# Batch verification
batch_result = client.verify_batch(
    verifications=[
        {'certificate_id': '0x1234...', 'student_address': 'GABCD...'},
        {'certificate_id': '0x5678...', 'student_address': 'GEFGH...'}
    ],
    verification_level='comprehensive'
)
```

## Support and Documentation

### Additional Resources

- [API Documentation](https://docs.strellerminds.com/api)
- [SDK Documentation](https://docs.strellerminds.com/sdk)
- [Support Portal](https://support.strellerminds.com)
- [Status Page](https://status.strellerminds.com)

### Contact Support

- **Email**: api-support@strellerminds.com
- **Documentation**: https://docs.strellerminds.com
- **GitHub Issues**: https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/issues

### API Versioning

- Current version: **v1.0.0**
- Versioning strategy: Semantic versioning
- Backward compatibility: Maintained within major versions
- Deprecation notices: 90 days advance notice

## Testing and Sandbox

### Sandbox Environment

For testing, use the sandbox environment:

- **Base URL**: `https://api-sandbox.strellerminds.com`
- **Test API Keys**: Same as demo keys above
- **Blockchain**: Testnet with mock certificates

### Test Certificates

The sandbox environment includes pre-configured test certificates:

| Certificate ID | Student Address | Status |
|----------------|------------------|---------|
| `0x1111...` | `GTEST1...` | Valid |
| `0x2222...` | `GTEST2...` | Expired |
| `0x3333...` | `GTEST3...` | Revoked |
| `0x4444...` | `GTEST4...` | Not Found |

## Compliance and Legal

### Data Privacy

- **GDPR Compliant**: Data processing and storage practices
- **Data Minimization**: Only collect necessary verification data
- **Retention Policies**: Audit logs retained for 7 years
- **Right to Erasure**: Data deletion upon request

### Security Certifications

- **ISO 27001**: Information security management
- **SOC 2 Type II**: Security controls and processes
- **PCI DSS**: Payment card industry compliance (if applicable)

### Legal Notices

- **Terms of Service**: https://strellerminds.com/terms
- **Privacy Policy**: https://strellerminds.com/privacy
- **Acceptable Use**: https://strellerminds.com/acceptable-use
