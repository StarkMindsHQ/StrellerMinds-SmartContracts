import {
  Keypair,
  Contract,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
  Operation,
  Asset,
  xdr
} from '@stellar/stellar-sdk';
import * as fs from 'fs';
import * as path from 'path';
import { config } from './config.js';

export class SorobanClient {
  private server: SorobanRpc.Server;
  private networkPassphrase: string;

  constructor() {
    this.server = new SorobanRpc.Server(config.networkUrl);
    this.networkPassphrase = config.networkPassphrase;
  }

  /**
   * Get or create a funded account
   */
  async getFundedAccount(secret?: string): Promise<Keypair> {
    const keypair = secret ? Keypair.fromSecret(secret) : Keypair.random();

    try {
      await this.server.getAccount(keypair.publicKey());
      console.log(`Account ${keypair.publicKey()} already exists and is funded`);
    } catch (error) {
      console.log(`Funding account ${keypair.publicKey()}...`);
      await this.fundAccount(keypair.publicKey());
    }

    return keypair;
  }

  /**
   * Fund an account using the localnet friendbot
   */
  private async fundAccount(publicKey: string): Promise<void> {
    const friendbotUrl = `${config.networkUrl.replace('/soroban/rpc', '')}/friendbot?addr=${publicKey}`;

    try {
      const response = await fetch(friendbotUrl);
      if (!response.ok) {
        throw new Error(`Friendbot request failed: ${response.statusText}`);
      }
      console.log(`Account ${publicKey} funded successfully`);
    } catch (error) {
      console.error(`Failed to fund account: ${error}`);
      throw error;
    }
  }

  /**
   * Deploy a contract from WASM file
   */
  async deployContract(
    wasmPath: string,
    deployer: Keypair
  ): Promise<string> {
    const wasmBuffer = fs.readFileSync(wasmPath);

    // Get account details
    const account = await this.server.getAccount(deployer.publicKey());

    // Upload WASM
    console.log(`Uploading WASM: ${path.basename(wasmPath)}`);
    const uploadOperation = Operation.uploadContractWasm({
      wasm: wasmBuffer,
    });

    const uploadTx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(uploadOperation)
      .setTimeout(30)
      .build();

    const uploadResult = await this.submitTransaction(uploadTx, deployer);
    const wasmHash = uploadResult.returnValue;

    if (!wasmHash) {
      throw new Error('Failed to get WASM hash from upload');
    }

    console.log(`WASM uploaded successfully, hash: ${wasmHash.toString('hex')}`);

    // Create contract instance
    const accountRefreshed = await this.server.getAccount(deployer.publicKey());
    const createOperation = Operation.createCustomContract({
      wasmHash: wasmHash as Buffer,
      address: (deployer as unknown) as any,
    });

    const createTx = new TransactionBuilder(accountRefreshed, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(createOperation)
      .setTimeout(30)
      .build();

    const createResult = await this.submitTransaction(createTx, deployer);
    const contractId = createResult.returnValue?.toString('hex');

    if (!contractId) {
      throw new Error('Failed to get contract ID from creation');
    }

    console.log(`Contract deployed successfully: ${contractId}`);
    return contractId;
  }

  /**
   * Submit and process a transaction
   */
  private async submitTransaction(
    transaction: any,
    signer: Keypair
  ): Promise<any> {
    // Sign transaction
    transaction.sign(signer);

    // Submit to network
    const response = await this.server.sendTransaction(transaction);

    if (response.status !== 'PENDING') {
      throw new Error(`Transaction failed: ${response.status}`);
    }

    // Poll for result
    let getResponse = await this.server.getTransaction(response.hash);
    let attempts = 0;
    const maxAttempts = 10;

    while (getResponse.status === 'NOT_FOUND' && attempts < maxAttempts) {
      await new Promise(resolve => setTimeout(resolve, 1000));
      getResponse = await this.server.getTransaction(response.hash);
      attempts++;
    }

    if (getResponse.status !== 'SUCCESS') {
      throw new Error(`Transaction not successful: ${getResponse.status}`);
    }

    return getResponse;
  }

  /**
   * Invoke a contract method
   */
  async invokeContract(
    contractId: string,
    method: string,
    args: xdr.ScVal[],
    caller: Keypair
  ): Promise<any> {
    const contract = new Contract(contractId);
    const account = await this.server.getAccount(caller.publicKey());

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(30)
      .build();

    return await this.submitTransaction(tx, caller);
  }

  /**
   * Get contract data
   */
  async getContractData(contractId: string): Promise<any> {
    const ledgerKey = xdr.LedgerKey.contractData(
      new xdr.LedgerKeyContractData({
        contract: new Contract(contractId).address().toScAddress(),
        key: xdr.ScVal.scvLedgerKeyContractInstance(),
        durability: xdr.ContractDataDurability.persistent(),
      })
    );

    const response = await this.server.getLedgerEntries(ledgerKey);
    return response.entries;
  }

  getServer(): SorobanRpc.Server {
    return this.server;
  }
}
