import { Publisher } from '@pact-foundation/pact-node';
import { PACT_CONFIG } from '../shared/pact.config';

async function publishPacts(): Promise<void> {
  const version = process.env.CONSUMER_VERSION || '1.0.0';
  const branch = process.env.GIT_BRANCH || 'main';

  console.log(`Publishing pacts v${version} (branch: ${branch})...`);

  await new Publisher({
    pactFilesOrDirs: [PACT_CONFIG.PACT_DIR],
    pactBroker: PACT_CONFIG.BROKER_URL,
    pactBrokerToken: PACT_CONFIG.BROKER_TOKEN,
    consumerVersion: version,
    branch,
    tags: [branch, 'latest'],
  }).publishPacts();

  console.log('✅ Pacts published');
}

publishPacts().catch((err) => {
  console.error('Failed to publish pacts:', err);
  process.exit(1);
});
