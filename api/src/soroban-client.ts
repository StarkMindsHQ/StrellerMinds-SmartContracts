/**
 * Thin wrapper around the Stellar SDK for calling the Certificate contract.
 * All contract reads are done via simulateTransaction (no signing needed).
 */
import {
  Contract,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  Account,
  xdr,
  scValToNative,
  nativeToScVal,
  Address,
} from "@stellar/stellar-sdk";
import { config } from "./config";
import { logger } from "./logger";
import { contractCallDuration } from "./metrics";
import type {
  Certificate,
  CertificateAnalytics,
  RevocationRecord,
  VerificationResult,
  ShareRecord,
  SocialSharingAnalytics,
  SharePlatform,
} from "./types";

const DUMMY_SOURCE = "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";

export class CertificateContractClient {
  private server: SorobanRpc.Server;
  private contract: Contract;

  constructor() {
    this.server = new SorobanRpc.Server(config.stellar.rpcUrl, {
      allowHttp: config.nodeEnv !== "production",
    });
    this.contract = new Contract(config.stellar.contractId);
  }

  // ── Private helpers ────────────────────────────────────────────────────────

  private async simulate(method: string, args: xdr.ScVal[]): Promise<unknown> {
    const end = contractCallDuration.startTimer({ method, success: "false" });
    try {
      const account = new Account(DUMMY_SOURCE, "0");
      const tx = new TransactionBuilder(account, {
        fee: "100",
        networkPassphrase: config.stellar.networkPassphrase,
      })
        .addOperation(this.contract.call(method, ...args))
        .setTimeout(30)
        .build();

      const result = await this.server.simulateTransaction(tx);

      if (SorobanRpc.Api.isSimulationError(result)) {
        throw new Error(`Contract simulation error: ${result.error}`);
      }
      if (!result.result) {
        throw new Error("No result returned from simulation");
      }

      end({ success: "true" });
      return scValToNative(result.result.retval);
    } catch (err) {
      end({ success: "false" });
      logger.error("Contract call failed", { method, error: err });
      throw err;
    }
  }

  private hexToScVal(hex: string): xdr.ScVal {
    const bytes = Buffer.from(hex.replace(/^0x/, ""), "hex");
    if (bytes.length !== 32) {
      throw new Error("Certificate ID must be 32 bytes (64 hex chars)");
    }
    return xdr.ScVal.scvBytes(bytes);
  }

  private mapCertificate(raw: Record<string, unknown>): Certificate {
    return {
      certificateId: Buffer.from(raw.certificate_id as Uint8Array).toString(
        "hex"
      ),
      courseId: raw.course_id as string,
      student: raw.student as string,
      title: raw.title as string,
      description: raw.description as string,
      metadataUri: raw.metadata_uri as string,
      issuedAt: Number(raw.issued_at),
      expiryDate: Number(raw.expiry_date),
      status: raw.status as Certificate["status"],
      issuer: raw.issuer as string,
      version: Number(raw.version),
      blockchainAnchor: raw.blockchain_anchor
        ? Buffer.from(raw.blockchain_anchor as Uint8Array).toString("hex")
        : null,
      templateId: (raw.template_id as string | null) ?? null,
      shareCount: Number(raw.share_count),
    };
  }

  private mapRevocation(
    raw: Record<string, unknown>
  ): RevocationRecord {
    return {
      certificateId: Buffer.from(raw.certificate_id as Uint8Array).toString(
        "hex"
      ),
      revokedBy: raw.revoked_by as string,
      revokedAt: Number(raw.revoked_at),
      reason: raw.reason as string,
      reissuanceEligible: raw.reissuance_eligible as boolean,
    };
  }

  // ── Public API ─────────────────────────────────────────────────────────────

