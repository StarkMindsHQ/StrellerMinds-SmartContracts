import * as LocalAuthentication from 'expo-local-authentication';
import * as SecureStore from 'expo-secure-store';
import { BiometricAuthResult } from '../types';

const SECRET_KEY = 'streller_biometric_secret';

export class BiometricAuthService {
  static async isAvailable(): Promise<boolean> {
    const compatible = await LocalAuthentication.hasHardwareAsync();
    const enrolled = await LocalAuthentication.isEnrolledAsync();
    return compatible && enrolled;
  }

  static async authenticate(promptMessage = 'Verify your identity'): Promise<BiometricAuthResult> {
    try {
      const result = await LocalAuthentication.authenticateAsync({
        promptMessage,
        fallbackLabel: 'Use passcode',
        disableDeviceFallback: false,
      });

      if (result.success) {
        return { success: true };
      }

      return { success: false, error: result.error || 'Authentication failed' };
    } catch (err: any) {
      return { success: false, error: err.message || 'Unknown biometric error' };
    }
  }

  static async setupBiometricProtection(): Promise<boolean> {
    const available = await this.isAvailable();
    if (!available) return false;

    const result = await this.authenticate('Enable biometric login');
    if (!result.success) return false;

    // Store a flag indicating biometric is enabled
    await SecureStore.setItemAsync(SECRET_KEY, 'enabled', {
      requireAuthentication: true,
      keychainAccessible: SecureStore.WHEN_UNLOCKED_THIS_DEVICE_ONLY,
    });

    return true;
  }

  static async isBiometricEnabled(): Promise<boolean> {
    try {
      const value = await SecureStore.getItemAsync(SECRET_KEY, {
        requireAuthentication: false,
      });
      return value === 'enabled';
    } catch {
      return false;
    }
  }

  static async disableBiometric(): Promise<void> {
    await SecureStore.deleteItemAsync(SECRET_KEY);
  }

  static async promptAndVerify(): Promise<boolean> {
    const enabled = await this.isBiometricEnabled();
    if (!enabled) return true;

    const result = await this.authenticate('Access your credentials');
    return result.success;
  }
}
