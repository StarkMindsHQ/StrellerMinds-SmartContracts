/**
 * Slack notification service
 *
 * Sends messages to Slack via incoming webhooks.
 * Supports certificate issuance notifications, system alerts,
 * custom webhooks, and channel-based routing.
 */
import https from "https";
import { URL } from "url";
import { logger } from "../logger";

export type SlackNotificationType = "certificate_issued" | "system_alert" | "system_warning" | "custom";

export interface SlackMessage {
  text: string;
  channel?: string; // override default channel
  username?: string;
  icon_emoji?: string;
  attachments?: SlackAttachment[];
  blocks?: unknown[];
}

export interface SlackAttachment {
  color?: string; // "good" | "warning" | "danger" | hex
  title?: string;
  text?: string;
  fields?: Array<{ title: string; value: string; short?: boolean }>;
  footer?: string;
  ts?: number;
}

export interface SlackWebhookConfig {
  url: string;
  channel?: string;
  username?: string;
}

export interface NotifyOptions {
  type: SlackNotificationType;
  title: string;
  message: string;
  fields?: Array<{ title: string; value: string; short?: boolean }>;
  channel?: string; // override channel routing
}

// Channel routing by notification type
const CHANNEL_ROUTING: Record<SlackNotificationType, string | undefined> = {
  certificate_issued: undefined, // falls back to webhook default
  system_alert: undefined,
  system_warning: undefined,
  custom: undefined,
};

const COLOR_MAP: Record<SlackNotificationType, string> = {
  certificate_issued: "good",
  system_alert: "danger",
  system_warning: "warning",
  custom: "#439FE0",
};

const ICON_MAP: Record<SlackNotificationType, string> = {
  certificate_issued: ":mortar_board:",
  system_alert: ":rotating_light:",
  system_warning: ":warning:",
  custom: ":bell:",
};

export class SlackNotifier {
  private webhooks: Map<string, SlackWebhookConfig>;
  private defaultWebhookKey: string;

  constructor(webhooks: Record<string, SlackWebhookConfig>, defaultKey = "default") {
    this.webhooks = new Map(Object.entries(webhooks));
    this.defaultWebhookKey = defaultKey;
  }

  /** Send a structured notification. Returns true on success. */
  async notify(options: NotifyOptions): Promise<boolean> {
    const webhook = this.webhooks.get(this.defaultWebhookKey);
    if (!webhook) {
      logger.warn("Slack notifier: no default webhook configured");
      return false;
    }

    const channel =
      options.channel ??
      CHANNEL_ROUTING[options.type] ??
      webhook.channel;

    const payload: SlackMessage = {
      text: options.title,
      ...(channel ? { channel } : {}),
      username: webhook.username ?? "StrellerMinds",
      icon_emoji: ICON_MAP[options.type],
      attachments: [
        {
          color: COLOR_MAP[options.type],
          text: options.message,
          fields: options.fields,
          footer: "StrellerMinds API",
          ts: Math.floor(Date.now() / 1000),
        },
      ],
    };

    return this.send(webhook.url, payload);
  }

  /** Send a raw Slack message to a named webhook. */
  async sendToWebhook(webhookKey: string, message: SlackMessage): Promise<boolean> {
    const webhook = this.webhooks.get(webhookKey);
    if (!webhook) {
      logger.warn("Slack notifier: webhook not found", { webhookKey });
      return false;
    }
    return this.send(webhook.url, message);
  }

  /** Register or update a webhook at runtime. */
  setWebhook(key: string, config: SlackWebhookConfig): void {
    this.webhooks.set(key, config);
  }

  /** Remove a webhook. */
  removeWebhook(key: string): boolean {
    return this.webhooks.delete(key);
  }

  listWebhooks(): string[] {
    return Array.from(this.webhooks.keys());
  }

  // ── Convenience helpers ──────────────────────────────────────────────────

  async notifyCertificateIssued(params: {
    certificateId: string;
    student: string;
    courseTitle: string;
    issuer: string;
    channel?: string;
  }): Promise<boolean> {
    return this.notify({
      type: "certificate_issued",
      title: ":mortar_board: Certificate Issued",
      message: `A new certificate has been issued on the StrellerMinds platform.`,
      fields: [
        { title: "Certificate ID", value: params.certificateId, short: true },
        { title: "Course", value: params.courseTitle, short: true },
        { title: "Student", value: params.student, short: true },
        { title: "Issuer", value: params.issuer, short: true },
      ],
      channel: params.channel,
    });
  }

  async notifySystemAlert(message: string, details?: string, channel?: string): Promise<boolean> {
    return this.notify({
      type: "system_alert",
      title: ":rotating_light: System Alert",
      message: details ? `${message}\n${details}` : message,
      channel,
    });
  }

  async notifySystemWarning(message: string, channel?: string): Promise<boolean> {
    return this.notify({
      type: "system_warning",
      title: ":warning: System Warning",
      message,
      channel,
    });
  }

  // ── Internal HTTP POST ───────────────────────────────────────────────────

  private send(webhookUrl: string, payload: SlackMessage): Promise<boolean> {
    return new Promise((resolve) => {
      let url: URL;
      try {
        url = new URL(webhookUrl);
      } catch {
        logger.error("Slack notifier: invalid webhook URL", { webhookUrl });
        resolve(false);
        return;
      }

      const body = JSON.stringify(payload);
      const options = {
        hostname: url.hostname,
        path: url.pathname + url.search,
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(body),
        },
      };

      const req = https.request(options, (res) => {
        let data = "";
        res.on("data", (chunk) => { data += chunk; });
        res.on("end", () => {
          if (res.statusCode === 200 && data === "ok") {
            resolve(true);
          } else {
            logger.warn("Slack notifier: unexpected response", {
              statusCode: res.statusCode,
              body: data,
            });
            resolve(false);
          }
        });
      });

      req.on("error", (err) => {
        logger.error("Slack notifier: request failed", { error: err });
        resolve(false);
      });

      req.setTimeout(5000, () => {
        req.destroy();
        logger.warn("Slack notifier: request timed out");
        resolve(false);
      });

      req.write(body);
      req.end();
    });
  }
}

// ── Singleton factory ────────────────────────────────────────────────────────

let _instance: SlackNotifier | null = null;

export function createSlackNotifier(
  webhooks: Record<string, SlackWebhookConfig>
): SlackNotifier {
  _instance = new SlackNotifier(webhooks);
  return _instance;
}

export function getSlackNotifier(): SlackNotifier | null {
  return _instance;
}
