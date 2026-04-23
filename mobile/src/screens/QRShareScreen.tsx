import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Share } from 'react-native';
import QRCode from 'react-native-qrcode-svg';
import { Certificate } from '../types';
import { CertificateService } from '../services/certificateService';

interface QRShareScreenProps {
  certificate: Certificate;
}

export default function QRShareScreen({ certificate }: QRShareScreenProps) {
  const service = new CertificateService();
  const payload = service.generateQRPayload(certificate);
  const qrData = JSON.stringify(payload);

  const handleShare = async () => {
    try {
      await Share.share({
        message: `Verify my credential: ${certificate.title}\n\nBlockchain Anchor: ${certificate.blockchainAnchor || 'N/A'}`,
        title: certificate.title,
      });
    } catch (err) {
      // User cancelled share
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Share Credential</Text>
      <Text style={styles.subtitle}>{certificate.title}</Text>

      <View style={styles.qrContainer}>
        <QRCode value={qrData} size={220} backgroundColor="#FFFFFF" color="#0F172A" />
      </View>

      <Text style={styles.hint}>
        Scan this QR code to verify the credential authenticity.
      </Text>

      <TouchableOpacity style={styles.button} onPress={handleShare}>
        <Text style={styles.buttonText}>Share Link</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8FAFC',
    alignItems: 'center',
    padding: 24,
  },
  title: {
    fontSize: 20,
    fontWeight: '700',
    color: '#0F172A',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#64748B',
    marginBottom: 32,
  },
  qrContainer: {
    backgroundColor: '#FFFFFF',
    padding: 24,
    borderRadius: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.08,
    shadowRadius: 8,
    elevation: 4,
    marginBottom: 24,
  },
  hint: {
    fontSize: 13,
    color: '#64748B',
    textAlign: 'center',
    marginBottom: 32,
    paddingHorizontal: 16,
  },
  button: {
    backgroundColor: '#2563EB',
    paddingVertical: 14,
    paddingHorizontal: 32,
    borderRadius: 12,
    width: '100%',
    alignItems: 'center',
  },
  buttonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: '600',
  },
});
