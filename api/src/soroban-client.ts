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
}

// Singleton
export const contractClient = new CertificateContractClient();
