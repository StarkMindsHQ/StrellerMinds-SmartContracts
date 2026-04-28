export type TemplateStatus = "draft" | "published" | "archived";
export type TemplateCategory =
  | "completion"
  | "achievement"
  | "excellence"
  | "participation"
  | "professional";

export interface TemplateElement {
  id: string;
  type: "text" | "image" | "qrcode" | "shape" | "signature";
  x: number;
  y: number;
  width: number;
  height: number;
  rotation?: number;
  zIndex?: number;
  properties: Record<string, unknown>;
}

export interface TemplateLayout {
  width: number;
  height: number;
  orientation: "landscape" | "portrait";
  background: {
    type: "color" | "image" | "gradient";
    value: string;
  };
  elements: TemplateElement[];
}

export interface CertificateTemplate {
  id: string;
  name: string;
  description?: string;
  category: TemplateCategory;
  status: TemplateStatus;
  layout: TemplateLayout;
  thumbnailUrl?: string;
  isLibraryTemplate: boolean;
  createdById?: string;
  usageCount: number;
  tags: string[];
  qrCodeEnabled: boolean;
  printOptimized: boolean;
  createdAt: string;
  updatedAt: string;
}
