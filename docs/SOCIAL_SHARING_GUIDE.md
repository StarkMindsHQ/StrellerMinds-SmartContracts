# Social Sharing for Achievements and Credentials

This guide covers the social sharing capabilities added to the StrellerMinds Certificate System, enabling users to share their achievements and credentials across major social media platforms with built-in analytics tracking.

## Overview

The social sharing feature provides:
- **Multi-platform support**: Twitter, LinkedIn, and Facebook
- **Custom share messages**: Users can customize messages when sharing (up to 500 characters)
- **Analytics tracking**: Real-time engagement metrics and analytics
- **Blockchain verification**: All shares are recorded on-chain for authenticity
- **User engagement metrics**: Track 20%+ user engagement through social shares

## Architecture

### Smart Contract Layer (Soroban)

Located in `contracts/social-sharing/src/`:

- **lib.rs**: Main contract implementation with share and analytics functions
- **types.rs**: Data structures (ShareRecord, SocialSharingAnalytics, SharePlatform)
- **storage.rs**: Persistent storage operations and key management
- **events.rs**: Event emission for share tracking
- **errors.rs**: Error handling and codes

### API Layer (Express/TypeScript)

Located in `api/src/`:

- **routes/social-sharing.ts**: RESTful endpoints for sharing operations
- **soroban-client.ts**: Extended with social sharing contract methods
- **types.ts**: TypeScript interfaces for ShareRecord and SocialSharingAnalytics

## API Endpoints

### Share an Achievement

**Endpoint**: `POST /api/v1/social-sharing`

**Authentication**: Required (User must be authenticated)

**Request Body**:
```json
{
  "certificateId": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "platform": "Twitter",
  "customMessage": "Just earned my blockchain certification! 🎓 #Learning"
}
```

**Response**:
```json
{
  "success": true,
  "shareRecord": {
    "certificateId": "0123456789abcdef...",
    "user": "GXXX...",
    "platform": "Twitter",
    "customMessage": "Just earned my blockchain certification! 🎓 #Learning",
    "shareUrl": "twitter://share/0123456789abcdef...",
    "timestamp": 1703001234,
    "engagementCount": 0,
    "verified": false
  },
  "engagement": {
    "platform": "Twitter",
    "shareUrl": "twitter://share/0123456789abcdef...",
    "engagementTarget": 20
  }
}
```

### Get Certificate Shares

**Endpoint**: `GET /api/v1/social-sharing/:certificateId`

**Authentication**: Required

**Response**:
```json
{
  "shares": [...],
  "count": 5
}
```

### Get User's Shares

**Endpoint**: `GET /api/v1/social-sharing/user/shares`

**Authentication**: Required

Returns all shares made by the authenticated user across all certificates and platforms.

### Update Engagement Metrics

**Endpoint**: `PUT /api/v1/social-sharing/:certificateId/engagement`

**Authentication**: Required (Admin only)

**Request Body**:
```json
{
  "user": "GXXX...",
  "platform": "Twitter",
  "engagementCount": 45
}
```

**Purpose**: Update engagement metrics (likes, retweets, shares) for a share event.

### Get Global Analytics

**Endpoint**: `GET /api/v1/social-sharing/analytics`

**Authentication**: Required

**Response**:
```json
{
  "totalShares": 1250,
  "twitterShares": 450,
  "linkedinShares": 350,
  "facebookShares": 450,
  "totalEngagement": 5000,
  "averageEngagement": 4.0,
  "uniqueSharers": 300,
  "lastUpdated": 1703001234,
  "engagementPercentage": 1.67
}
```

### Get Certificate-Specific Analytics

**Endpoint**: `GET /api/v1/social-sharing/certificate/:certificateId/analytics`

**Authentication**: Required

Returns analytics specific to a single certificate.

## Platform Integration

### Twitter Integration

