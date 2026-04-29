const fs = require('fs');
const path = require('path');

const SUG_FILE = path.join(process.cwd(), 'api', 'data', 'suggestions.json');

function main() {
  if (!fs.existsSync(SUG_FILE)) {
    console.error('suggestions.json not found:', SUG_FILE);
    process.exit(2);
  }

  let raw = fs.readFileSync(SUG_FILE, 'utf8');
  try {
    const obj = JSON.parse(raw);
    const keys = Object.keys(obj || {});
    if (keys.length === 0) {
      console.error('suggestions.json is empty');
      process.exit(3);
    }
    console.log('suggestions.json contains', keys.length, 'queries. Smoke test passed.');
    process.exit(0);
  } catch (err) {
    console.error('Failed to parse suggestions.json', err);
    process.exit(4);
  }
}

if (require.main === module) main();
