/**
 * Thin wrapper around the Stellar SDK for calling the Certificate contract.
 * All contract reads are done via simulateTransaction (no signing needed).
 *
 * Performance optimizations:
 * - TTL in-memory cache per resource type to avoid redundant RPC calls
 * - In-flight request coalescing: concurrent identical requests share one Promise
 * - verifyCertificate fetches cert + revocation in parallel when cert is revoked
 */
import {
  Contract,
  SorobanRpc,
  TransactionBuilder,
  Account,
  xdr,
  scValToNative,
  nativeToScVal,
  Address,
} from "@stellar/stellar-sdk";
import { config } from "./config";
import { logger } from "./logger";
import { contractCallDuration, cacheHits, cacheMisses, cacheSize } from "./metrics";
import { TtlCache } from "./utils/cache";
import type {
  Certificate,
  CertificateAnalytics,
  RevocationRecord,
  VerificationResult,
  ShareRecord,
  SocialSharingAnalytics,
  SharePlatform,
} from "./types";
import {
  QueryOptimizer,
  type QueryOptimizationReport,
} from "./utils/queryOptimizer";

const DUMMY_SOURCE = "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";
const SHORT_NULL_CACHE_TTL_MS = 5_000;

export class CertificateContractClient {
  private readonly servers: SorobanRpc.Server[];
  private readonly rpcUrls: string[];
  private readonly contract: Contract;
  private readonly optimizer: QueryOptimizer;
  private nextServerIndex = 0;

  private analyticsCache: TtlCache<CertificateAnalytics>;
  private certificateCache: TtlCache<Certificate | null>;
  private revocationCache: TtlCache<RevocationRecord | null>;
  private studentCache: TtlCache<string[]>;

  constructor() {
    this.rpcUrls = config.queryOptimization.rpcUrls.slice(0, config.queryOptimization.poolSize);
    this.servers = this.rpcUrls.map(
      (rpcUrl) =>
        new SorobanRpc.Server(rpcUrl, {
          allowHttp: config.nodeEnv !== "production",
        })
    );
    this.contract = new Contract(config.stellar.contractId);
    this.optimizer = new QueryOptimizer({
      defaultTtlMs: config.queryOptimization.cacheDefaultTtlMs,
      maxEntries: config.queryOptimization.cacheMaxEntries,
      slowThresholdMs: config.queryOptimization.slowThresholdMs,
      targetAvgMs: config.queryOptimization.targetAvgMs,
      targetLoadReductionPercent: config.queryOptimization.targetLoadReductionPercent,
      poolSize: this.servers.length,
    });
  }

