# Mobile App Testing Guide

This guide defines the test strategy for the StrellerMinds iOS and Android apps. It covers unit, integration, UI, accessibility, performance, security-adjacent, and release validation checks for credential viewing, QR sharing, offline access, biometric login, push notifications, and blockchain verification.

## Goals

- Keep core credential workflows reliable across iOS and Android.
- Catch regressions before App Store, Play Store, TestFlight, or internal-track releases.
- Maintain fast feedback for developers and deeper validation in CI.
- Validate offline, low-connectivity, and device-security behavior that users depend on.

## Test Pyramid

| Layer | Purpose | Examples | Run cadence |
| --- | --- | --- | --- |
| Unit | Validate isolated business logic and view models | QR payload parsing, credential status mapping, cache expiry | Every commit |
| Integration | Validate app services working together | Credential sync, Stellar verification, encrypted storage reads | Pull requests and nightly |
| UI | Validate user journeys on real screens | Login, credential details, QR sharing, offline credential access | Pull requests for changed flows |
| Performance | Validate speed, stability, and resource use | Launch time, credential list load, QR generation, battery/network usage | Nightly and release candidates |
| Accessibility | Validate inclusive mobile UX | Screen reader labels, contrast, Dynamic Type/font scaling | Pull requests and release candidates |

## Unit Testing

### Scope

Unit tests should cover pure logic and small collaborators:

- Credential validation and verification-state mapping.
- QR code payload serialization and parsing.
- Offline cache expiration rules, including the 30-day access policy.
- Biometric availability decisions and fallback state transitions.
- View model state for loading, empty, success, and error screens.
- Date, locale, currency, and network error formatting.

### iOS Example: XCTest View Model Test

```swift
import XCTest
@testable import StrellerMinds

final class CredentialListViewModelTests: XCTestCase {
    func testLoadsCachedCredentialsWhenOffline() async throws {
        let cache = InMemoryCredentialCache(credentials: [
            Credential.fixture(id: "cert-1", title: "Blockchain Basics")
        ])
        let service = CredentialServiceMock(result: .failure(NetworkError.offline))
        let viewModel = CredentialListViewModel(service: service, cache: cache)

        await viewModel.loadCredentials()

        XCTAssertEqual(viewModel.credentials.count, 1)
        XCTAssertEqual(viewModel.state, .offline)
        XCTAssertEqual(viewModel.credentials.first?.id, "cert-1")
    }
}
```

### Android Example: JUnit ViewModel Test

```kotlin
@OptIn(ExperimentalCoroutinesApi::class)
class CredentialListViewModelTest {
    @get:Rule val dispatcherRule = MainDispatcherRule()

    @Test
    fun loadsCachedCredentialsWhenOffline() = runTest {
        val cache = FakeCredentialCache(
            listOf(CredentialFixture.create(id = "cert-1", title = "Blockchain Basics"))
        )
        val service = FakeCredentialService(error = NetworkError.Offline)
        val viewModel = CredentialListViewModel(service, cache)

        viewModel.loadCredentials()

        assertEquals(CredentialListState.Offline, viewModel.state.value)
        assertEquals("cert-1", viewModel.credentials.value.first().id)
    }
}
```

## Integration Testing

### Scope

Integration tests should validate service boundaries:

- API client request/response decoding for credentials, verification, and user profile endpoints.
- Stellar SDK verification using local fixtures or testnet-safe mocked clients.
- Secure storage read/write/delete behavior for tokens, keys, and cached credentials.
- Offline sync from API response to local database and back to UI state.
- Push notification registration and deep-link routing.

### Recommended Patterns

- Use mock web servers for deterministic API responses.
- Use local database instances that are reset per test.
- Keep blockchain network calls behind a test double unless a nightly test intentionally targets testnet.
- Test migration scripts with real previous-version database snapshots.
- Avoid tests that depend on wall-clock time; inject a clock.

### Android Example: MockWebServer

```kotlin
@Test
fun fetchCredentialDecodesVerificationStatus() = runTest {
    server.enqueue(MockResponse().setBody("""
        {"id":"cert-1","title":"Blockchain Basics","verified":true}
    """.trimIndent()))

    val api = CredentialApi(baseUrl = server.url("/"))
    val credential = api.fetchCredential("cert-1")

    assertTrue(credential.verified)
    assertEquals("Blockchain Basics", credential.title)
}
```

### iOS Example: URLProtocol Stub

```swift
func testFetchCredentialDecodesVerificationStatus() async throws {
    URLProtocolStub.stub(
        url: URL(string: "https://api.test/credentials/cert-1")!,
        data: #"{"id":"cert-1","title":"Blockchain Basics","verified":true}"#.data(using: .utf8)!
    )

    let client = CredentialAPIClient(session: .stubbed)
    let credential = try await client.fetchCredential(id: "cert-1")

    XCTAssertTrue(credential.verified)
    XCTAssertEqual(credential.title, "Blockchain Basics")
}
```

