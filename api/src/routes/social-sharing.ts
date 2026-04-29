// @ts-nocheck
/**
 * Social sharing routes for achievements and credentials
 *
 * POST /api/v1/social-sharing                    — share a certificate (auth required)
 * GET  /api/v1/social-sharing/:certificateId     — get shares for a certificate (auth required)
 * GET  /api/v1/social-sharing/user/shares        — get user's shares (auth required)
 * PUT  /api/v1/social-sharing/:certificateId/engagement — update engagement metrics (admin required)
 * GET  /api/v1/social-sharing/analytics          — get social sharing analytics (auth required)
 * GET  /api/v1/social-sharing/certificate/:certificateId/analytics — cert-specific analytics
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendError } from "../utils/response";
import { certificateIdSchema, stellarAddressSchema } from "../utils/validate";
import { logger } from "../logger";
import type {
  ShareRequest,
  ShareResponse,
  SocialSharingAnalytics,
  SharePlatform,
} from "../types";

const router = Router();

// ── POST /social-sharing ────────────────────────────────────────────────────────
// Share a certificate to social media
router.post(
  "/",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const userAddress = req.user?.address;
    if (!userAddress) {
      sendError(res, 401, "UNAUTHORIZED", "User not authenticated", undefined, req.requestId);
      return;
    }

    const { certificateId, platform, customMessage } = req.body as ShareRequest;

    // Validate inputs
    const certParsed = certificateIdSchema.safeParse(certificateId);
    if (!certParsed.success) {
      sendError(
        res,
        400,
        "INVALID_CERTIFICATE_ID",
        "Certificate ID must be a 64-character hex string",
        undefined,
        req.requestId
      );
      return;
    }

    const validPlatforms = ["Twitter", "LinkedIn", "Facebook"];
    if (!validPlatforms.includes(platform)) {
      sendError(
        res,
        400,
        "INVALID_PLATFORM",
        "Platform must be one of: Twitter, LinkedIn, Facebook",
        undefined,
        req.requestId
      );
      return;
    }

    if (!customMessage || customMessage.length === 0 || customMessage.length > 500) {
      sendError(
        res,
        400,
        "INVALID_MESSAGE",
        "Custom message must be between 1 and 500 characters",
        undefined,
        req.requestId
      );
      return;
    }

    try {
      // Call contract to record share
      const shareRecord = await contractClient.shareAchievement(
        userAddress,
        certParsed.data,
        platform as SharePlatform,
        customMessage
      );

      logger.info("Achievement shared", {
        certificateId: certParsed.data,
        user: userAddress,
        platform,
        requestId: req.requestId,
      });

      // Track analytics event
      await contractClient.trackSocialShare(
        userAddress,
        certParsed.data,
        platform,
        customMessage
      );

      const response: ShareResponse = {
        success: true,
        shareRecord,
        engagement: {
          platform: platform as SharePlatform,
          shareUrl: shareRecord.shareUrl,
          engagementTarget: Math.ceil(100 * 0.2), // 20% user engagement target
        },
      };

      sendSuccess(res, response, 201, req.requestId);
    } catch (err) {
      logger.error("Share achievement failed", {
        certificateId: certParsed.data,
        user: userAddress,
        error: err,
        requestId: req.requestId,
      });
      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to record share on blockchain",
        undefined,
        req.requestId
      );
    }
  }
);

// ── GET /social-sharing/:certificateId ──────────────────────────────────────────
// Get all shares for a specific certificate
router.get(
  "/:certificateId",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const { certificateId } = req.params;

    const parsed = certificateIdSchema.safeParse(certificateId);
    if (!parsed.success) {
      sendError(
        res,
        400,
        "INVALID_CERTIFICATE_ID",
        "Certificate ID must be a 64-character hex string",
        undefined,
        req.requestId
      );
      return;
    }

    try {
      const shares = await contractClient.getCertificateShares(parsed.data);

      logger.info("Retrieved certificate shares", {
        certificateId: parsed.data,
        shareCount: shares.length,
        requestId: req.requestId,
      });

      sendSuccess(res, { shares, count: shares.length }, 200, req.requestId);
    } catch (err) {
      logger.error("Get certificate shares failed", {
        certificateId: parsed.data,
        error: err,
        requestId: req.requestId,
      });
      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to query blockchain",
        undefined,
        req.requestId
      );
    }
  }
);

// ── GET /social-sharing/user/shares ─────────────────────────────────────────────
// Get all shares by authenticated user
router.get(
  "/user/shares",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const userAddress = req.user?.address;
    if (!userAddress) {
      sendError(res, 401, "UNAUTHORIZED", "User not authenticated", undefined, req.requestId);
      return;
    }

    try {
      const shares = await contractClient.getUserShares(userAddress);

      logger.info("Retrieved user shares", {
        user: userAddress,
        shareCount: shares.length,
        requestId: req.requestId,
      });

      sendSuccess(res, { shares, count: shares.length }, 200, req.requestId);
    } catch (err) {
      logger.error("Get user shares failed", {
        user: userAddress,
        error: err,
        requestId: req.requestId,
      });
      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to query blockchain",
        undefined,
        req.requestId
      );
    }
  }
);

// ── PUT /social-sharing/:certificateId/engagement ────────────────────────────────
// Update engagement metrics (admin only)
router.put(
  "/:certificateId/engagement",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const userAddress = req.user?.address;
    const isAdmin = req.user?.isAdmin || false;

    if (!isAdmin) {
      sendError(res, 403, "FORBIDDEN", "Admin authorization required", undefined, req.requestId);
      return;
    }

    const { certificateId } = req.params;
    const { user, platform, engagementCount } = req.body;

    const certParsed = certificateIdSchema.safeParse(certificateId);
    if (!certParsed.success) {
      sendError(
        res,
        400,
        "INVALID_CERTIFICATE_ID",
        "Certificate ID must be a 64-character hex string",
        undefined,
        req.requestId
      );
      return;
    }

    const userParsed = stellarAddressSchema.safeParse(user);
    if (!userParsed.success) {
      sendError(res, 400, "INVALID_USER", "Invalid Stellar address", undefined, req.requestId);
      return;
    }

    if (typeof engagementCount !== "number" || engagementCount < 0) {
      sendError(
        res,
        400,
        "INVALID_ENGAGEMENT",
        "Engagement count must be a non-negative number",
        undefined,
        req.requestId
      );
      return;
    }

    try {
      await contractClient.updateEngagement(
        userAddress!,
        certParsed.data,
        userParsed.data,
        platform,
        engagementCount
      );

      logger.info("Engagement updated", {
        certificateId: certParsed.data,
        user: userParsed.data,
        platform,
        engagementCount,
        requestId: req.requestId,
      });

      sendSuccess(res, { success: true, engagementCount }, 200, req.requestId);
    } catch (err) {
      logger.error("Update engagement failed", {
        certificateId: certParsed.data,
        error: err,
        requestId: req.requestId,
      });
      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to update engagement",
        undefined,
        req.requestId
      );
    }
  }
);

// ── GET /social-sharing/analytics ───────────────────────────────────────────────
// Get global social sharing analytics
router.get(
  "/analytics",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const analytics = await contractClient.getSocialSharingAnalytics();

      // Calculate engagement percentage
      const engagementPercentage =
        analytics.uniqueSharers > 0
          ? (analytics.averageEngagement / analytics.uniqueSharers) * 100
          : 0;

      logger.info("Retrieved social sharing analytics", {
        totalShares: analytics.totalShares,
        engagementPercentage,
        requestId: req.requestId,
      });

      sendSuccess(res, { ...analytics, engagementPercentage }, 200, req.requestId);
    } catch (err) {
      logger.error("Get analytics failed", { error: err, requestId: req.requestId });
      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to query analytics",
        undefined,
        req.requestId
      );
    }
  }
);

// ── GET /social-sharing/certificate/:certificateId/analytics ───────────────────
// Get analytics for a specific certificate
router.get(
  "/certificate/:certificateId/analytics",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const { certificateId } = req.params;

    const parsed = certificateIdSchema.safeParse(certificateId);
    if (!parsed.success) {
      sendError(
        res,
        400,
        "INVALID_CERTIFICATE_ID",
        "Certificate ID must be a 64-character hex string",
        undefined,
        req.requestId
      );
      return;
    }

    try {
      const analytics = await contractClient.getCertificateSocialAnalytics(parsed.data);

      logger.info("Retrieved certificate social analytics", {
        certificateId: parsed.data,
        totalShares: analytics.totalShares,
        requestId: req.requestId,
      });

      sendSuccess(res, analytics, 200, req.requestId);
    } catch (err) {
      logger.error("Get certificate analytics failed", {
        certificateId: parsed.data,
        error: err,
        requestId: req.requestId,
      });
      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to query analytics",
        undefined,
        req.requestId
      );
    }
  }
);

export default router;
