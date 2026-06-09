# StrellerMinds Mobile Apps

Native iOS and Android applications for secure credential verification, sharing, and management.

## 🚀 Features

### Core Functionality
- **📱 Credential Viewing**: Display certificate details with verification status
- **🔗 QR Code Sharing**: Generate and share credentials via QR codes
- **📴 Offline Access**: 30-day offline credential viewing
- **🔐 Biometric Login**: Face ID/fingerprint authentication
- **🔔 Push Notifications**: Alerts for new certificates and expirations

### Security Features
- **🔒 End-to-End Encryption**: All stored credentials encrypted
- **🛡️ Secure Storage**: Private keys in device secure storage
- **✅ Verification**: Blockchain-based credential verification
- **🔍 Anti-Spoofing**: Protection against credential tampering

### User Experience
- **⚡ Fast Performance**: < 2 second app launch, < 1 second credential loading
- **🎨 Modern UI**: Clean, professional interface
- **♿ Accessibility**: Full accessibility support
- **🌍 Multi-Language**: Support for multiple languages

## 📱 App Store Information

### iOS App
- **Platform**: iOS 15.0+
- **Size**: ~25MB
- **Category**: Education
- **Rating**: ⭐⭐⭐⭐⭐ (Target: 4.5+)
- **Downloads**: 10,000+ (Target first month)

### Android App
- **Platform**: Android 7.0+ (API 24+)
- **Size**: ~20MB
- **Category**: Education
- **Rating**: ⭐⭐⭐⭐⭐ (Target: 4.5+)
- **Downloads**: 10,000+ (Target first month)

## 🏗️ Architecture

### Technology Stack

#### iOS
- **Language**: Swift 5.9+
- **UI**: SwiftUI
- **Architecture**: MVVM with Combine
- **Database**: Core Data
- **Networking**: URLSession
- **Stellar**: stellar-ios-sdk

#### Android
- **Language**: Kotlin 1.9+
- **UI**: Jetpack Compose
- **Architecture**: MVVM with Coroutines
- **Database**: Room
- **Networking**: OkHttp + Retrofit
- **Stellar**: stellar-android-sdk

### Key Components

#### 1. Credential Service
```swift
class CredentialService {
    func fetchCredential(id: String) async throws -> Credential
    func getUserCredentials() async throws -> [Credential]
    func verifyCredential(id: String) async throws -> VerificationResult
}
```

#### 2. QR Code Service
```swift
class QRCodeService {
    func generateQRCode(for credential: Credential) -> UIImage
    func scanQRCode(from image: UIImage) -> Credential?
}
```

#### 3. Offline Manager
```swift
class OfflineManager {
    func syncCredentials() async
    func isCredentialAvailableOffline(id: String) -> Bool
    func getCachedCredential(id: String) -> Credential?
}
```

#### 4. Biometric Service
```swift
class BiometricAuthService {
    func authenticate() async throws -> Bool
    func isAvailable() -> Bool
    func setupBiometric() async throws
}
```

## 🔧 Installation

### Development Setup

#### Prerequisites
- Xcode 15.0+ (for iOS)
- Android Studio Hedgehog+ (for Android)
- iOS 15.0+ device/simulator
- Android 7.0+ device/emulator

#### iOS Setup
```bash
cd mobile-app/ios
pod install
open StrellerMinds.xcworkspace
```

#### Android Setup
```bash
cd mobile-app/android
./gradlew build
```

### Production Builds

#### iOS App Store
```bash
# Archive and upload to App Store Connect
xcodebuild -workspace StrellerMinds.xcworkspace \
          -scheme StrellerMinds \
          -configuration Release \
          -destination generic/platform=iOS \
          archive
```

#### Android Play Store
```bash
# Generate signed APK/AAB
./gradlew assembleRelease
./gradlew bundleRelease
```

## 📊 Usage Guide

### Getting Started

1. **Download the App**
   - iOS: App Store - "StrellerMinds Credentials"
   - Android: Google Play - "StrellerMinds Credentials"

2. **Create Account**
   - Sign up with email or social login
   - Set up biometric authentication
   - Secure your account with 2FA

3. **Add Credentials**
   - Scan QR code from certificate issuer
   - Import from Stellar blockchain
   - Manual entry for legacy credentials

### Viewing Credentials

1. **Open App** → Authenticate with biometrics
2. **Browse** your credential collection
3. **Tap** any credential to view details
4. **Verify** authenticity on blockchain

### Sharing Credentials

1. **Select** credential to share
2. **Tap** Share button
3. **Choose** sharing method:
   - QR Code (in-person)
   - Direct link (email/messaging)
   - Social media (LinkedIn)

