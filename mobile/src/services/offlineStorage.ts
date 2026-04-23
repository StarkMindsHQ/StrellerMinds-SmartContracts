import AsyncStorage from '@react-native-async-storage/async-storage';
import { Certificate, OfflineCacheEntry } from '../types';

const CERTIFICATES_KEY = '@streller/credentials';
const OFFLINE_CACHE_KEY = '@streller/offline_cache';
const LAST_SYNC_KEY = '@streller/last_sync';

// 30 days in milliseconds
const OFFLINE_TTL_MS = 30 * 24 * 60 * 60 * 1000;

export class OfflineStorage {
  static async saveCertificates(certs: Certificate[]): Promise<void> {
    const entries: Record<string, OfflineCacheEntry> = {};
    const now = Date.now();
    for (const cert of certs) {
      entries[cert.certificateId] = {
        certificate: cert,
        cachedAt: now,
        expiresAt: now + OFFLINE_TTL_MS,
      };
    }
    await AsyncStorage.setItem(OFFLINE_CACHE_KEY, JSON.stringify(entries));
    await AsyncStorage.setItem(LAST_SYNC_KEY, now.toString());
  }

  static async getCachedCertificates(): Promise<Certificate[]> {
    const raw = await AsyncStorage.getItem(OFFLINE_CACHE_KEY);
    if (!raw) return [];

    const entries: Record<string, OfflineCacheEntry> = JSON.parse(raw);
    const now = Date.now();
    const valid: Certificate[] = [];

    for (const entry of Object.values(entries)) {
      if (entry.expiresAt > now) {
        valid.push(entry.certificate);
      }
    }

    return valid;
  }

  static async getCertificateById(id: string): Promise<Certificate | null> {
    const raw = await AsyncStorage.getItem(OFFLINE_CACHE_KEY);
    if (!raw) return null;

    const entries: Record<string, OfflineCacheEntry> = JSON.parse(raw);
    const entry = entries[id];
    if (!entry || entry.expiresAt <= Date.now()) return null;
    return entry.certificate;
  }

  static async addOrUpdateCertificate(cert: Certificate): Promise<void> {
    const raw = await AsyncStorage.getItem(OFFLINE_CACHE_KEY);
    const entries: Record<string, OfflineCacheEntry> = raw ? JSON.parse(raw) : {};
    const now = Date.now();
    entries[cert.certificateId] = {
      certificate: cert,
      cachedAt: now,
      expiresAt: now + OFFLINE_TTL_MS,
    };
    await AsyncStorage.setItem(OFFLINE_CACHE_KEY, JSON.stringify(entries));
  }

  static async clearExpiredCache(): Promise<void> {
    const raw = await AsyncStorage.getItem(OFFLINE_CACHE_KEY);
    if (!raw) return;

    const entries: Record<string, OfflineCacheEntry> = JSON.parse(raw);
    const now = Date.now();
    const cleaned: Record<string, OfflineCacheEntry> = {};

    for (const [key, entry] of Object.entries(entries)) {
      if (entry.expiresAt > now) {
        cleaned[key] = entry;
      }
    }

    await AsyncStorage.setItem(OFFLINE_CACHE_KEY, JSON.stringify(cleaned));
  }

  static async isDataFresh(): Promise<boolean> {
    const lastSync = await AsyncStorage.getItem(LAST_SYNC_KEY);
    if (!lastSync) return false;
    const lastSyncTime = parseInt(lastSync, 10);
    return Date.now() - lastSyncTime < OFFLINE_TTL_MS;
  }

  static async getLastSyncTime(): Promise<number | null> {
    const lastSync = await AsyncStorage.getItem(LAST_SYNC_KEY);
    return lastSync ? parseInt(lastSync, 10) : null;
  }

  static async clearAll(): Promise<void> {
    await AsyncStorage.removeItem(OFFLINE_CACHE_KEY);
    await AsyncStorage.removeItem(LAST_SYNC_KEY);
    await AsyncStorage.removeItem(CERTIFICATES_KEY);
  }
}
