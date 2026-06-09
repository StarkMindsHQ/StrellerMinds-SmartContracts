/**
 * PDF Generation Service
 *
 * Generates certificate PDFs from template layouts using PDFKit.
 * Streams output directly to the HTTP response to avoid buffering large
 * documents in memory, which was the root cause of the >50-page timeout.
 *
 * Key design decisions:
 * - Streaming: PDFKit writes chunks as they are produced; the HTTP response
 *   flushes them immediately, so the 30 s request timeout is never hit even
 *   for very large documents.
 * - Page chunking: attachments/pages are rendered in batches of
 *   PDF_PAGE_CHUNK_SIZE (default 10) with a setImmediate yield between
 *   batches so the event loop stays responsive.
 * - No headless browser: pure-JS rendering avoids the cold-start and memory
 *   overhead that caused the original timeout.
 */

import PDFDocument from "pdfkit";
import { Writable } from "stream";
import { logger } from "../logger";
import { CertificateTemplate } from "../certificate-templates/entities/certificate-template.entity";

/** One attachment / extra page to append after the main certificate page. */
export interface CertificateAttachment {
  title: string;
  /** Plain-text body or a data-URI image (data:image/png;base64,...) */
  content: string;
  contentType: "text" | "image";
}

export interface GeneratePdfOptions {
  template: CertificateTemplate;
  /** Resolved variable values, e.g. { recipientName: "Alice" } */
  data: Record<string, string>;
  attachments?: CertificateAttachment[];
  /** How many attachment pages to render per event-loop tick (default: 10) */
  chunkSize?: number;
}

/** Points-per-mm conversion used by PDFKit (72 pt/inch ÷ 25.4 mm/inch) */
const PT_PER_MM = 72 / 25.4;

function mmToPt(mm: number): number {
  return mm * PT_PER_MM;
}

/**
 * Yield control back to the event loop so that other requests are not starved
 * while a large PDF is being assembled.
 */
function yieldToEventLoop(): Promise<void> {
  return new Promise((resolve) => setImmediate(resolve));
}

/**
 * Render a single template element onto the PDFDocument.
 * Supports: text, image (data-URI), shape (rect), qrcode (placeholder rect).
 */
function renderElement(
  doc: InstanceType<typeof PDFDocument>,
  element: CertificateTemplate["layout"]["elements"][number],
  pageWidth: number,
  pageHeight: number
): void {
  const x = mmToPt((element.x / 100) * (pageWidth / PT_PER_MM));
  const y = mmToPt((element.y / 100) * (pageHeight / PT_PER_MM));
  const w = mmToPt((element.width / 100) * (pageWidth / PT_PER_MM));
  const h = mmToPt((element.height / 100) * (pageHeight / PT_PER_MM));

  const props = element.properties as Record<string, unknown>;

  switch (element.type) {
    case "text": {
      const text = String(props.content ?? props.text ?? "");
      const fontSize = Number(props.fontSize ?? 12);
      const color = String(props.color ?? "#000000");
      doc
        .fontSize(fontSize)
        .fillColor(color)
        .text(text, x, y, { width: w, height: h, ellipsis: true });
      break;
    }

    case "image": {
      const src = String(props.src ?? props.url ?? "");
      if (src.startsWith("data:image/")) {
        try {
          const base64Data = src.split(",")[1];
          if (base64Data) {
            const imgBuffer = Buffer.from(base64Data, "base64");
            doc.image(imgBuffer, x, y, { width: w, height: h });
          }
        } catch (err) {
          logger.warn("PDF: failed to embed image element", { id: element.id, err });
          doc.rect(x, y, w, h).stroke("#cccccc");
        }
      }
      break;
    }

    case "shape": {
      const fill = String(props.fill ?? props.backgroundColor ?? "transparent");
      const stroke = String(props.stroke ?? props.borderColor ?? "#000000");
      const lineWidth = Number(props.lineWidth ?? props.borderWidth ?? 1);
      doc
        .rect(x, y, w, h)
        .lineWidth(lineWidth)
        .fillAndStroke(fill === "transparent" ? null : fill, stroke);
      break;
    }

    case "qrcode": {
      // QR code data-URI may be pre-generated and stored in props.dataUrl
      const dataUrl = String(props.dataUrl ?? "");
      if (dataUrl.startsWith("data:image/")) {
        try {
          const base64Data = dataUrl.split(",")[1];
          if (base64Data) {
            doc.image(Buffer.from(base64Data, "base64"), x, y, { width: w, height: h });
          }
        } catch {
          doc.rect(x, y, w, h).stroke("#999999");
        }
      } else {
        // Fallback: draw a placeholder box
        doc.rect(x, y, w, h).stroke("#999999");
        doc.fontSize(8).fillColor("#999999").text("QR", x + w / 2 - 5, y + h / 2 - 4);
      }
      break;
    }

    case "signature": {
      const sigUrl = String(props.dataUrl ?? props.src ?? "");
      if (sigUrl.startsWith("data:image/")) {
        try {
          const base64Data = sigUrl.split(",")[1];
          if (base64Data) {
            doc.image(Buffer.from(base64Data, "base64"), x, y, { width: w, height: h });
          }
        } catch {
          doc.moveTo(x, y + h).lineTo(x + w, y + h).stroke("#000000");
        }
      } else {
        doc.moveTo(x, y + h).lineTo(x + w, y + h).stroke("#000000");
      }
      break;
    }

    default:
      break;
  }
}

