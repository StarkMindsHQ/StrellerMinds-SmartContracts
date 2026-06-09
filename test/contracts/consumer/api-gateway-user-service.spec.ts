import { Pact, Matchers } from '@pact-foundation/pact';
import * as path from 'path';
import { PACT_CONFIG, SERVICES } from '../../../src/shared/pact.config';
import { UserDto, UserResponseDto, UsersListResponseDto } from '../../../src/shared/dto/contract.dto';

const { like, eachLike, term } = Matchers;

const provider = new Pact({
  consumer: SERVICES.API_GATEWAY,
  provider: SERVICES.USER_SERVICE,
  port: 4001,
  log: path.join(PACT_CONFIG.LOG_DIR, 'api-gateway-user-service.log'),
  dir: PACT_CONFIG.PACT_DIR,
  logLevel: 'warn',
});

async function getUserById(baseUrl: string, userId: string): Promise<UserResponseDto> {
  const res = await fetch(`${baseUrl}/users/${userId}`, {
    headers: { Accept: 'application/json' },
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<UserResponseDto>;
}

async function getUsers(baseUrl: string, page = 1, limit = 10): Promise<UsersListResponseDto> {
  const res = await fetch(`${baseUrl}/users?page=${page}&limit=${limit}`, {
    headers: { Accept: 'application/json' },
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<UsersListResponseDto>;
}

async function createUser(baseUrl: string, payload: { email: string; username: string; password: string }): Promise<UserResponseDto> {
  const res = await fetch(`${baseUrl}/users`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<UserResponseDto>;
}

describe('ApiGateway → UserService (Consumer Contract)', () => {
  beforeAll(() => provider.setup());
  afterEach(() => provider.verify());
  afterAll(() => provider.finalize());

  describe('GET /users/:id', () => {
    const userId = 'user-abc-123';

    beforeEach(() =>
      provider.addInteraction({
        state: 'a user with id user-abc-123 exists',
        uponReceiving: 'a request to get user by id',
        withRequest: {
          method: 'GET',
          path: `/users/${userId}`,
          headers: { Accept: 'application/json' },
        },
        willRespondWith: {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
          body: {
            success: true,
            data: {
              id: like(userId),
              email: like('user@example.com'),
              username: like('johndoe'),
              createdAt: term({
                generate: '2024-01-15T10:30:00.000Z',
                matcher: '^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}',
              }),
              roles: eachLike('user'),
            },
          },
        },
      }),
    );

    it('returns a user object', async () => {
      const result = await getUserById('http://localhost:4001', userId);
      expect(result.success).toBe(true);
      expect(result.data).toMatchObject<Partial<UserDto>>({
        id: expect.any(String),
        email: expect.any(String),
        username: expect.any(String),
        roles: expect.arrayContaining([expect.any(String)]),
      });
    });
  });

  describe('GET /users/:id (not found)', () => {
    beforeEach(() =>
      provider.addInteraction({
        state: 'a user with id nonexistent does not exist',
        uponReceiving: 'a request for a non-existent user',
        withRequest: {
          method: 'GET',
          path: '/users/nonexistent',
          headers: { Accept: 'application/json' },
        },
        willRespondWith: {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
          body: {
            success: false,
            error: {
              code: like('USER_NOT_FOUND'),
              message: like('User not found'),
              statusCode: like(404),
            },
          },
        },
      }),
    );

    it('throws on 404', async () => {
      await expect(getUserById('http://localhost:4001', 'nonexistent')).rejects.toThrow('HTTP 404');
    });
  });

  describe('GET /users', () => {
    beforeEach(() =>
      provider.addInteraction({
        state: 'users exist in the system',
        uponReceiving: 'a request to list users',
        withRequest: {
          method: 'GET',
          path: '/users',
          query: { page: '1', limit: '10' },
          headers: { Accept: 'application/json' },
        },
        willRespondWith: {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
          body: {
            success: true,
            data: eachLike({
              id: like('user-abc-123'),
              email: like('user@example.com'),
              username: like('johndoe'),
              createdAt: like('2024-01-15T10:30:00.000Z'),
              roles: eachLike('user'),
            }),
            total: like(1),
            page: like(1),
            limit: like(10),
          },
        },
      }),
    );

    it('returns paginated users', async () => {
      const result = await getUsers('http://localhost:4001', 1, 10);
      expect(result.success).toBe(true);
      expect(Array.isArray(result.data)).toBe(true);
    });
  });

  describe('POST /users', () => {
    const newUser = { email: 'newuser@example.com', username: 'newuser', password: 'SecurePass123!' };

    beforeEach(() =>
      provider.addInteraction({
        state: 'the user system is ready to accept new users',
        uponReceiving: 'a request to create a new user',
        withRequest: {
          method: 'POST',
          path: '/users',
          headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
          body: {
            email: like(newUser.email),
            username: like(newUser.username),
            password: like(newUser.password),
          },
        },
        willRespondWith: {
          status: 201,
          headers: { 'Content-Type': 'application/json' },
          body: {
            success: true,
            data: {
              id: like('user-new-456'),
              email: like(newUser.email),
              username: like(newUser.username),
              createdAt: like('2024-01-15T10:30:00.000Z'),
              roles: eachLike('user'),
            },
          },
        },
      }),
    );

    it('creates and returns a new user', async () => {
      const result = await createUser('http://localhost:4001', newUser);
      expect(result.success).toBe(true);
      expect(result.data.email).toBe(newUser.email);
    });
  });
});