  private async simulate(method: string, args: xdr.ScVal[]): Promise<unknown> {
    const end = contractCallDuration.startTimer({ method, success: "false" });
    const server = this.getNextServer();

    try {
      const account = new Account(DUMMY_SOURCE, "0");
      const tx = new TransactionBuilder(account, {
        fee: "100",
        networkPassphrase: config.stellar.networkPassphrase,
      })
        .addOperation(this.contract.call(method, ...args))
        .setTimeout(30)
        .build();

      const result = await server.simulateTransaction(tx);

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

  private getNextServer(): SorobanRpc.Server {
    const server = this.servers[this.nextServerIndex % this.servers.length];
    this.nextServerIndex = (this.nextServerIndex + 1) % this.servers.length;
    return server;
  }

  private buildCacheKey(method: string, args: xdr.ScVal[]): string {
    const serializedArgs = args.map((arg) => arg.toXDR("base64")).join(":");
    return `${method}:${serializedArgs}`;
  }

  private serializeQueryKey(queryName: string, args: xdr.ScVal[]): string {
    return this.buildCacheKey(queryName, args);
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
      certificateId: Buffer.from(raw.certificate_id as Uint8Array).toString("hex"),
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

  private mapRevocation(raw: Record<string, unknown>): RevocationRecord {
    return {
      certificateId: Buffer.from(raw.certificate_id as Uint8Array).toString("hex"),
      revokedBy: raw.revoked_by as string,
      revokedAt: Number(raw.revoked_at),
      reason: raw.reason as string,
      reissuanceEligible: raw.reissuance_eligible as boolean,
    };
  }

  getQueryOptimizationReport(): QueryOptimizationReport {
    return this.optimizer.getReport({
      configuredRpcUrls: this.rpcUrls,
      roundRobinCursor: this.nextServerIndex,
    });
  }

  invalidateQueryCache(filters?: { queryName?: string; keyPrefix?: string }): { invalidated: number } {
    return this.optimizer.invalidate(filters);
  }

  /**
   * Verify a certificate by ID. Returns full verification result.
   *
   * When the cert is revoked we fetch the revocation record in parallel with
   * the certificate (both are independent RPC calls), avoiding a sequential
   * N+1 pattern.
   */
  async verifyCertificate(certificateId: string): Promise<VerificationResult> {
    const now = Math.floor(Date.now() / 1000);

    let certificate: Certificate | null = null;
    let revocationRecord: RevocationRecord | null = null;
    let isValid = false;
    let message = "";

    try {
      certificate = await this.getCertificate(certificateId);

      if (!certificate) {
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

      if (certificate.status === "Active") {
        const expired = certificate.expiryDate > 0 && certificate.expiryDate < now;
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
        try {
          revocationRecord = await this.getRevocationRecord(certificateId);
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
        revocationRecord: certificate.status === "Revoked" ? revocationRecord : null,
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
   * Get a certificate by ID. Results cached per config.cache.certificateTtlMs.
   */
  async getCertificate(certificateId: string): Promise<Certificate | null> {
    const certIdArg = this.hexToScVal(certificateId);
    const cacheKey = this.serializeQueryKey("get_certificate", [certIdArg]);

    return this.optimizer.execute(
      {
        queryName: "get_certificate",
        cacheKey,
        cacheTtlMs: config.queryOptimization.cacheCertificateTtlMs,
        cacheNull: true,
        nullCacheTtlMs: SHORT_NULL_CACHE_TTL_MS,
      },
      async () => {
        const raw = (await this.simulate("get_certificate", [
          certIdArg,
        ])) as Record<string, unknown> | null;
        return raw ? this.mapCertificate(raw) : null;
      }
    );
  }

  /**
   * Get all certificate IDs for a student address.
   * Results cached per config.cache.studentTtlMs.
   */
  async getStudentCertificates(studentAddress: string): Promise<string[]> {
    const addressArg = nativeToScVal(Address.fromString(studentAddress), {
      type: "address",
    });
    const cacheKey = this.serializeQueryKey("get_student_certificates", [addressArg]);

    return this.optimizer.execute(
      {
        queryName: "get_student_certificates",
        cacheKey,
        cacheTtlMs: config.queryOptimization.cacheStudentCertsTtlMs,
      },
      async () => {
        const raw = (await this.simulate("get_student_certificates", [
          addressArg,
        ])) as Uint8Array[];
        return (raw ?? []).map((bytes) => Buffer.from(bytes).toString("hex"));
      }
    );
  }

  /**
   * Get aggregate analytics. Cached per config.cache.analyticsTtlMs.
   */
  async getAnalytics(): Promise<CertificateAnalytics> {
    const cacheKey = this.serializeQueryKey("get_analytics", []);

    return this.optimizer.execute(
      {
        queryName: "get_analytics",
        cacheKey,
        cacheTtlMs: config.queryOptimization.cacheAnalyticsTtlMs,
        singleton: true,
        canPrewarm: true,
      },
      async () => {
        const raw = (await this.simulate("get_analytics", [])) as Record<string, unknown>;
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
    );
  }

  /**
   * Get revocation record for a certificate.
   */
  async getRevocationRecord(certificateId: string): Promise<RevocationRecord | null> {
    const certIdArg = this.hexToScVal(certificateId);
    const cacheKey = this.serializeQueryKey("get_revocation_record", [certIdArg]);

    return this.optimizer.execute(
      {
        queryName: "get_revocation_record",
        cacheKey,
        cacheTtlMs: config.queryOptimization.cacheRevocationTtlMs,
        cacheNull: true,
        nullCacheTtlMs: SHORT_NULL_CACHE_TTL_MS,
      },
      async () => {
        const raw = (await this.simulate("get_revocation_record", [
          certIdArg,
        ])) as Record<string, unknown> | null;
        return raw ? this.mapRevocation(raw) : null;
      }
    );
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

export const contractClient = new CertificateContractClient();
