import { Pact, Matchers } from '@pact-foundation/pact';
import * as path from 'path';
import { PACT_CONFIG, SERVICES } from '../../../src/shared/pact.config';
import { ValidateTokenResponseDto, AuthResponseDto } from '../../../src/shared/dto/contract.dto';

const { like, term } = Matchers;

const provider = new Pact({
  consumer: SERVICES.USER_SERVICE,
  provider: SERVICES.AUTH_SERVICE,
  port: 4002,
  log: path.join(PACT_CONFIG.LOG_DIR, 'user-service-auth-service.log'),
  dir: PACT_CONFIG.PACT_DIR,
  logLevel: 'warn',
});

async function validateToken(baseUrl: string, token: string): Promise<ValidateTokenResponseDto> {
  const res = await fetch(`${baseUrl}/auth/validate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ token }),
  });
  return res.json() as Promise<ValidateTokenResponseDto>;
}

async function login(baseUrl: string, email: string, password: string): Promise<AuthResponseDto> {
  const res = await fetch(`${baseUrl}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<AuthResponseDto>;
}

describe('UserService → AuthService (Consumer Contract)', () => {
  beforeAll(() => provider.setup());
  afterEach(() => provider.verify());
  afterAll(() => provider.finalize());

  describe('POST /auth/validate (valid token)', () => {
    const validToken = 'eyJhbGciOiJIUzI1NiJ9.valid.signature';

    beforeEach(() =>
      provider.addInteraction({
        state: 'a valid JWT token exists for user user-abc-123',
        uponReceiving: 'a request to validate a valid token',
        withRequest: {
          method: 'POST',
          path: '/auth/validate',
          headers: { 'Content-Type': 'application/json' },
          body: { token: like(validToken) },
        },
        willRespondWith: {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
          body: { valid: true, userId: like('user-abc-123'), roles: like(['user']) },
        },
      }),
    );

    it('returns valid=true with userId and roles', async () => {
      const result = await validateToken('http://localhost:4002', validToken);
      expect(result.valid).toBe(true);
      expect(result.userId).toBeDefined();
    });
  });

  describe('POST /auth/validate (expired token)', () => {
    const expiredToken = 'eyJhbGciOiJIUzI1NiJ9.expired.signature';

    beforeEach(() =>
      provider.addInteraction({
        state: 'an expired JWT token',
        uponReceiving: 'a request to validate an expired token',
        withRequest: {
          method: 'POST',
          path: '/auth/validate',
          headers: { 'Content-Type': 'application/json' },
          body: { token: like(expiredToken) },
        },
        willRespondWith: {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
          body: { valid: false },
        },
      }),
    );

    it('returns valid=false for expired token', async () => {
      const result = await validateToken('http://localhost:4002', expiredToken);
      expect(result.valid).toBe(false);
    });
  });

  describe('POST /auth/login', () => {
    beforeEach(() =>
      provider.addInteraction({
        state: 'a user with email user@example.com exists with correct password',
        uponReceiving: 'a request to login with valid credentials',
        withRequest: {
          method: 'POST',
          path: '/auth/login',
          headers: { 'Content-Type': 'application/json' },
          body: { email: like('user@example.com'), password: like('password123') },
        },
        willRespondWith: {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
          body: {
            success: true,
            data: {
              accessToken: term({
                generate: 'eyJhbGciOiJIUzI1NiJ9.access.token',
                matcher: '^[A-Za-z0-9-_]+\\.[A-Za-z0-9-_]+\\.[A-Za-z0-9-_]+$',
              }),
              refreshToken: term({
                generate: 'eyJhbGciOiJIUzI1NiJ9.refresh.token',
                matcher: '^[A-Za-z0-9-_]+\\.[A-Za-z0-9-_]+\\.[A-Za-z0-9-_]+$',
              }),
              expiresIn: like(3600),
              tokenType: 'Bearer',
            },
          },
        },
      }),
    );

    it('returns JWT tokens on login', async () => {
      const result = await login('http://localhost:4002', 'user@example.com', 'password123');
      expect(result.success).toBe(true);
      expect(result.data.tokenType).toBe('Bearer');
    });
  });
});
