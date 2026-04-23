import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { Certificate } from '../types';
import { CertificateService } from '../services/certificateService';

interface CredentialDetailScreenProps {
  certificate: Certificate;
  sourceSecret: string;
  onSharePress: (cert: Certificate) => void;
}

export default function CredentialDetailScreen({
  certificate,
  sourceSecret,
  onSharePress,
}: CredentialDetailScreenProps) {
  const [verifying, setVerifying] = useState(false);
  const [verificationResult, setVerificationResult] = useState<boolean | null>(null);

  const service = new CertificateService();

  const handleVerify = async () => {
    setVerifying(true);
    try {
      const isValid = await service.verifyCertificate(certificate.certificateId, sourceSecret);
      setVerificationResult(isValid);
      Alert.alert(
        isValid ? 'Certificate Valid' : 'Certificate Invalid',
        isValid
          ? 'This credential is authentic and active.'
          : 'This credential could not be verified.'
      );
    } catch (err) {
      Alert.alert('Error', 'Unable to verify certificate at this time.');
    } finally {
      setVerifying(false);
    }
  };

  const isExpired = certificate.expiryDate > 0 && certificate.expiryDate < Date.now() / 1000;

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <Text style={styles.title}>{certificate.title}</Text>
        <View
          style={[
            styles.statusBadge,
            certificate.status === 'active' && !isExpired
              ? styles.statusActive
              : styles.statusInactive,
          ]}
        >
          <Text style={styles.statusText}>
            {isExpired ? 'Expired' : certificate.status}
          </Text>
        </View>
      </View>

      <View style={styles.section}>
        <Text style={styles.label}>Course</Text>
        <Text style={styles.value}>{certificate.courseId}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.label}>Description</Text>
        <Text style={styles.value}>{certificate.description}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.label}>Student Address</Text>
        <Text style={styles.valueMono}>{certificate.student}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.label}>Issuer</Text>
        <Text style={styles.valueMono}>{certificate.issuer}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.label}>Issued At</Text>
        <Text style={styles.value}>
          {new Date(certificate.issuedAt * 1000).toLocaleDateString()}
        </Text>
      </View>

      {certificate.expiryDate > 0 && (
        <View style={styles.section}>
          <Text style={styles.label}>Expires</Text>
          <Text style={[styles.value, isExpired && styles.expiredText]}>
            {new Date(certificate.expiryDate * 1000).toLocaleDateString()}
          </Text>
        </View>
      )}

      {certificate.blockchainAnchor && (
        <View style={styles.section}>
          <Text style={styles.label}>Blockchain Anchor</Text>
          <Text style={styles.valueMono}>{certificate.blockchainAnchor}</Text>
        </View>
      )}

      <View style={styles.section}>
        <Text style={styles.label}>Version</Text>
        <Text style={styles.value}>{certificate.version}</Text>
      </View>

      <View style={styles.actions}>
        <TouchableOpacity
          style={[styles.button, styles.verifyButton]}
          onPress={handleVerify}
          disabled={verifying}
        >
          {verifying ? (
            <ActivityIndicator color="#FFFFFF" />
          ) : (
            <Text style={styles.buttonText}>Verify on Blockchain</Text>
          )}
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.button, styles.shareButton]}
          onPress={() => onSharePress(certificate)}
        >
          <Text style={styles.buttonText}>Share via QR</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8FAFC',
  },
  content: {
    padding: 20,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 24,
  },
  title: {
    fontSize: 22,
    fontWeight: '700',
    color: '#0F172A',
    flex: 1,
    marginRight: 12,
  },
  statusBadge: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
  },
  statusActive: {
    backgroundColor: '#DCFCE7',
  },
  statusInactive: {
    backgroundColor: '#FEE2E2',
  },
  statusText: {
    fontSize: 12,
    fontWeight: '700',
    textTransform: 'uppercase',
  },
  section: {
    marginBottom: 20,
  },
  label: {
    fontSize: 12,
    fontWeight: '600',
    color: '#64748B',
    textTransform: 'uppercase',
    marginBottom: 4,
  },
  value: {
    fontSize: 15,
    color: '#0F172A',
    lineHeight: 22,
  },
  valueMono: {
    fontSize: 13,
    color: '#334155',
    fontFamily: 'monospace',
  },
  expiredText: {
    color: '#EF4444',
  },
  actions: {
    marginTop: 12,
    gap: 12,
  },
  button: {
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  verifyButton: {
    backgroundColor: '#2563EB',
  },
  shareButton: {
    backgroundColor: '#059669',
  },
  buttonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: '600',
  },
});
