import Redis from "ioredis";
import { config } from "./config";
import { logger } from "./logger";

class RedisCache {
  private client: Redis | null = null;
  private connected = false;

  connect(): void {
    if (this.client) return;

    this.client = new Redis(config.redis.url, {
      lazyConnect: true,
      maxRetriesPerRequest: 3,
      enableReadyCheck: true,
      retryStrategy: (times) => {
        if (times > 5) return null; // stop retrying after 5 attempts
        return Math.min(times * 200, 2000);
      },
    });

    this.client.on("connect", () => {
      this.connected = true;
      logger.info("Redis connected");
    });

    this.client.on("error", (err) => {
      this.connected = false;
      logger.error("Redis error", { error: err });
    });

    this.client.on("close", () => {
      this.connected = false;
      logger.warn("Redis connection closed");
    });

    this.client.connect().catch((err) => {
      logger.error("Redis initial connect failed", { error: err });
    });
  }

  isConnected(): boolean {
    return this.connected;
  }

  async get<T>(key: string): Promise<T | null> {
    if (!this.client || !this.connected) return null;
    try {
      const val = await this.client.get(key);
      return val ? (JSON.parse(val) as T) : null;
    } catch (err) {
      logger.error("Redis get failed", { key, error: err });
      return null;
    }
  }

  async set(key: string, value: unknown, ttlSeconds: number): Promise<void> {
    if (!this.client || !this.connected) return;
    try {
      await this.client.set(key, JSON.stringify(value), "EX", ttlSeconds);
    } catch (err) {
      logger.error("Redis set failed", { key, error: err });
    }
  }

  async del(...keys: string[]): Promise<void> {
    if (!this.client || !this.connected) return;
    try {
      await this.client.del(...keys);
    } catch (err) {
      logger.error("Redis del failed", { keys, error: err });
    }
  }

  async disconnect(): Promise<void> {
    if (this.client) {
      await this.client.quit();
      this.client = null;
      this.connected = false;
    }
  }
}

export const cache = new RedisCache();
