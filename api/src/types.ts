// ── Certificate domain types (mirrors the Soroban contract types) ───────────

export type CertificateStatus =
  | "Active"
  | "Revoked"
  | "Expired"
  | "Suspended"
  | "Reissued";

export interface Certificate {
  certificateId: string; // hex-encoded BytesN<32>
  courseId: string;
  student: string; // Stellar address
  title: string;
  description: string;
  metadataUri: string;
  issuedAt: number; // unix timestamp
  expiryDate: number; // unix timestamp, 0 = no expiry
  status: CertificateStatus;
  issuer: string; // Stellar address
  version: number;
  blockchainAnchor: string | null;
  templateId: string | null;
  shareCount: number;
}

export interface RevocationRecord {
  certificateId: string;
  revokedBy: string;
  revokedAt: number;
  reason: string;
  reissuanceEligible: boolean;
}

export interface VerificationResult {
  certificateId: string;
  isValid: boolean;
  status: CertificateStatus;
  verifiedAt: number;
  certificate: Certificate | null;
  revocationRecord: RevocationRecord | null;
  message: string;
}

export interface CertificateAnalytics {
  totalIssued: number;
  totalRevoked: number;
  totalExpired: number;
  totalReissued: number;
  totalShared: number;
  totalVerified: number;
  activeCertificates: number;
  pendingRequests: number;
  avgApprovalTime: number;
  lastUpdated: number;
}

// ── Social Sharing types ─────────────────────────────────────────────────────

export type SharePlatform = "Twitter" | "LinkedIn" | "Facebook";

export interface ShareRecord {
  certificateId: string;
  user: string; // Stellar address
  platform: SharePlatform;
  customMessage: string;
  shareUrl: string;
  timestamp: number;
  engagementCount: number;
  verified: boolean;
}

export interface SocialSharingAnalytics {
  totalShares: number;
  twitterShares: number;
  linkedinShares: number;
  facebookShares: number;
  totalEngagement: number;
  averageEngagement: number;
  uniqueSharers: number;
  lastUpdated: number;
}

export interface ShareRequest {
  certificateId: string;
  platform: SharePlatform;
  customMessage?: string;
}

export interface ShareResponse {
  success: boolean;
  shareRecord: ShareRecord;
  engagement: {
    platform: SharePlatform;
    shareUrl: string;
    engagementTarget: number; // target engagement for 20% user engagement goal
  };
}

// ── API response envelope ────────────────────────────────────────────────────

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: ApiError | null;
  meta: ResponseMeta;
}

export interface ApiError {
  code: string;
  message: string;
  details?: unknown;
}

export interface ResponseMeta {
  requestId: string;
  timestamp: string;
  version: string;
}

// ── Auth types ───────────────────────────────────────────────────────────────

export type RateLimitTier = "free" | "pro" | "enterprise" | "internal";

export interface JwtPayload {
  sub: string;       // subject (API key id or user id)
  iat: number;
  exp: number;
  scope: string[];   // e.g. ['verify', 'read']
  tier?: RateLimitTier; // rate limit tier, defaults to 'free'
}

// ── Rate limit usage analytics ───────────────────────────────────────────────

export interface UserRateLimitStatus {
  userId: string;
  tier: RateLimitTier;
  /** Requests consumed in the current window */
  consumed: number;
  /** Remaining requests in the current window */
  remaining: number;
  /** Requests per minute allowed for this tier */
  limit: number;
  /** Burst allowance for this tier */
  burstLimit: number;
  /** Unix timestamp when the window resets */
  resetAt: number;
  /** Whether the user is currently throttled */
  throttled: boolean;
}
