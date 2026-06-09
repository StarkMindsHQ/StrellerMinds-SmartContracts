import { Verifier, VerifierOptions } from '@pact-foundation/pact';
import * as path from 'path';
import { PACT_CONFIG, SERVICES, PROVIDER_URLS } from '../../../src/shared/pact.config';

const stateHandlers: Record<string, () => Promise<void>> = {
  'a user with id user-abc-123 exists': async () => {
    console.log('[State] user-abc-123 exists');
  },
  'a user with id nonexistent does not exist': async () => {
    console.log('[State] nonexistent user cleaned');
  },
  'users exist in the system': async () => {
    console.log('[State] users seeded');
  },
  'the user system is ready to accept new users': async () => {
    console.log('[State] ready for new user');
  },
};

describe(`Provider Verification: ${SERVICES.USER_SERVICE}`, () => {
  it('satisfies all consumer contracts', async () => {
    const isCI = !!process.env.CI;

    const verifierOptions: VerifierOptions = {
      provider: SERVICES.USER_SERVICE,
      providerBaseUrl: PROVIDER_URLS[SERVICES.USER_SERVICE],
      providerStatesSetupUrl: `${PROVIDER_URLS[SERVICES.USER_SERVICE]}/_pact/provider-states`,
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
              path.join(PACT_CONFIG.PACT_DIR, `${SERVICES.API_GATEWAY}-${SERVICES.USER_SERVICE}.json`),
            ],
          }),
    };

    const output = await new Verifier(verifierOptions).verifyProvider();
    console.log('Verification result:', output);
  }, 60_000);
});
