import axios from 'axios';
import dotenv from 'dotenv';
dotenv.config({ path: __dirname + '/../.env.local' });

describe('Soroban Localnet', () => {
  it('should be running and respond with network info', async () => {
    const rpcUrl = process.env.RPC_URL?.replace(/\/rpc$/, '') || 'http://localhost:8000';
    const res = await axios.get(rpcUrl);
    expect(res.status).toBe(200);
    expect(res.data).toHaveProperty('network_passphrase');
    expect(res.data).toHaveProperty('core_latest_ledger');
    expect(res.data.network_passphrase).toBe(process.env.NETWORK_PASSPHRASE);
  });
});
