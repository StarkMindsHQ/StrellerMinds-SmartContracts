import { Contract, SorobanRpc, TransactionBuilder, Networks, BASE_FEE, xdr } from 'soroban-client';
import { Certificate, SharedCredentialPayload } from '../types';
import { OfflineStorage } from './offlineStorage';

const CONTRACT_ID = process.env.EXPO_PUBLIC_CERTIFICATE_CONTRACT_ID || '';
const RPC_URL = process.env.EXPO_PUBLIC_SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = Networks.TESTNET;

export class CertificateService {
  private server: SorobanRpc.Server;
  private contract: Contract;

  constructor() {
    this.server = new SorobanRpc.Server(RPC_URL);
    this.contract = new Contract(CONTRACT_ID);
  }

  async getStudentCertificates(studentAddress: string, sourceSecret: string): Promise<Certificate[]> {
    try {
      const keypair = this.keypairFromSecret(sourceSecret);
      const account = await this.server.getAccount(keypair.publicKey());

      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call('get_student_certificates', this.addressToScVal(studentAddress))
        )
        .setTimeout(30)
        .build();

      const result = await this.server.simulateTransaction(tx);
      const certs = this.parseCertificates(result);

      // Update offline cache
      await OfflineStorage.saveCertificates(certs);

      return certs;
    } catch (error) {
      console.error('Failed to fetch certificates, falling back to cache', error);
      return OfflineStorage.getCachedCertificates();
    }
  }

  async getCertificate(certificateId: string, sourceSecret: string): Promise<Certificate | null> {
    // Try offline first
    const cached = await OfflineStorage.getCertificateById(certificateId);
    if (cached) return cached;

    try {
      const keypair = this.keypairFromSecret(sourceSecret);
      const account = await this.server.getAccount(keypair.publicKey());

      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call('get_certificate', this.bytesToScVal(certificateId))
        )
        .setTimeout(30)
        .build();

      const result = await this.server.simulateTransaction(tx);
      const cert = this.parseCertificate(result);

      if (cert) {
        await OfflineStorage.addOrUpdateCertificate(cert);
      }

      return cert;
    } catch (error) {
      console.error('Failed to fetch certificate', error);
      return null;
    }
  }

  async verifyCertificate(certificateId: string, sourceSecret: string): Promise<boolean> {
    try {
      const keypair = this.keypairFromSecret(sourceSecret);
      const account = await this.server.getAccount(keypair.publicKey());

      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call('verify_certificate', this.bytesToScVal(certificateId))
        )
        .setTimeout(30)
        .build();

      const result = await this.server.simulateTransaction(tx);
      return this.parseBoolean(result);
    } catch {
      // Offline fallback: check cache validity
      const cached = await OfflineStorage.getCertificateById(certificateId);
      if (!cached) return false;
      if (cached.status !== 'active') return false;
      if (cached.expiryDate > 0 && cached.expiryDate < Date.now() / 1000) return false;
      return true;
    }
  }

  async shareCertificate(
    certificateId: string,
    platform: string,
    verificationUrl: string,
    sourceSecret: string
  ): Promise<void> {
    const keypair = this.keypairFromSecret(sourceSecret);
    const account = await this.server.getAccount(keypair.publicKey());

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        this.contract.call(
          'share_certificate',
          this.addressToScVal(keypair.publicKey()),
          this.bytesToScVal(certificateId),
          this.stringToScVal(platform),
          this.stringToScVal(verificationUrl)
        )
      )
      .setTimeout(30)
      .build();

    await this.server.simulateTransaction(tx);
    // In production, sign and submit the transaction
  }

  generateQRPayload(certificate: Certificate): SharedCredentialPayload {
    return {
      certificateId: certificate.certificateId,
      student: certificate.student,
      title: certificate.title,
      issuer: certificate.issuer,
      blockchainAnchor: certificate.blockchainAnchor || '',
      timestamp: Date.now(),
    };
  }

  parseQRPayload(jsonString: string): SharedCredentialPayload | null {
    try {
      return JSON.parse(jsonString) as SharedCredentialPayload;
    } catch {
      return null;
    }
  }

  private keypairFromSecret(secret: string) {
    // @ts-ignore
    const { Keypair } = require('soroban-client');
    return Keypair.fromSecret(secret);
  }

  private addressToScVal(address: string) {
    // @ts-ignore
    const { Address } = require('soroban-client');
    return new Address(address).toScVal();
  }

  private bytesToScVal(hex: string) {
    // @ts-ignore
    const { xdr } = require('soroban-client');
    return xdr.ScVal.scvBytes(Buffer.from(hex, 'hex'));
  }

  private stringToScVal(str: string) {
    // @ts-ignore
    const { xdr } = require('soroban-client');
    return xdr.ScVal.scvString(str);
  }

  private parseCertificates(result: any): Certificate[] {
    // Simplified parsing - in production use generated SDK or proper XDR parsing
    return [];
  }

  private parseCertificate(result: any): Certificate | null {
    // Simplified parsing
    return null;
  }

  private parseBoolean(result: any): boolean {
    // Simplified parsing
    return false;
  }
}
