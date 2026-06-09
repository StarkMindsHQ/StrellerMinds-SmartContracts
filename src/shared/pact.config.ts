import * as path from 'path';

export const PACT_CONFIG = {
  LOG_DIR: path.resolve(process.cwd(), 'pact/logs'),
  PACT_DIR: path.resolve(process.cwd(), 'pact/contracts'),
  BROKER_URL: process.env.PACT_BROKER_URL || 'http://localhost:9292',
  BROKER_TOKEN: process.env.PACT_BROKER_TOKEN || '',
  CONSUMER_VERSION: process.env.CONSUMER_VERSION || '1.0.0',
  GIT_BRANCH: process.env.GIT_BRANCH || 'main',
} as const;

export const SERVICES = {
  USER_SERVICE: 'UserService',
  AUTH_SERVICE: 'AuthService',
  NOTIFICATION_SERVICE: 'NotificationService',
  API_GATEWAY: 'ApiGateway',
} as const;

export const PROVIDER_URLS = {
  [SERVICES.USER_SERVICE]:
    process.env.USER_SERVICE_URL || 'http://localhost:3001',
  [SERVICES.AUTH_SERVICE]:
    process.env.AUTH_SERVICE_URL || 'http://localhost:3002',
  [SERVICES.NOTIFICATION_SERVICE]:
    process.env.NOTIFICATION_SERVICE_URL || 'http://localhost:3003',
} as const;
