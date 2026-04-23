import { useState, useEffect, useCallback } from 'react';
import { Certificate } from '../types';
import { CertificateService } from '../services/certificateService';
import { OfflineStorage } from '../services/offlineStorage';

export function useCertificates(studentAddress: string, sourceSecret: string) {
  const [certificates, setCertificates] = useState<Certificate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isOffline, setIsOffline] = useState(false);

  const service = new CertificateService();

  const fetchCertificates = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const certs = await service.getStudentCertificates(studentAddress, sourceSecret);
      setCertificates(certs);
      setIsOffline(false);
    } catch (err: any) {
      console.error('Fetch error:', err);
      // Fallback to offline cache
      const cached = await OfflineStorage.getCachedCertificates();
      setCertificates(cached);
      setIsOffline(true);
      if (cached.length === 0) {
        setError('Failed to load certificates. Please check your connection.');
      }
    } finally {
      setLoading(false);
    }
  }, [studentAddress, sourceSecret]);

  const refresh = useCallback(async () => {
    await fetchCertificates();
  }, [fetchCertificates]);

  useEffect(() => {
    fetchCertificates();
  }, [fetchCertificates]);

  return {
    certificates,
    loading,
    error,
    isOffline,
    refresh,
  };
}