Share to Twitter with support for:
- Custom branded messages
- Pre-formatted hashtags (#BlockchainCertificate)
- Share URL format: `twitter://share/{certificateId}`
- Direct link to certificate verification page

### LinkedIn Integration

Share to LinkedIn with:
- Professional credential formatting
- Course details and skills highlighted
- Share URL format: `linkedin://share/{certificateId}`
- LinkedIn article/post preview

### Facebook Integration

Share to Facebook with:
- Shareable certificate preview
- Certificate image/thumbnail
- Share URL format: `facebook://share/{certificateId}`
- Social proof through engagement metrics

## Analytics and Engagement Tracking

### Engagement Metrics

The system tracks engagement across platforms:
- **Likes**: Reactions and likes on shared content
- **Comments**: Discussion and comments on shares
- **Shares**: Re-shares and forwards
- **Clicks**: Clicks to verify certificate
- **Impressions**: Number of people who see the share

### Engagement Goal: 20% User Engagement

The system is designed to achieve 20% user engagement, defined as:
- Active users sharing achievements
- Social media followers engaging with shares
- Click-through rate to certificate verification

**Calculation**:
```
Engagement Percentage = (Total Engagement / Unique Sharers) × 100%
Target: ≥ 20%
```

### Analytics Data Stored On-Chain

- Total shares per certificate
- Shares per platform
- Cumulative engagement metrics
- Timestamp of shares and updates
- User verification status

## Usage Examples

### Example 1: Share to Twitter
```bash
curl -X POST http://localhost:3000/api/v1/social-sharing \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "certificateId": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "platform": "Twitter",
    "customMessage": "Excited to share my achievement! #BlockchainCertificate"
  }'
```

### Example 2: Get User Engagement Stats
```bash
curl http://localhost:3000/api/v1/social-sharing/analytics \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Example 3: Update Engagement After Social Campaign
```bash
curl -X PUT http://localhost:3000/api/v1/social-sharing/0123456789abc/engagement \
  -H "Authorization: Bearer YOUR_ADMIN_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user": "GXXX...",
    "platform": "Twitter",
    "engagementCount": 150
  }'
```

## Development

### Building the Smart Contract

```bash
cd contracts/social-sharing
cargo build --target wasm32-unknown-unknown --release
```

### Running Tests

```bash
cd contracts/social-sharing
cargo test
```

### Local Development API

```bash
cd api
npm install
npm run dev
```

## Error Handling

### Error Codes

| Code | Meaning |
|------|---------|
| INVALID_CERTIFICATE_ID | Certificate ID format is invalid |
| INVALID_MESSAGE | Message is empty or exceeds 500 characters |
| INVALID_PLATFORM | Platform is not Twitter, LinkedIn, or Facebook |
| SHARE_RECORD_NOT_FOUND | Requested share record doesn't exist |
| UNAUTHORIZED | User is not authorized to perform this action |
| CONTRACT_ERROR | Blockchain contract call failed |

### Example Error Response

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INVALID_MESSAGE",
    "message": "Custom message must be between 1 and 500 characters"
  },
  "meta": {
    "requestId": "req-123-456",
    "timestamp": "2023-12-20T15:30:00Z",
    "version": "1.0.0"
  }
}
```

## Performance Considerations

### Rate Limiting

- Share endpoint: 100 requests per 15 minutes per user
- Analytics endpoint: 30 requests per 15 minutes per user
- General endpoints: 1000 requests per 15 minutes per user

### Storage Optimization

- Share records are stored efficiently on-chain with composite keys
- Engagement metrics are updated incrementally
- Analytics are computed from stored share records

## Security

### Authentication

All endpoints (except public verification) require JWT authentication with valid scope.

### Authorization

- Admin-only endpoints require explicit admin flag in JWT
- Users can only access their own share records
- Analytics updates are only allowed for admins

### On-Chain Verification

- All shares are cryptographically verified on-chain
- Share records include timestamps and user signatures
- Cannot modify historical share data

## Future Enhancements

- Direct API integrations for automatic posting
- Webhook notifications for engagement updates
- Social media analytics dashboards
- Gamification features (sharing badges, achievements)
- Multi-language support for share messages
- Custom branded share templates

## Support

For issues or questions about social sharing features, refer to the main project documentation or open an issue in the repository.

## License

Apache 2.0
