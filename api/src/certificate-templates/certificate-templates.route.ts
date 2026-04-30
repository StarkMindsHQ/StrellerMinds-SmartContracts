// @ts-nocheck
import { Router, Request, Response } from "express";
import { randomUUID } from "crypto";
import { sendSuccess, sendError } from "../utils/response";
import { generateQrCodeDataUrl } from "../common/qr-code/qr-code.service";
import { templateStore, ensureLibrarySeeded } from "./certificate-templates.store";
import {
  CreateTemplateSchema,
  UpdateTemplateSchema,
  QueryTemplatesSchema,
  GenerateQrCodeSchema,
  PreviewTemplateSchema,
  GeneratePdfSchema,
} from "./dto/certificate-template.dto";
import { streamCertificatePdf } from "../services/pdfService";
import { logger } from "../logger";

const router = Router();

// Seed library on first request
router.use((_req, _res, next) => { ensureLibrarySeeded(); next(); });

// ── POST /certificate-templates ───────────────────────────────────────────────
router.post("/", (req: Request, res: Response) => {
  const parsed = CreateTemplateSchema.safeParse(req.body);
  if (!parsed.success) {
    return sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);
  }
  const template = templateStore.create({
    ...parsed.data,
    tags: parsed.data.tags ?? [],
    status: "draft",
    isLibraryTemplate: false,
    createdById: (req as any).user?.id,
  });
  return sendSuccess(res, template, 201, req.requestId);
});

// ── GET /certificate-templates ────────────────────────────────────────────────
router.get("/", (req: Request, res: Response) => {
  const parsed = QueryTemplatesSchema.safeParse(req.query);
  if (!parsed.success) {
    return sendError(res, 400, "VALIDATION_ERROR", "Invalid query params", parsed.error.flatten(), req.requestId);
  }
  const result = templateStore.findAll(parsed.data);
  return sendSuccess(res, { ...result, page: parsed.data.page, limit: parsed.data.limit }, 200, req.requestId);
});

// ── GET /certificate-templates/library ───────────────────────────────────────
router.get("/library", (req: Request, res: Response) => {
  const parsed = QueryTemplatesSchema.safeParse(req.query);
  if (!parsed.success) {
    return sendError(res, 400, "VALIDATION_ERROR", "Invalid query params", parsed.error.flatten(), req.requestId);
  }
  const result = templateStore.findAll({ ...parsed.data, libraryOnly: true });
  return sendSuccess(res, result, 200, req.requestId);
});

// ── POST /certificate-templates/qr-code/generate ─────────────────────────────
router.post("/qr-code/generate", async (req: Request, res: Response) => {
  const parsed = GenerateQrCodeSchema.safeParse(req.body);
  if (!parsed.success) {
    return sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);
  }
  try {
    const dataUrl = await generateQrCodeDataUrl(parsed.data);
    return sendSuccess(res, { dataUrl }, 200, req.requestId);
  } catch (err) {
    return sendError(res, 500, "QR_GENERATION_FAILED", "Failed to generate QR code", undefined, req.requestId);
  }
});

// ── GET /certificate-templates/:id ───────────────────────────────────────────
router.get("/:id", (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);
  return sendSuccess(res, template, 200, req.requestId);
});

// ── PATCH /certificate-templates/:id ─────────────────────────────────────────
router.patch("/:id", (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);
  if (template.isLibraryTemplate) return sendError(res, 400, "FORBIDDEN", "Clone a library template before editing", undefined, req.requestId);

  const parsed = UpdateTemplateSchema.safeParse(req.body);
  if (!parsed.success) return sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);

  const updated = templateStore.update(req.params.id, parsed.data);
  return sendSuccess(res, updated, 200, req.requestId);
});

// ── DELETE /certificate-templates/:id ────────────────────────────────────────
router.delete("/:id", (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);
  if (template.isLibraryTemplate) return sendError(res, 400, "FORBIDDEN", "Library templates cannot be deleted", undefined, req.requestId);
  templateStore.delete(req.params.id);
  return sendSuccess(res, null, 204, req.requestId);
});

// ── POST /certificate-templates/:id/clone ────────────────────────────────────
router.post("/:id/clone", (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);

  const { id, createdAt, updatedAt, usageCount, ...rest } = template;
  const clone = templateStore.create({
    ...rest,
    name: `${template.name} (Copy)`,
    isLibraryTemplate: false,
    status: "draft",
    createdById: (req as any).user?.id,
  });
  return sendSuccess(res, clone, 201, req.requestId);
});

