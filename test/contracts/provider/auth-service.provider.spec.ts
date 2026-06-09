import { Verifier, VerifierOptions } from '@pact-foundation/pact';
import * as path from 'path';
import { PACT_CONFIG, SERVICES, PROVIDER_URLS } from '../../../src/shared/pact.config';

const stateHandlers: Record<string, () => Promise<void>> = {
  'a valid JWT token exists for user user-abc-123': async () => {
    console.log('[State] valid token set up');
  },
  'an expired JWT token': async () => {
    console.log('[State] expired token scenario ready');
  },
  'a user with email user@example.com exists with correct password': async () => {
    console.log('[State] login user seeded');
  },
};

describe(`Provider Verification: ${SERVICES.AUTH_SERVICE}`, () => {
  it('satisfies all consumer contracts', async () => {
    const isCI = !!process.env.CI;

    const verifierOptions: VerifierOptions = {
      provider: SERVICES.AUTH_SERVICE,
      providerBaseUrl: PROVIDER_URLS[SERVICES.AUTH_SERVICE],
      providerStatesSetupUrl: `${PROVIDER_URLS[SERVICES.AUTH_SERVICE]}/_pact/provider-states`,
      stateHandlers,
      logLevel: 'warn',
      ...(isCI
        ? {
            pactBrokerUrl: PACT_CONFIG.BROKER_URL,
            pactBrokerToken: PACT_CONFIG.BROKER_TOKEN,
            consumerVersionSelectors: [
              { branch: 'main', latest: true },
              { branch: PACT_CONFIG.GIT_BRANCH, latest: true },
              { deployedOrReleased: true },
            ],
            publishVerificationResult: true,
            providerVersion: process.env.PROVIDER_VERSION || '1.0.0',
            providerVersionBranch: PACT_CONFIG.GIT_BRANCH,
          }
        : {
            pactUrls: [
              path.join(PACT_CONFIG.PACT_DIR, `${SERVICES.USER_SERVICE}-${SERVICES.AUTH_SERVICE}.json`),
            ],
          }),
    };

    const output = await new Verifier(verifierOptions).verifyProvider();
    console.log('Verification result:', output);
  }, 60_000);
});
