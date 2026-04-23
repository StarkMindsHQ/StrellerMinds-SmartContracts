export interface Certificate {
  certificateId: string;
  courseId: string;
  student: string;
  title: string;
  description: string;
  metadataUri: string;
  issuedAt: number;
  expiryDate: number;
  status: 'active' | 'revoked' | 'expired' | 'reissued';
  issuer: string;
  version: number;
  blockchainAnchor?: string;
  templateId?: string;
  shareCount: number;
}

export interface SharedCredentialPayload {
  certificateId: string;
  student: string;
  title: string;
  issuer: string;
  blockchainAnchor: string;
  timestamp: number;
  signature?: string;
}

export interface OfflineCacheEntry {
  certificate: Certificate;
  cachedAt: number;
  expiresAt: number;
}

export interface BiometricAuthResult {
  success: boolean;
  error?: string;
}

export interface NotificationPayload {
  title: string;
  body: string;
  data?: Record<string, any>;
}