### Offline Access

1. **Sync** credentials while online
2. **Access** credentials offline for 30 days
3. **Automatic** sync when connection restored

## 🔒 Security

### Data Protection
- **Encryption**: AES-256 for all local data
- **Secure Storage**: Keychain/Keystore for sensitive data
- **Authentication**: Biometric + PIN backup
- **Verification**: Blockchain-based verification

### Privacy Features
- **Local Storage**: Credentials stored locally, not in cloud
- **No Tracking**: No analytics or tracking without consent
- **Data Minimization**: Only necessary data collected
- **User Control**: Full control over credential sharing

### Security Best Practices
- Regular security audits
- Penetration testing
- Dependency updates
- Vulnerability scanning

## 🧪 Testing

### Test Coverage
- **Unit Tests**: 90%+ coverage
- **Integration Tests**: All major flows
- **UI Tests**: Critical user journeys
- **Performance Tests**: Load and stress testing

See [Mobile App Testing Guide](TESTING_GUIDE.md) for the full testing strategy, platform examples, CI commands, accessibility checks, and release validation checklist.

### Running Tests

#### iOS
```bash
cd mobile-app/ios
xcodebuild test -scheme StrellerMinds -destination 'platform=iOS Simulator,name=iPhone 15'
```

#### Android
```bash
cd mobile-app/android
./gradlew test
./gradlew connectedAndroidTest
```

### Performance Benchmarks
- App Launch: < 2 seconds
- Credential Load: < 1 second
- QR Generation: < 500ms
- Battery Usage: < 5% daily

## 📈 Analytics & Monitoring

### Key Metrics
- **User Engagement**: Daily/Monthly active users
- **Credential Usage**: Views, shares, verifications
- **Performance**: App crashes, load times
- **User Satisfaction**: App store ratings, feedback

### Monitoring Tools
- **Firebase Analytics**: User behavior tracking
- **Crashlytics**: Crash reporting
- **Performance Monitoring**: App performance
- **Custom Analytics**: Credential-specific metrics

## 🚀 Deployment

### Release Process

1. **Development**
   - Feature development on feature branches
   - Code reviews and testing
   - Integration testing

2. **Staging**
   - Beta testing with TestFlight/Play Console
   - Performance testing
   - Security validation

3. **Production**
   - App Store/Play Store submission
   - Phased rollout
   - Monitoring and support

### Release Checklist
- [ ] All tests passing
- [ ] Performance benchmarks met
- [ ] Security audit completed
- [ ] App store metadata ready
- [ ] User documentation updated
- [ ] Support team trained

## 🤝 Contributing

### Development Workflow
1. Fork the repository
2. Create feature branch
3. Implement changes with tests
4. Submit pull request
5. Code review and merge

### Code Standards
- Swift: SwiftLint configuration
- Kotlin: Detekt configuration
- Documentation: Comprehensive code comments
- Testing: TDD approach encouraged

## 📞 Support

### User Support
- **Email**: support@strellerminds.io
- **Help Center**: https://help.strellerminds.io
- **FAQ**: In-app help section
- **Community**: Discord/Telegram channels

### Developer Support
- **Documentation**: Comprehensive API docs
- **SDK Integration**: Stellar SDK guides
- **Sample Code**: GitHub repository
- **Technical Support**: dev-support@strellerminds.io

## 🗺️ Roadmap

### Version 1.0 (Current)
- ✅ Basic credential viewing
- ✅ QR code sharing
- ✅ Offline access
- ✅ Biometric authentication
- ✅ Push notifications

### Version 1.1 (Q2 2024)
- 🔄 Wallet integration
- 🔄 Social media sharing
- 🔄 Advanced search
- 🔄 Credential templates

### Version 2.0 (Q3 2024)
- 📋 Multi-blockchain support
- 📋 AI-powered recommendations
- 📋 AR credential display
- 📋 Voice authentication

### Version 2.1 (Q4 2024)
- 📋 Web application (PWA)
- 📋 Desktop apps
- 📋 Wearable integration
- 📋 Enterprise features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Stellar Development Foundation for blockchain infrastructure
- Open source community for libraries and tools
- Beta testers for valuable feedback
- Educational partners for credential integration

---

**Download Today!**

🍎 [iOS App Store](https://apps.apple.com/app/strellerminds-credentials) | 🤖 [Google Play Store](https://play.google.com/store/apps/details?id=io.strellerminds.credentials)

**Questions?** Contact us at [mobile@strellerminds.io](mailto:mobile@strellerminds.io)
