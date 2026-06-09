import QRCode from "qrcode";

export interface QrCodeOptions {
  data: string;
  size?: number;
  foregroundColor?: string;
  backgroundColor?: string;
}

export async function generateQrCodeDataUrl(opts: QrCodeOptions): Promise<string> {
  const { data, size = 120, foregroundColor = "#000000", backgroundColor = "#FFFFFF" } = opts;
  return QRCode.toDataURL(data, {
    width: size,
    color: { dark: foregroundColor, light: backgroundColor },
    errorCorrectionLevel: "H",
    margin: 1,
  });
}

export async function generateQrCodeBuffer(data: string, size = 120): Promise<Buffer> {
  return QRCode.toBuffer(data, { width: size, errorCorrectionLevel: "H" });
}
