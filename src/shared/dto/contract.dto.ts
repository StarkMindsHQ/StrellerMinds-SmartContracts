// ─── User Service ─────────────────────────────────────────────────────────────

export interface UserDto {
  id: string;
  email: string;
  username: string;
  createdAt: string;
  roles: string[];
}

export interface CreateUserDto {
  email: string;
  username: string;
  password: string;
}

export interface UserResponseDto {
  success: boolean;
  data: UserDto;
}

export interface UsersListResponseDto {
  success: boolean;
  data: UserDto[];
  total: number;
  page: number;
  limit: number;
}

// ─── Auth Service ─────────────────────────────────────────────────────────────

export interface TokenDto {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
  tokenType: 'Bearer';
}

export interface LoginDto {
  email: string;
  password: string;
}

export interface ValidateTokenDto {
  token: string;
}

export interface ValidateTokenResponseDto {
  valid: boolean;
  userId?: string;
  roles?: string[];
}

export interface AuthResponseDto {
  success: boolean;
  data: TokenDto;
}

// ─── Notification Service ─────────────────────────────────────────────────────

export interface NotificationDto {
  id: string;
  userId: string;
  type: 'email' | 'sms' | 'push';
  message: string;
  status: 'pending' | 'sent' | 'failed';
  createdAt: string;
}

export interface SendNotificationDto {
  userId: string;
  type: 'email' | 'sms' | 'push';
  message: string;
  subject?: string;
}

export interface NotificationResponseDto {
  success: boolean;
  data: NotificationDto;
}

// ─── Error shapes ─────────────────────────────────────────────────────────────

export interface ErrorResponseDto {
  success: false;
  error: {
    code: string;
    message: string;
    statusCode: number;
  };
}

// ─── Version metadata ─────────────────────────────────────────────────────────

export interface ContractVersionDto {
  service: string;
  version: string;
  contractVersion: string;
  supportedVersions: string[];
  }
