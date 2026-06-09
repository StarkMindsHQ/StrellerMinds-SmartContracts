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

/**
 * Verification level validation
 */
export const verificationLevelSchema = z.enum(["basic", "enhanced", "comprehensive"], {
  errorMap: (issue, ctx) => {
    if (issue.code === z.ZodIssueCode.invalid_enum_value) {
      return { message: "Verification level must be 'basic', 'enhanced', or 'comprehensive'" };
    }
    return { message: ctx.defaultError };
  }
});

/**
 * Employer verification request schema
 */
export const employerVerificationSchema = z.object({
  certificateId: certificateIdSchema,
  studentAddress: stellarAddressSchema,
  verificationLevel: verificationLevelSchema.optional().default("basic"),
  includeMetadata: z.boolean().optional().default(false),
});

/**
 * Batch verification item schema
 */
export const batchVerificationItemSchema = z.object({
  certificateId: certificateIdSchema,
  studentAddress: stellarAddressSchema,
});

/**
 * Batch verification request schema
 */
export const batchVerificationSchema = z.object({
  verifications: z.array(batchVerificationItemSchema)
    .min(1, "At least one verification item required")
    .max(50, "Maximum 50 verification items per batch"),
  verificationLevel: verificationLevelSchema.optional().default("basic"),
  includeMetadata: z.boolean().optional().default(false),
});

/**
 * Pagination parameters schema
 */
export const paginationSchema = z.object({
  limit: z.coerce.number()
    .int("Limit must be an integer")
    .min(1, "Limit must be at least 1")
    .max(1000, "Limit cannot exceed 1000")
    .optional()
    .default(100),
  offset: z.coerce.number()
    .int("Offset must be an integer")
    .min(0, "Offset must be non-negative")
    .optional()
    .default(0),
});

/**
 * Date range schema for analytics
 */
export const dateRangeSchema = z.object({
  startDate: z.string().datetime("Invalid start date format").optional(),
  endDate: z.string().datetime("Invalid end date format").optional(),
}).refine(
  (data) => {
    if (data.startDate && data.endDate) {
      return new Date(data.startDate) <= new Date(data.endDate);
    }
    return true;
  },
  {
    message: "Start date must be before or equal to end date",
    path: ["endDate"]
  }
);

/**
 * Validate email format
 */
export const emailSchema = z.string()
  .email("Invalid email format")
  .transform(val => val.toLowerCase());

/**
 * Validate organization ID
 */
export const organizationIdSchema = z.string()
  .min(1, "Organization ID cannot be empty")
  .max(100, "Organization ID too long")
  .regex(/^[a-zA-Z0-9_-]+$/, "Organization ID can only contain letters, numbers, hyphens, and underscores");

/**
 * Format validation error details for API responses
 */
export function formatValidationError(error: z.ZodError): {
  field: string;
  message: string;
  code: string;
}[] {
  return error.errors.map(err => ({
    field: err.path.join('.'),
    message: err.message,
    code: err.code,
  }));
}
