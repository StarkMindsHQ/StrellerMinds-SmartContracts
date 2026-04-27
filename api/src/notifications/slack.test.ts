/**
 * Tests for the SlackNotifier service
 */
import https from "https";
import { EventEmitter } from "events";
import {
  SlackNotifier,
  createSlackNotifier,
  getSlackNotifier,
} from "./slack";

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeHttpsMock(statusCode: number, body: string) {
  const req = new EventEmitter() as any;
  req.write = jest.fn();
  req.end = jest.fn();
  req.destroy = jest.fn();
  req.setTimeout = jest.fn();

  const res = new EventEmitter() as any;
  res.statusCode = statusCode;

  jest.spyOn(https, "request").mockImplementation((_opts: any, cb: any) => {
    cb(res);
    // Emit response data asynchronously
    setImmediate(() => {
      res.emit("data", body);
      res.emit("end");
    });
    return req;
  });

  return { req, res };
}

afterEach(() => {
  jest.restoreAllMocks();
});

// ── SlackNotifier unit tests ──────────────────────────────────────────────────

describe("SlackNotifier", () => {
  const webhookUrl = "https://hooks.slack.com/services/T000/B000/XXXX";

  describe("notify()", () => {
    it("returns true when Slack responds with 200 ok", async () => {
      makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });
      const result = await notifier.notify({
        type: "custom",
        title: "Test",
        message: "Hello",
      });
      expect(result).toBe(true);
    });

    it("returns false when Slack responds with non-200", async () => {
      makeHttpsMock(500, "error");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });
      const result = await notifier.notify({
        type: "custom",
        title: "Test",
        message: "Hello",
      });
      expect(result).toBe(false);
    });

    it("returns false when Slack responds with 200 but unexpected body", async () => {
      makeHttpsMock(200, "not_ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });
      const result = await notifier.notify({
        type: "custom",
        title: "Test",
        message: "Hello",
      });
      expect(result).toBe(false);
    });

    it("returns false when no default webhook is configured", async () => {
      const notifier = new SlackNotifier({});
      const result = await notifier.notify({
        type: "custom",
        title: "Test",
        message: "Hello",
      });
      expect(result).toBe(false);
    });

    it("sends correct JSON payload to Slack", async () => {
      const { req } = makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });

      await notifier.notify({
        type: "certificate_issued",
        title: "Cert Issued",
        message: "A certificate was issued",
        fields: [{ title: "ID", value: "abc123", short: true }],
      });

      expect(req.write).toHaveBeenCalledTimes(1);
      const payload = JSON.parse(req.write.mock.calls[0][0]);
      expect(payload.text).toBe("Cert Issued");
      expect(payload.icon_emoji).toBe(":mortar_board:");
      expect(payload.attachments[0].color).toBe("good");
      expect(payload.attachments[0].fields[0].title).toBe("ID");
    });

    it("uses correct color for system_alert type", async () => {
      const { req } = makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });

      await notifier.notify({ type: "system_alert", title: "Alert", message: "Down" });

      const payload = JSON.parse(req.write.mock.calls[0][0]);
      expect(payload.attachments[0].color).toBe("danger");
      expect(payload.icon_emoji).toBe(":rotating_light:");
    });

    it("uses correct color for system_warning type", async () => {
      const { req } = makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });

      await notifier.notify({ type: "system_warning", title: "Warn", message: "High load" });

      const payload = JSON.parse(req.write.mock.calls[0][0]);
      expect(payload.attachments[0].color).toBe("warning");
    });

    it("overrides channel when provided", async () => {
      const { req } = makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });

      await notifier.notify({
        type: "custom",
        title: "T",
        message: "M",
        channel: "#custom-channel",
      });

      const payload = JSON.parse(req.write.mock.calls[0][0]);
      expect(payload.channel).toBe("#custom-channel");
    });
  });

  describe("notifyCertificateIssued()", () => {
    it("sends a certificate_issued notification with correct fields", async () => {
      const { req } = makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });

      const result = await notifier.notifyCertificateIssued({
        certificateId: "abc123",
        student: "GABC...",
        courseTitle: "Blockchain 101",
        issuer: "GXYZ...",
      });

      expect(result).toBe(true);
      const payload = JSON.parse(req.write.mock.calls[0][0]);
      const fields = payload.attachments[0].fields as Array<{ title: string; value: string }>;
      expect(fields.find((f) => f.title === "Certificate ID")?.value).toBe("abc123");
      expect(fields.find((f) => f.title === "Course")?.value).toBe("Blockchain 101");
    });
  });

  describe("notifySystemAlert()", () => {
    it("sends a system_alert notification", async () => {
      const { req } = makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });

      await notifier.notifySystemAlert("Service down", "DB unreachable");

      const payload = JSON.parse(req.write.mock.calls[0][0]);
      expect(payload.attachments[0].color).toBe("danger");
      expect(payload.attachments[0].text).toContain("DB unreachable");
    });
  });

  describe("notifySystemWarning()", () => {
    it("sends a system_warning notification", async () => {
      const { req } = makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({ default: { url: webhookUrl } });

      await notifier.notifySystemWarning("High memory usage");

      const payload = JSON.parse(req.write.mock.calls[0][0]);
      expect(payload.attachments[0].color).toBe("warning");
    });
  });

  describe("sendToWebhook()", () => {
    it("sends to a named webhook", async () => {
      makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({
        alerts: { url: webhookUrl, channel: "#alerts" },
      });

      const result = await notifier.sendToWebhook("alerts", {
        text: "Alert message",
      });
      expect(result).toBe(true);
    });

    it("returns false for unknown webhook key", async () => {
      const notifier = new SlackNotifier({});
      const result = await notifier.sendToWebhook("nonexistent", { text: "x" });
      expect(result).toBe(false);
    });
  });

  describe("webhook management", () => {
    it("setWebhook() registers a new webhook", async () => {
      makeHttpsMock(200, "ok");
      const notifier = new SlackNotifier({});
      notifier.setWebhook("new", { url: webhookUrl });

      expect(notifier.listWebhooks()).toContain("new");
    });

    it("removeWebhook() removes an existing webhook", () => {
      const notifier = new SlackNotifier({ toRemove: { url: webhookUrl } });
      expect(notifier.removeWebhook("toRemove")).toBe(true);
      expect(notifier.listWebhooks()).not.toContain("toRemove");
    });

    it("removeWebhook() returns false for unknown key", () => {
      const notifier = new SlackNotifier({});
      expect(notifier.removeWebhook("ghost")).toBe(false);
    });

    it("listWebhooks() returns all registered keys", () => {
      const notifier = new SlackNotifier({
        default: { url: webhookUrl },
        alerts: { url: webhookUrl },
      });
      expect(notifier.listWebhooks()).toEqual(
        expect.arrayContaining(["default", "alerts"])
      );
    });
  });

  describe("error handling", () => {
    it("returns false on network error", async () => {
      const req = new EventEmitter() as any;
      req.write = jest.fn();
      req.end = jest.fn();
      req.destroy = jest.fn();
      req.setTimeout = jest.fn();

      jest.spyOn(https, "request").mockImplementation((_opts: any, _cb: any) => {
        setImmediate(() => req.emit("error", new Error("ECONNREFUSED")));
        return req;
      });

      const notifier = new SlackNotifier({ default: { url: webhookUrl } });
      const result = await notifier.notify({ type: "custom", title: "T", message: "M" });
      expect(result).toBe(false);
    });

    it("returns false for invalid webhook URL", async () => {
      const notifier = new SlackNotifier({ default: { url: "not-a-url" } });
      const result = await notifier.notify({ type: "custom", title: "T", message: "M" });
      expect(result).toBe(false);
    });
  });
});

// ── Singleton factory tests ───────────────────────────────────────────────────

describe("createSlackNotifier / getSlackNotifier", () => {
  it("createSlackNotifier returns a SlackNotifier instance", () => {
    const notifier = createSlackNotifier({
      default: { url: "https://hooks.slack.com/services/T000/B000/XXXX" },
    });
    expect(notifier).toBeInstanceOf(SlackNotifier);
  });

  it("getSlackNotifier returns the singleton created by createSlackNotifier", () => {
    const notifier = createSlackNotifier({
      default: { url: "https://hooks.slack.com/services/T000/B000/XXXX" },
    });
    expect(getSlackNotifier()).toBe(notifier);
  });
});
