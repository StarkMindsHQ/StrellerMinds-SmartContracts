import React, { useState, useEffect } from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { StatusBar } from 'expo-status-bar';
import { Text, View } from 'react-native';

import LoginScreen from './src/screens/LoginScreen';
import CredentialsScreen from './src/screens/CredentialsScreen';
import CredentialDetailScreen from './src/screens/CredentialDetailScreen';
import QRShareScreen from './src/screens/QRShareScreen';
import QRScanScreen from './src/screens/QRScanScreen';
import SettingsScreen from './src/screens/SettingsScreen';

import { Certificate, SharedCredentialPayload } from './src/types';
import { PushNotificationService } from './src/services/pushNotification';
import { OfflineStorage } from './src/services/offlineStorage';

const Stack = createStackNavigator();
const Tab = createBottomTabNavigator();

// Demo configuration - replace with real wallet secret in production
const DEMO_STUDENT_ADDRESS = 'GBUV3...DEMO';
const DEMO_SOURCE_SECRET = 'S...DEMO_SECRET';

function CredentialsStack() {
  return (
    <Stack.Navigator screenOptions={{ headerShown: false }}>
      <Stack.Screen name="CredentialsList">
        {({ navigation }: any) => (
          <CredentialsScreen
            studentAddress={DEMO_STUDENT_ADDRESS}
            sourceSecret={DEMO_SOURCE_SECRET}
            onSelectCertificate={(cert: Certificate) =>
              navigation.navigate('CredentialDetail', { certificate: cert })
            }
          />
        )}
      </Stack.Screen>
      <Stack.Screen name="CredentialDetail" options={{ headerShown: true, title: 'Credential' }}>
        {({ route, navigation }: any) => (
          <CredentialDetailScreen
            certificate={route.params.certificate}
            sourceSecret={DEMO_SOURCE_SECRET}
            onSharePress={(cert: Certificate) =>
              navigation.navigate('QRShare', { certificate: cert })
            }
          />
        )}
      </Stack.Screen>
      <Stack.Screen name="QRShare" options={{ headerShown: true, title: 'Share' }}>
        {({ route }: any) => <QRShareScreen certificate={route.params.certificate} />}
      </Stack.Screen>
    </Stack.Navigator>
  );
}

function ScanStack() {
  return (
    <Stack.Navigator screenOptions={{ headerShown: false }}>
      <Stack.Screen name="QRScan">
        {({ navigation }: any) => (
          <QRScanScreen
            onCredentialVerified={(payload: SharedCredentialPayload) => {
              // Could navigate to a verification result screen
              console.log('Verified payload:', payload);
            }}
          />
        )}
      </Stack.Screen>
    </Stack.Navigator>
  );
}

function MainTabs() {
  return (
    <Tab.Navigator
      screenOptions={({ route }) => ({
        tabBarIcon: ({ focused, color, size }: any) => {
          let label = '';
          if (route.name === 'Credentials') label = '📜';
          else if (route.name === 'Scan') label = '📷';
          else if (route.name === 'Settings') label = '⚙️';
          return (
            <View style={{ alignItems: 'center' }}>
              <Text style={{ fontSize: size }}>{label}</Text>
              <Text style={{ fontSize: 10, color }}>{route.name}</Text>
            </View>
          );
        },
        tabBarShowLabel: false,
        headerShown: false,
      })}
    >
      <Tab.Screen name="Credentials" component={CredentialsStack} />
      <Tab.Screen name="Scan" component={ScanStack} />
      <Tab.Screen name="Settings" component={SettingsScreen} />
    </Tab.Navigator>
  );
}

export default function App() {
  const [authenticated, setAuthenticated] = useState(false);

  useEffect(() => {
    // Clean expired cache on startup
    OfflineStorage.clearExpiredCache();

    // Register for push notifications
    PushNotificationService.registerForPushNotifications().then((token) => {
      if (token) {
        console.log('Push token:', token);
      }
    });

    // Listen for incoming notifications
    const subscription = PushNotificationService.addNotificationReceivedListener((notification) => {
      console.log('Notification received:', notification);
    });

    return () => {
      PushNotificationService.removeSubscription(subscription);
    };
  }, []);

  if (!authenticated) {
    return (
      <>
        <LoginScreen onAuthenticated={() => setAuthenticated(true)} />
        <StatusBar style="auto" />
      </>
    );
  }

  return (
    <NavigationContainer>
      <MainTabs />
      <StatusBar style="auto" />
    </NavigationContainer>
  );
}
