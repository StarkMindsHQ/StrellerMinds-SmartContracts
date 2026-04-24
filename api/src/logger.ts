import winston from "winston";
import { config } from "./config";

export const logger = winston.createLogger({
  level: config.nodeEnv === "production" ? "info" : "debug",
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    config.nodeEnv === "production"
      ? winston.format.json()
      : winston.format.prettyPrint()
  ),
  defaultMeta: { service: "certificate-verification-api" },
  transports: [new winston.transports.Console()],
});