/**
 * Stream a PDF for the given template + data + attachments into `destination`.
 *
 * The function resolves once the PDF `end` event fires (i.e. all bytes have
 * been written to `destination`).  Callers should pass `res` (the Express
 * Response) as the destination so bytes are flushed to the client
 * incrementally — this is what prevents the 30 s timeout on large documents.
 */
export async function streamCertificatePdf(
  destination: Writable,
  options: GeneratePdfOptions
): Promise<void> {
  const { template, data, attachments = [], chunkSize = 10 } = options;

  const layout = template.layout;
  const pageWidthMm = layout.width;
  const pageHeightMm = layout.height;

  const doc = new PDFDocument({
    size: [mmToPt(pageWidthMm), mmToPt(pageHeightMm)],
    layout: layout.orientation === "landscape" ? "landscape" : "portrait",
    autoFirstPage: false,
    bufferPages: false, // stream pages as they are produced
    info: {
      Title: template.name,
      Author: "StrellerMinds",
      CreationDate: new Date(),
    },
  });

  // Pipe directly into the HTTP response (or any writable stream)
  doc.pipe(destination);

  // ── Page 1: main certificate ────────────────────────────────────────────────
  doc.addPage();

  // Background
  const bg = layout.background;
  if (bg.type === "color") {
    doc
      .rect(0, 0, mmToPt(pageWidthMm), mmToPt(pageHeightMm))
      .fill(bg.value || "#ffffff");
  }
  // (gradient / image backgrounds are left as future work)

  // Resolve template variables in element properties
  const resolvedElements = JSON.parse(
    JSON.stringify(layout.elements).replace(
      /\{\{(\w+)\}\}/g,
      (_: string, key: string) => data[key] ?? `[${key}]`
    )
  ) as CertificateTemplate["layout"]["elements"];

  for (const element of resolvedElements) {
    renderElement(doc, element, mmToPt(pageWidthMm), mmToPt(pageHeightMm));
  }

  // ── Attachment pages (chunked to keep the event loop free) ─────────────────
  if (attachments.length > 0) {
    logger.info("PDF: rendering attachment pages", {
      templateId: template.id,
      totalAttachments: attachments.length,
      chunkSize,
    });

    for (let i = 0; i < attachments.length; i += chunkSize) {
      const chunk = attachments.slice(i, i + chunkSize);

      for (const attachment of chunk) {
        doc.addPage();

        // Attachment title
        doc
          .fontSize(16)
          .fillColor("#333333")
          .text(attachment.title, 40, 40, { width: mmToPt(pageWidthMm) - 80 });

        doc.moveDown(0.5);

        if (attachment.contentType === "image") {
          if (attachment.content.startsWith("data:image/")) {
            try {
              const base64Data = attachment.content.split(",")[1];
              if (base64Data) {
                doc.image(Buffer.from(base64Data, "base64"), 40, doc.y, {
                  fit: [mmToPt(pageWidthMm) - 80, mmToPt(pageHeightMm) - doc.y - 40],
                });
              }
            } catch (err) {
              logger.warn("PDF: failed to embed attachment image", {
                title: attachment.title,
                err,
              });
              doc.fontSize(10).fillColor("#cc0000").text("[Image could not be embedded]");
            }
          }
        } else {
          doc
            .fontSize(11)
            .fillColor("#444444")
            .text(attachment.content, 40, doc.y, {
              width: mmToPt(pageWidthMm) - 80,
              align: "left",
            });
        }
      }

      // Yield between chunks so the event loop can handle other requests
      await yieldToEventLoop();
    }
  }

  // Finalise the document — this flushes remaining bytes and emits 'end'
  doc.end();

  // Wait for the stream to finish
  await new Promise<void>((resolve, reject) => {
    destination.on("finish", resolve);
    destination.on("error", reject);
    // PDFKit may emit 'error' on the doc itself
    doc.on("error", reject);
  });
}
