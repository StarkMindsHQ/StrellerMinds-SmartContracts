import { randomUUID } from "crypto";
import { CertificateTemplate, TemplateLayout } from "./entities/certificate-template.entity";
import { LIBRARY_TEMPLATES } from "./templates/library-templates";

// In-memory store — swap for a real DB (pg, prisma, etc.) later
const store = new Map<string, CertificateTemplate>();

// Seed library templates on startup
let seeded = false;
export function ensureLibrarySeeded(): void {
  if (seeded) return;
  LIBRARY_TEMPLATES.forEach((t) => {
    const id = randomUUID();
    store.set(id, {
      ...t,
      id,
      status: "published",
      usageCount: 0,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });
  });
  seeded = true;
}

export const templateStore = {
  create(data: Omit<CertificateTemplate, "id" | "createdAt" | "updatedAt" | "usageCount">): CertificateTemplate {
    const now = new Date().toISOString();
    const template: CertificateTemplate = {
      ...data,
      id: randomUUID(),
      usageCount: 0,
      createdAt: now,
      updatedAt: now,
    };
    store.set(template.id, template);
    return template;
  },

  findById(id: string): CertificateTemplate | undefined {
    return store.get(id);
  },

  findAll(filters: {
    category?: string;
    status?: string;
    libraryOnly?: boolean;
    search?: string;
    page: number;
    limit: number;
  }): { data: CertificateTemplate[]; total: number } {
    let results = Array.from(store.values());

    if (filters.category) results = results.filter((t) => t.category === filters.category);
    if (filters.status) results = results.filter((t) => t.status === filters.status);
    if (filters.libraryOnly) results = results.filter((t) => t.isLibraryTemplate);
    if (filters.search) {
      const q = filters.search.toLowerCase();
      results = results.filter(
        (t) =>
          t.name.toLowerCase().includes(q) ||
          t.description?.toLowerCase().includes(q)
      );
    }

    results.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    const total = results.length;
    const data = results.slice((filters.page - 1) * filters.limit, filters.page * filters.limit);
    return { data, total };
  },

  update(id: string, patch: Partial<CertificateTemplate>): CertificateTemplate | undefined {
    const existing = store.get(id);
    if (!existing) return undefined;
    const updated = { ...existing, ...patch, id, updatedAt: new Date().toISOString() };
    store.set(id, updated);
    return updated;
  },

  delete(id: string): boolean {
    return store.delete(id);
  },

  incrementUsage(id: string): void {
    const t = store.get(id);
    if (t) store.set(id, { ...t, usageCount: t.usageCount + 1 });
  },
};