## UI Testing

### Critical Journeys

- First launch, sign in, biometric setup, and PIN fallback.
- Credential list loading, filtering, empty state, and error recovery.
- Credential details view with verified, expired, revoked, and pending states.
- QR code generation, QR scan, and share-sheet launch.
- Offline mode after a successful sync.
- Push notification tap opening the correct credential.
- Logout and secure local data cleanup.

### iOS XCTest UI Example

```swift
func testUserCanOpenCredentialDetails() {
    let app = XCUIApplication()
    app.launchArguments = ["--ui-testing", "--seed-credentials"]
    app.launch()

    app.buttons["Sign in"].tap()
    app.cells["Blockchain Basics"].tap()

    XCTAssertTrue(app.staticTexts["Verified"].exists)
    XCTAssertTrue(app.buttons["Share"].exists)
}
```

### Android Compose UI Example

```kotlin
@Test
fun userCanOpenCredentialDetails() {
    composeRule.setContent {
        StrellerMindsApp(fakeRepository = SeededCredentialRepository())
    }

    composeRule.onNodeWithText("Blockchain Basics").performClick()
    composeRule.onNodeWithText("Verified").assertIsDisplayed()
    composeRule.onNodeWithContentDescription("Share credential").assertIsDisplayed()
}
```

## Performance Testing

### Budgets

| Metric | Target |
| --- | --- |
| Cold app launch | Under 2 seconds |
| Credential list load from cache | Under 1 second |
| Credential details render | Under 700 ms |
| QR code generation | Under 500 ms |
| Offline sync for 100 credentials | Under 5 seconds on a mid-range device |
| Crash-free sessions | 99.5% or higher |

### Checks

- Measure cold and warm launch on representative low, mid, and high-end devices.
- Run credential list tests with 0, 1, 100, and 1,000 credentials.
- Profile memory while scrolling credential lists and opening QR screens.
- Track network payload size for sync endpoints.
- Test battery impact for background sync and push handling.

### Android Macrobenchmark Example

```kotlin
@Test
fun coldStartup() = benchmarkRule.measureRepeated(
    packageName = "io.strellerminds.credentials",
    metrics = listOf(StartupTimingMetric()),
    iterations = 5,
    startupMode = StartupMode.COLD
) {
    pressHome()
    startActivityAndWait()
}
```

### iOS XCTest Metric Example

```swift
func testCredentialListLaunchPerformance() {
    measure(metrics: [XCTApplicationLaunchMetric()]) {
        let app = XCUIApplication()
        app.launchArguments = ["--ui-testing", "--seed-credentials"]
        app.launch()
    }
}
```

## Accessibility Testing

- Every actionable control must have a meaningful accessibility label.
- Credential status must not rely on color alone.
- Text must support Dynamic Type/font scaling without truncating critical data.
- QR screens must provide non-visual alternatives such as copyable verification links.
- Minimum touch target size should be 44x44 pt on iOS and 48x48 dp on Android.
- Validate VoiceOver and TalkBack flows for login, credential details, and sharing.

## Test Data

Maintain deterministic fixtures for:

- Verified certificate.
- Expired certificate.
- Revoked certificate.
- Pending verification certificate.
- Credential with long title and long issuer name.
- Credential with missing optional metadata.
- Offline cache older than 30 days.
- User with no credentials.

Avoid production user data in test fixtures. Use generated public keys, mock certificate IDs, and fake issuer names.

## CI Checklist

Run on every pull request:

```bash
# iOS
xcodebuild test \
  -scheme StrellerMinds \
  -destination 'platform=iOS Simulator,name=iPhone 15'

# Android
./gradlew test
./gradlew connectedDebugAndroidTest
```

Run nightly or before release:

```bash
# Android performance
./gradlew :benchmark:connectedCheck

# Android release validation
./gradlew lint test assembleRelease

# iOS release validation
xcodebuild test \
  -scheme StrellerMinds \
  -configuration Release \
  -destination 'platform=iOS Simulator,name=iPhone 15'
```

## Release Candidate Checklist

- Unit, integration, UI, accessibility, and performance tests pass.
- TestFlight and Play internal-track smoke tests pass on real devices.
- Offline mode works after reinstall, token refresh, and network loss.
- Biometric fallback works after biometric enrollment changes.
- App handles revoked, expired, and malformed credentials gracefully.
- Crash reporting and performance monitoring are enabled for the release build.
- Store screenshots and metadata match the release candidate behavior.

## Best Practices

- Prefer small, deterministic tests over broad tests that are hard to debug.
- Put business rules in view models or services where they can be unit tested.
- Use stable accessibility identifiers for UI automation.
- Keep network, clock, storage, and biometric providers injectable.
- Test both success and failure paths for every critical workflow.
- Add regression tests for every production bug before fixing it.
- Keep flaky tests out of required checks until the flake is understood and fixed.
- Review performance budgets during feature planning, not only at release time.

