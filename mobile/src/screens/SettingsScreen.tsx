import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet, TouchableOpacity, Switch, Alert } from 'react-native';
import { BiometricAuthService } from '../services/biometricAuth';
import { PushNotificationService } from '../services/pushNotification';
import { OfflineStorage } from '../services/offlineStorage';

export default function SettingsScreen() {
  const [biometricEnabled, setBiometricEnabled] = useState(false);
  const [biometricAvailable, setBiometricAvailable] = useState(false);
  const [pushToken, setPushToken] = useState<string | null>(null);
  const [lastSync, setLastSync] = useState<number | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    const available = await BiometricAuthService.isAvailable();
    setBiometricAvailable(available);
    const enabled = await BiometricAuthService.isBiometricEnabled();
    setBiometricEnabled(enabled);
    const sync = await OfflineStorage.getLastSyncTime();
    setLastSync(sync);
  };

  const toggleBiometric = async (value: boolean) => {
    if (value) {
      const success = await BiometricAuthService.setupBiometricProtection();
      if (success) {
        setBiometricEnabled(true);
        Alert.alert('Success', 'Biometric login enabled.');
      } else {
        Alert.alert('Error', 'Failed to enable biometric login.');
      }
    } else {
      await BiometricAuthService.disableBiometric();
      setBiometricEnabled(false);
    }
  };

  const registerPush = async () => {
    const token = await PushNotificationService.registerForPushNotifications();
    if (token) {
      setPushToken(token);
      Alert.alert('Push Notifications', 'Registered successfully.');
    } else {
      Alert.alert('Push Notifications', 'Unable to register. Use a physical device.');
    }
  };

  const clearCache = async () => {
    await OfflineStorage.clearAll();
    setLastSync(null);
    Alert.alert('Cache Cleared', 'All offline data has been removed.');
  };

  return (
    <View style={styles.container}>
      <Text style={styles.header}>Settings</Text>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Security</Text>
        <View style={styles.row}>
          <Text style={styles.rowLabel}>Biometric Login</Text>
          {biometricAvailable ? (
            <Switch value={biometricEnabled} onValueChange={toggleBiometric} />
          ) : (
            <Text style={styles.rowHint}>Not available</Text>
          )}
        </View>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Notifications</Text>
        <TouchableOpacity style={styles.rowButton} onPress={registerPush}>
          <Text style={styles.rowLabel}>
            {pushToken ? 'Push Notifications Active' : 'Enable Push Notifications'}
          </Text>
          <Text style={styles.rowHint}>{pushToken ? 'Registered' : 'Tap to register'}</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Offline Data</Text>
        <View style={styles.row}>
          <Text style={styles.rowLabel}>Last Sync</Text>
          <Text style={styles.rowHint}>
            {lastSync ? new Date(lastSync).toLocaleString() : 'Never'}
          </Text>
        </View>
        <TouchableOpacity style={styles.dangerButton} onPress={clearCache}>
          <Text style={styles.dangerText}>Clear Offline Cache</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8FAFC',
    padding: 20,
  },
  header: {
    fontSize: 24,
    fontWeight: '700',
    color: '#0F172A',
    marginBottom: 24,
  },
  section: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 4,
    elevation: 1,
  },
  sectionTitle: {
    fontSize: 12,
    fontWeight: '700',
    color: '#64748B',
    textTransform: 'uppercase',
    marginBottom: 12,
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
  },
  rowButton: {
    paddingVertical: 8,
  },
  rowLabel: {
    fontSize: 15,
    color: '#0F172A',
    fontWeight: '500',
  },
  rowHint: {
    fontSize: 13,
    color: '#94A3B8',
  },
  dangerButton: {
    marginTop: 12,
    backgroundColor: '#FEE2E2',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  dangerText: {
    color: '#B91C1C',
    fontWeight: '600',
    fontSize: 14,
  },
});