  /**
   * Verify a certificate by ID. Returns full verification result.
   */
  async verifyCertificate(certificateId: string): Promise<VerificationResult> {
    const now = Math.floor(Date.now() / 1000);
    const certIdArg = this.hexToScVal(certificateId);

    // Fetch certificate data
    let certificate: Certificate | null = null;
    let revocationRecord: RevocationRecord | null = null;
    let isValid = false;
    let message = "";

    try {
      const raw = (await this.simulate("get_certificate", [
        certIdArg,
      ])) as Record<string, unknown> | null;

      if (!raw) {
        return {
          certificateId,
          isValid: false,
          status: "Revoked",
          verifiedAt: now,
          certificate: null,
          revocationRecord: null,
          message: "Certificate not found",
        };
      }

      certificate = this.mapCertificate(raw);

      // Check status
      if (certificate.status === "Active") {
        const expired =
          certificate.expiryDate > 0 && certificate.expiryDate < now;
        if (expired) {
          isValid = false;
          message = "Certificate has expired";
        } else {
          isValid = true;
          message = "Certificate is valid and active";
        }
      } else if (certificate.status === "Revoked") {
        isValid = false;
        message = "Certificate has been revoked";
        // Fetch revocation record
        try {
          const revRaw = (await this.simulate("get_revocation_record", [
            certIdArg,
          ])) as Record<string, unknown> | null;
          if (revRaw) revocationRecord = this.mapRevocation(revRaw);
        } catch {
          // non-fatal
        }
      } else {
        isValid = false;
        message = `Certificate status: ${certificate.status}`;
      }

      return {
        certificateId,
        isValid,
        status: certificate.status,
        verifiedAt: now,
        certificate,
        revocationRecord,
        message,
      };
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg.includes("CertificateNotFound") || msg.includes("not found")) {
        return {
          certificateId,
          isValid: false,
          status: "Revoked",
          verifiedAt: now,
          certificate: null,
          revocationRecord: null,
          message: "Certificate not found",
        };
      }
      throw err;
    }
  }

  /**
   * Get a certificate by ID.
   */
  async getCertificate(certificateId: string): Promise<Certificate | null> {
    const raw = (await this.simulate("get_certificate", [
      this.hexToScVal(certificateId),
    ])) as Record<string, unknown> | null;
    return raw ? this.mapCertificate(raw) : null;
  }

  /**
   * Get all certificate IDs for a student address.
   */
  async getStudentCertificates(studentAddress: string): Promise<string[]> {
    const addressArg = nativeToScVal(
      Address.fromString(studentAddress),
      { type: "address" }
    );
    const raw = (await this.simulate("get_student_certificates", [
      addressArg,
    ])) as Uint8Array[];
    return (raw ?? []).map((b) => Buffer.from(b).toString("hex"));
  }

  /**
   * Get aggregate analytics from the contract.
   */
  async getAnalytics(): Promise<CertificateAnalytics> {
    const raw = (await this.simulate("get_analytics", [])) as Record<
      string,
      unknown
    >;
    return {
      totalIssued: Number(raw.total_issued),
      totalRevoked: Number(raw.total_revoked),
      totalExpired: Number(raw.total_expired),
      totalReissued: Number(raw.total_reissued),
      totalShared: Number(raw.total_shared),
      totalVerified: Number(raw.total_verified),
      activeCertificates: Number(raw.active_certificates),
      pendingRequests: Number(raw.pending_requests),
      avgApprovalTime: Number(raw.avg_approval_time),
      lastUpdated: Number(raw.last_updated),
    };
  }

  /**
   * Get revocation record for a certificate.
   */
  async getRevocationRecord(
    certificateId: string
  ): Promise<RevocationRecord | null> {
    const raw = (await this.simulate("get_revocation_record", [
      this.hexToScVal(certificateId),
    ])) as Record<string, unknown> | null;
    return raw ? this.mapRevocation(raw) : null;
  }

  // ── Social Sharing Methods ─────────────────────────────────────────────────

  /**
   * Record a share of an achievement to social media.
   */
  async shareAchievement(
    userAddress: string,
    certificateId: string,
    platform: SharePlatform,
    customMessage: string
  ): Promise<ShareRecord> {
    const userArg = nativeToScVal(
      Address.fromString(userAddress),
      { type: "address" }
    );
    const certArg = this.hexToScVal(certificateId);
    const platformArg = nativeToScVal(platform === "Twitter" ? 0 : platform === "LinkedIn" ? 1 : 2);
    const messageArg = nativeToScVal(customMessage);

    const raw = (await this.simulate("share_achievement", [
      userArg,
      certArg,
      platformArg,
      messageArg,
    ])) as Record<string, unknown>;

    return this.mapShareRecord(raw);
  }

  /**
   * Get all shares for a specific certificate.
   */
  async getCertificateShares(certificateId: string): Promise<ShareRecord[]> {
    const certArg = this.hexToScVal(certificateId);
    const rawArray = (await this.simulate("get_certificate_shares", [
      certArg,
    ])) as Record<string, unknown>[];

    return (rawArray ?? []).map((raw) => this.mapShareRecord(raw));
  }

  /**
   * Get all shares by a user.
   */
  async getUserShares(userAddress: string): Promise<ShareRecord[]> {
    const userArg = nativeToScVal(
      Address.fromString(userAddress),
      { type: "address" }
    );
    const rawArray = (await this.simulate("get_user_shares", [
      userArg,
    ])) as Record<string, unknown>[];

    return (rawArray ?? []).map((raw) => this.mapShareRecord(raw));
  }

  /**
   * Update engagement metrics for a share (admin only).
   */
  async updateEngagement(
    adminAddress: string,
    certificateId: string,
    userAddress: string,
    platform: SharePlatform,
    engagementCount: number
  ): Promise<void> {
    const adminArg = nativeToScVal(
      Address.fromString(adminAddress),
      { type: "address" }
    );
    const certArg = this.hexToScVal(certificateId);
    const userArg = nativeToScVal(
      Address.fromString(userAddress),
      { type: "address" }
    );
    const platformArg = nativeToScVal(
      platform === "Twitter" ? 0 : platform === "LinkedIn" ? 1 : 2
    );
    const engagementArg = nativeToScVal(engagementCount);

    await this.simulate("update_engagement", [
      adminArg,
      certArg,
      userArg,
      platformArg,
      engagementArg,
    ]);
  }

  /**
   * Get global social sharing analytics.
   */
  async getSocialSharingAnalytics(): Promise<SocialSharingAnalytics> {
    const raw = (await this.simulate("get_analytics", [])) as Record<
      string,
      unknown
    >;
    return {
      totalShares: Number(raw.total_shares),
      twitterShares: Number(raw.twitter_shares),
      linkedinShares: Number(raw.linkedin_shares),
      facebookShares: Number(raw.facebook_shares),
      totalEngagement: Number(raw.total_engagement),
      averageEngagement: Number(raw.average_engagement),
      uniqueSharers: Number(raw.unique_sharers),
      lastUpdated: Number(raw.last_updated),
    };
  }

  /**
   * Get analytics for a specific certificate.
   */
  async getCertificateSocialAnalytics(
    certificateId: string
  ): Promise<SocialSharingAnalytics> {
    const certArg = this.hexToScVal(certificateId);
    const raw = (await this.simulate("get_certificate_analytics", [
      certArg,
    ])) as Record<string, unknown>;
    return {
      totalShares: Number(raw.total_shares),
      twitterShares: Number(raw.twitter_shares),
      linkedinShares: Number(raw.linkedin_shares),
      facebookShares: Number(raw.facebook_shares),
      totalEngagement: Number(raw.total_engagement),
      averageEngagement: Number(raw.average_engagement),
      uniqueSharers: Number(raw.unique_sharers),
      lastUpdated: Number(raw.last_updated),
    };
  }

  /**
   * Track a social share for analytics.
   */
  async trackSocialShare(
    userAddress: string,
    certificateId: string,
    platform: SharePlatform,
    customMessage: string
  ): Promise<void> {
    // Log analytics event
    logger.info("Social share tracked", {
      user: userAddress,
      certificateId,
      platform,
      messageLength: customMessage.length,
    });

    // This could be extended to track in a separate analytics database
    // or call a dedicated analytics contract method
  }

  // ── Helper methods ─────────────────────────────────────────────────────────

  private mapShareRecord(raw: Record<string, unknown>): ShareRecord {
    return {
      certificateId: Buffer.from(raw.certificate_id as Uint8Array).toString(
        "hex"
      ),
      user: raw.user as string,
      platform: ["Twitter", "LinkedIn", "Facebook"][
        Number(raw.platform)
      ] as SharePlatform,
      customMessage: raw.custom_message as string,
      shareUrl: raw.share_url as string,
      timestamp: Number(raw.timestamp),
      engagementCount: Number(raw.engagement_count),
      verified: raw.verified as boolean,
    };
  }
}

// Singleton
export const contractClient = new CertificateContractClient();