// ── PATCH /certificate-templates/:id/publish ─────────────────────────────────
router.patch("/:id/publish", (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);
  const updated = templateStore.update(req.params.id, { status: "published" });
  return sendSuccess(res, updated, 200, req.requestId);
});

// ── PATCH /certificate-templates/:id/archive ─────────────────────────────────
router.patch("/:id/archive", (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);
  const updated = templateStore.update(req.params.id, { status: "archived" });
  return sendSuccess(res, updated, 200, req.requestId);
});

// ── POST /certificate-templates/:id/preview ──────────────────────────────────
router.post("/:id/preview", async (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);

  const parsed = PreviewTemplateSchema.safeParse(req.body);
  if (!parsed.success) return sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);

  const { sampleData, format } = parsed.data;

  if (format === "pdf") {
    // Stream the PDF directly — avoids buffering and the 30 s timeout
    const filename = `${template.name.replace(/[^a-z0-9]/gi, "_")}_preview.pdf`;
    res.setHeader("Content-Type", "application/pdf");
    res.setHeader("Content-Disposition", `inline; filename="${filename}"`);
    res.setHeader("Transfer-Encoding", "chunked");

    try {
      await streamCertificatePdf(res, {
        template,
        data: sampleData as Record<string, string>,
      });
    } catch (err) {
      logger.error("PDF preview generation failed", { templateId: template.id, err });
      if (!res.headersSent) {
        sendError(res, 500, "PDF_GENERATION_FAILED", "Failed to generate PDF preview", undefined, req.requestId);
      } else {
        res.destroy();
      }
    }
    return;
  }

  const resolvedLayout = JSON.parse(
    JSON.stringify(template.layout).replace(/\{\{(\w+)\}\}/g, (_, key) => sampleData[key] ?? `[${key}]`)
  );

  templateStore.incrementUsage(req.params.id);

  const previewUrl = `data:application/json;base64,${Buffer.from(
    JSON.stringify({ layout: resolvedLayout, format })
  ).toString("base64")}`;

  return sendSuccess(res, { previewUrl, format }, 200, req.requestId);
});

// ── POST /certificate-templates/:id/pdf ──────────────────────────────────────
// Dedicated PDF generation endpoint that supports attachments (extra pages).
// Uses streaming so documents with >50 pages never hit the 30 s HTTP timeout.
//
// Request body (all fields optional):
//   data        – template variable values, e.g. { recipientName: "Alice" }
//   attachments – array of { title, content, contentType } extra pages
//   chunkSize   – attachment pages per event-loop tick (default 10, max 50)
router.post("/:id/pdf", async (req: Request, res: Response) => {
  const template = templateStore.findById(req.params.id);
  if (!template) {
    return sendError(res, 404, "NOT_FOUND", `Template ${req.params.id} not found`, undefined, req.requestId);
  }

  const parsed = GeneratePdfSchema.safeParse(req.body);
  if (!parsed.success) {
    return sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);
  }

  const { data, attachments, chunkSize } = parsed.data;
  const totalPages = 1 + attachments.length;

  logger.info("PDF generation started", {
    templateId: template.id,
    templateName: template.name,
    totalPages,
    chunkSize,
    requestId: req.requestId,
  });

  const filename = `${template.name.replace(/[^a-z0-9]/gi, "_")}.pdf`;
  res.setHeader("Content-Type", "application/pdf");
  res.setHeader("Content-Disposition", `attachment; filename="${filename}"`);
  res.setHeader("Transfer-Encoding", "chunked");
  // Tell nginx/CloudFront not to buffer — bytes reach the client immediately
  res.setHeader("X-Accel-Buffering", "no");

  const startMs = Date.now();

  try {
    await streamCertificatePdf(res, {
      template,
      data: data as Record<string, string>,
      attachments,
      chunkSize,
    });

    templateStore.incrementUsage(req.params.id);

    logger.info("PDF generation completed", {
      templateId: template.id,
      totalPages,
      durationMs: Date.now() - startMs,
      requestId: req.requestId,
    });
  } catch (err) {
    logger.error("PDF generation failed", {
      templateId: template.id,
      totalPages,
      durationMs: Date.now() - startMs,
      err,
      requestId: req.requestId,
    });

    if (!res.headersSent) {
      sendError(res, 500, "PDF_GENERATION_FAILED", "Failed to generate PDF", undefined, req.requestId);
    } else {
      // Bytes already flowing — destroy the socket so the client gets an error
      res.destroy(err instanceof Error ? err : new Error(String(err)));
    }
  }
});

export default router;
