import * as fs from 'fs';
import * as path from 'path';

const PACT_DIR = path.resolve(process.cwd(), 'pact/contracts');
const DOCS_DIR = path.resolve(process.cwd(), 'docs/contracts');

interface PactFile {
  consumer: { name: string };
  provider: { name: string };
  interactions: Array<{
    description: string;
    providerState?: string;
    request: { method: string; path: string; headers?: Record<string,string>; body?: unknown };
    response: { status: number; headers?: Record<string,string>; body?: unknown };
  }>;
  metadata: { pactSpecification: { version: string } };
}

function generateDoc(pact: PactFile, filename: string): string {
  let doc = `# Contract: ${pact.consumer.name} → ${pact.provider.name}\n\n`;
  doc += `> Auto-generated from \`${filename}\`\n\n`;
  doc += `| Property | Value |\n|---|---|\n`;
  doc += `| Consumer | ${pact.consumer.name} |\n`;
  doc += `| Provider | ${pact.provider.name} |\n`;
  doc += `| Interactions | ${pact.interactions.length} |\n\n`;
  doc += `## Interactions\n\n`;

  pact.interactions.forEach((i) => {
    doc += `### ${i.description}\n\n`;
    if (i.providerState) doc += `> **State:** ${i.providerState}\n\n`;
    doc += `**Request:** \`${i.request.method} ${i.request.path}\`\n\n`;
    doc += `**Response:** \`HTTP ${i.response.status}\`\n\n`;
    if (i.response.body) doc += `\`\`\`json\n${JSON.stringify(i.response.body, null, 2)}\n\`\`\`\n\n`;
    doc += `---\n\n`;
  });

  return doc;
}

async function generateDocs(): Promise<void> {
  fs.mkdirSync(DOCS_DIR, { recursive: true });

  if (!fs.existsSync(PACT_DIR)) {
    console.warn('No pact directory found. Run consumer tests first.');
    return;
  }

  const files = fs.readdirSync(PACT_DIR).filter((f) => f.endsWith('.json'));

  files.forEach((file) => {
    const pact: PactFile = JSON.parse(fs.readFileSync(path.join(PACT_DIR, file), 'utf-8'));
    const doc = generateDoc(pact, file);
    fs.writeFileSync(path.join(DOCS_DIR, file.replace('.json', '.md')), doc);
    console.log(`✅ ${file.replace('.json', '.md')}`);
  });

  console.log(`\nDocs generated in ${DOCS_DIR}`);
}

generateDocs().catch(console.error);
