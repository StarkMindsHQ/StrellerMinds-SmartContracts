/**
 * Redis Pub/Sub bridge for multi-instance WebSocket fan-out
 *
 * When a notification is published, it's pushed to a Redis channel.
 * Every server instance subscribes to that channel and delivers the
 * notification to any locally-connected clients that match the target.
 *
 * Channel layout:
 *   ws:notifications          — broadcast to all users
 *   ws:user:<userId>          — targeted to a specific user
 *   ws:topic:<topicName>      — fan-out to topic subscribers
 */
import Redis from "ioredis";
import { config } from "../config";
import { logger } from "../logger";
import type { PubSubMessage } from "./types";

export type MessageHandler = (message: PubSubMessage) => void;

const CHANNEL_BROADCAST = "ws:notifications";
const CHANNEL_USER_PREFIX = "ws:user:";
const CHANNEL_TOPIC_PREFIX = "ws:topic:";

export class WsPubSub {
  private publisher: Redis;
  private subscriber: Redis;
  private handlers: MessageHandler[] = [];
  private subscribedChannels = new Set<string>();
  private ready = false;

  constructor() {
    const redisOptions = {
      lazyConnect: true,
      maxRetriesPerRequest: 3,
      retryStrategy: (times: number) => {
        if (times > 10) return null;
        return Math.min(times * 200, 3000);
      },
    };

    this.publisher = new Redis(config.redis.url, redisOptions);
    this.subscriber = new Redis(config.redis.url, redisOptions);
  }

  async connect(): Promise<void> {
    await Promise.all([
      this.publisher.connect(),
      this.subscriber.connect(),
    ]);

    this.subscriber.on("message", (channel: string, raw: string) => {
      try {
        const message = JSON.parse(raw) as PubSubMessage;
        for (const handler of this.handlers) {
          handler(message);
        }
      } catch (err) {
        logger.error("WsPubSub: failed to parse message", { channel, error: err });
      }
    });

    this.subscriber.on("error", (err) => {
      logger.error("WsPubSub subscriber error", { error: err });
    });

    this.publisher.on("error", (err) => {
      logger.error("WsPubSub publisher error", { error: err });
    });

    // Always subscribe to the broadcast channel
    await this.subscribeChannel(CHANNEL_BROADCAST);
    this.ready = true;
    logger.info("WsPubSub connected");
  }

  async disconnect(): Promise<void> {
    this.ready = false;
    await Promise.all([
      this.publisher.quit(),
      this.subscriber.quit(),
    ]);
    logger.info("WsPubSub disconnected");
  }

  /** Register a handler that receives all inbound pub/sub messages. */
  onMessage(handler: MessageHandler): void {
    this.handlers.push(handler);
  }

  /**
   * Publish a notification to the appropriate Redis channel(s).
   * Other server instances will receive it and deliver to their local clients.
   */
  async publish(message: PubSubMessage): Promise<void> {
    if (!this.ready) {
      logger.warn("WsPubSub: publish called before ready");
      return;
    }

    const raw = JSON.stringify(message);

    if (message.broadcast) {
      await this.publisher.publish(CHANNEL_BROADCAST, raw);
      return;
    }

    if (message.targetUserId) {
      const channel = `${CHANNEL_USER_PREFIX}${message.targetUserId}`;
      await this.ensureSubscribed(channel);
      await this.publisher.publish(channel, raw);
    }

    if (message.targetTopic) {
      const channel = `${CHANNEL_TOPIC_PREFIX}${message.targetTopic}`;
      await this.ensureSubscribed(channel);
      await this.publisher.publish(channel, raw);
    }
  }

  /**
   * Subscribe to a user-specific channel (called when a user connects).
   */
  async subscribeUser(userId: string): Promise<void> {
    await this.ensureSubscribed(`${CHANNEL_USER_PREFIX}${userId}`);
  }

  /**
   * Subscribe to a topic channel.
   */
  async subscribeTopic(topic: string): Promise<void> {
    await this.ensureSubscribed(`${CHANNEL_TOPIC_PREFIX}${topic}`);
  }

  // ── Internal ────────────────────────────────────────────────────────────────

  private async subscribeChannel(channel: string): Promise<void> {
    await this.subscriber.subscribe(channel);
    this.subscribedChannels.add(channel);
    logger.debug("WsPubSub subscribed", { channel });
  }

  private async ensureSubscribed(channel: string): Promise<void> {
    if (!this.subscribedChannels.has(channel)) {
      await this.subscribeChannel(channel);
    }
  }
}

// Singleton
export const wsPubSub = new WsPubSub();
