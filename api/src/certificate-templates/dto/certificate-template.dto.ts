import { z } from "zod";

export const TemplateCategoryEnum = z.enum([
  "completion",
  "achievement",
  "excellence",
  "participation",
  "professional",
]);

export const TemplateStatusEnum = z.enum(["draft", "published", "archived"]);

export const TemplateElementSchema = z.object({
  id: z.string(),
  type: z.enum(["text", "image", "qrcode", "shape", "signature"]),
  x: z.number().min(0),
  y: z.number().min(0),
  width: z.number().min(1),
  height: z.number().min(1),
  rotation: z.number().min(-360).max(360).optional(),
  zIndex: z.number().min(0).optional(),
  properties: z.record(z.unknown()),
});

export const TemplateLayoutSchema = z.object({
  width: z.number().min(50).max(1000),
  height: z.number().min(50).max(1000),
  orientation: z.enum(["landscape", "portrait"]),
  background: z.object({
    type: z.enum(["color", "image", "gradient"]),
    value: z.string(),
  }),
  elements: z.array(TemplateElementSchema),
});

export const CreateTemplateSchema = z.object({
  name: z.string().min(1).max(120),
  description: z.string().max(500).optional(),
  category: TemplateCategoryEnum,
  layout: TemplateLayoutSchema,
  tags: z.array(z.string()).optional(),
  qrCodeEnabled: z.boolean().optional().default(true),
  printOptimized: z.boolean().optional().default(true),
});

export const UpdateTemplateSchema = CreateTemplateSchema.partial().extend({
  status: TemplateStatusEnum.optional(),
});

export const QueryTemplatesSchema = z.object({
  category: TemplateCategoryEnum.optional(),
  status: TemplateStatusEnum.optional(),
  libraryOnly: z
    .string()
    .optional()
    .transform((v) => v === "true"),
  search: z.string().optional(),
  page: z.coerce.number().min(1).optional().default(1),
  limit: z.coerce.number().min(1).max(100).optional().default(20),
});

export const GenerateQrCodeSchema = z.object({
  data: z.string().min(1),
  size: z.number().min(50).max(500).optional(),
  foregroundColor: z.string().optional(),
  backgroundColor: z.string().optional(),
});

export const PreviewTemplateSchema = z.object({
  format: z.enum(["png", "pdf", "svg"]).optional().default("png"),
  sampleData: z
    .object({
      recipientName: z.string().optional(),
      courseName: z.string().optional(),
      completionDate: z.string().optional(),
      instructorName: z.string().optional(),
    })
    .catchall(z.string().optional()),
});

export const AttachmentSchema = z.object({
  title: z.string().min(1).max(200),
  content: z.string().min(1),
  contentType: z.enum(["text", "image"]).default("text"),
});

export const GeneratePdfSchema = z.object({
  data: z
    .object({
      recipientName: z.string().optional(),
      courseName: z.string().optional(),
      completionDate: z.string().optional(),
      instructorName: z.string().optional(),
    })
    .catchall(z.string().optional())
    .default({}),
  attachments: z.array(AttachmentSchema).optional().default([]),
  /**
   * Number of attachment pages rendered per event-loop tick.
   * Increase for faster generation on powerful servers; decrease to keep
   * the server more responsive under concurrent load.
   * Default: 10.  Max: 50.
   */
  chunkSize: z.coerce.number().min(1).max(50).optional().default(10),
});

export type CreateTemplateDto = z.infer<typeof CreateTemplateSchema>;
export type UpdateTemplateDto = z.infer<typeof UpdateTemplateSchema>;
export type QueryTemplatesDto = z.infer<typeof QueryTemplatesSchema>;
export type GenerateQrCodeDto = z.infer<typeof GenerateQrCodeSchema>;
export type PreviewTemplateDto = z.infer<typeof PreviewTemplateSchema>;
export type GeneratePdfDto = z.infer<typeof GeneratePdfSchema>;
