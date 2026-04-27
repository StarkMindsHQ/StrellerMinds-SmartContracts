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
} from "./types";

const DUMMY_SOURCE = "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";

export class CertificateContractClient {
  private server: SorobanRpc.Server;
  private contract: Contract;

  private analyticsCache: TtlCache<CertificateAnalytics>;
  private certificateCache: TtlCache<Certificate | null>;
  private revocationCache: TtlCache<RevocationRecord | null>;
  private studentCache: TtlCache<string[]>;

  constructor() {
    this.server = new SorobanRpc.Server(config.stellar.rpcUrl, {
      allowHttp: config.nodeEnv !== "production",
    });
    this.contract = new Contract(config.stellar.contractId);

    this.analyticsCache = new TtlCache(config.cache.analyticsTtlMs);
    this.certificateCache = new TtlCache(config.cache.certificateTtlMs);
    this.revocationCache = new TtlCache(config.cache.revocationTtlMs);
    this.studentCache = new TtlCache(config.cache.studentTtlMs);

    // Periodically publish cache sizes to Prometheus
    setInterval(() => this.publishCacheMetrics(), 15_000).unref();
  }

  // ── Private helpers ────────────────────────────────────────────────────────

  private publishCacheMetrics(): void {
    cacheSize.set({ cache: "analytics" }, this.analyticsCache.stats().size);
    cacheSize.set({ cache: "certificate" }, this.certificateCache.stats().size);
    cacheSize.set({ cache: "revocation" }, this.revocationCache.stats().size);
    cacheSize.set({ cache: "student" }, this.studentCache.stats().size);
  }

  private trackCacheResult(cacheName: string, hit: boolean): void {
    if (hit) {
      cacheHits.inc({ cache: cacheName });
    } else {
      cacheMisses.inc({ cache: cacheName });
    }
  }

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
   *
   * When the cert is revoked we fetch the revocation record in parallel with
   * the certificate (both are independent RPC calls), avoiding a sequential
   * N+1 pattern.
   */
  async verifyCertificate(certificateId: string): Promise<VerificationResult> {
    const now = Math.floor(Date.now() / 1000);
    const certIdArg = this.hexToScVal(certificateId);

    try {
      // Fire cert + revocation fetches in parallel.
      // getCertificate uses its own cache; getRevocationRecord uses its own.
      // Both are coalesced, so concurrent identical calls share one RPC.
      const [certificate, revocationRecord] = await Promise.all([
        this.getCertificate(certificateId),
        this.getRevocationRecord(certificateId).catch(() => null),
      ]);

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

      let isValid = false;
      let message = "";

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
    const key = `cert:${certificateId}`;
    const beforeHits = this.certificateCache.hits;

    const result = await this.certificateCache.getOrFetch(key, async () => {
      const raw = (await this.simulate("get_certificate", [
        this.hexToScVal(certificateId),
      ])) as Record<string, unknown> | null;
      return raw ? this.mapCertificate(raw) : null;
    });

    this.trackCacheResult("certificate", this.certificateCache.hits > beforeHits);
    return result;
  }

  /**
   * Get all certificate IDs for a student address.
   * Results cached per config.cache.studentTtlMs.
   */
  async getStudentCertificates(studentAddress: string): Promise<string[]> {
    const key = `student:${studentAddress}`;
    const beforeHits = this.studentCache.hits;

    const result = await this.studentCache.getOrFetch(key, async () => {
      const addressArg = nativeToScVal(
        Address.fromString(studentAddress),
        { type: "address" }
      );
      const raw = (await this.simulate("get_student_certificates", [
        addressArg,
      ])) as Uint8Array[];
      return (raw ?? []).map((b) => Buffer.from(b).toString("hex"));
    });

    this.trackCacheResult("student", this.studentCache.hits > beforeHits);
    return result;
  }

  /**
   * Get aggregate analytics. Cached per config.cache.analyticsTtlMs.
   */
  async getAnalytics(): Promise<CertificateAnalytics> {
    const key = "analytics:global";
    const beforeHits = this.analyticsCache.hits;

    const result = await this.analyticsCache.getOrFetch(key, async () => {
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
    });

    this.trackCacheResult("analytics", this.analyticsCache.hits > beforeHits);
    return result;
  }

  /**
   * Get revocation record for a certificate.
   * Cached per config.cache.revocationTtlMs (revocations are immutable once set).
   */
  async getRevocationRecord(
    certificateId: string
  ): Promise<RevocationRecord | null> {
    const key = `revocation:${certificateId}`;
    const beforeHits = this.revocationCache.hits;

    const result = await this.revocationCache.getOrFetch(key, async () => {
      const raw = (await this.simulate("get_revocation_record", [
        this.hexToScVal(certificateId),
      ])) as Record<string, unknown> | null;
      return raw ? this.mapRevocation(raw) : null;
    });

    this.trackCacheResult("revocation", this.revocationCache.hits > beforeHits);
    return result;
  }

  /**
   * Invalidate all caches for a specific certificate (call after issue/revoke).
   */
  invalidateCertificate(certificateId: string): void {
    this.certificateCache.delete(`cert:${certificateId}`);
    this.revocationCache.delete(`revocation:${certificateId}`);
    this.analyticsCache.delete("analytics:global");
  }

  /**
   * Returns a snapshot of cache hit/miss stats for all caches.
   */
  cacheStats() {
    return {
      analytics: this.analyticsCache.stats(),
      certificate: this.certificateCache.stats(),
      revocation: this.revocationCache.stats(),
      student: this.studentCache.stats(),
    };
  }
}

// Singleton
export const contractClient = new CertificateContractClient();
