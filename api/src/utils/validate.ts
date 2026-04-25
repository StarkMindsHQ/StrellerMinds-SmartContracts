import { z } from "zod";

/** Validates a 32-byte hex certificate ID (64 hex chars, optional 0x prefix) */
export const certificateIdSchema = z
  .string()
  .regex(/^(0x)?[0-9a-fA-F]{64}$/, "Certificate ID must be a 64-character hex string");

/** Validates a Stellar address (G... public key, 56 chars) */
export const stellarAddressSchema = z
  .string()
  .regex(/^G[A-Z2-7]{55}$/, "Invalid Stellar address");

export function normalizeCertId(id: string): string {
  return id.replace(/^0x/, "").toLowerCase();
}
