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
} from "./dto/certificate-template.dto";

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
  const resolvedLayout = JSON.parse(
    JSON.stringify(template.layout).replace(/\{\{(\w+)\}\}/g, (_, key) => sampleData[key] ?? `[${key}]`)
  );

  templateStore.incrementUsage(req.params.id);

  const previewUrl = `data:application/json;base64,${Buffer.from(
    JSON.stringify({ layout: resolvedLayout, format })
  ).toString("base64")}`;

  return sendSuccess(res, { previewUrl, format }, 200, req.requestId);
});

export default router;
