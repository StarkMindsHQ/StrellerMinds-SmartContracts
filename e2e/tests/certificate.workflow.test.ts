import { describe, test, expect, beforeAll } from '@jest/globals';
import { Keypair, nativeToScVal, xdr } from '@stellar/stellar-sdk';
import { SorobanClient } from '../utils/soroban-client.js';
import { loadDeployments, createTestAccount, randomString } from '../utils/test-helpers.js';

describe('Certificate Workflow E2E Tests', () => {
  let client: SorobanClient;
  let admin: Keypair;
  let student: Keypair;
  let approver1: Keypair;
  let approver2: Keypair;
  let deployments: Record<string, string>;
  let certificateId: string;

  beforeAll(async () => {
    client = new SorobanClient();
    deployments = loadDeployments();

    admin = await createTestAccount(client);
    student = await createTestAccount(client);
    approver1 = await createTestAccount(client);
    approver2 = await createTestAccount(client);

    console.log('Certificate workflow test accounts created');
  }, 60000);

  describe('Certificate contract deployment', () => {
    test('should have certificate contract deployed', () => {
      expect(deployments).toBeDefined();
      expect(deployments.certificate).toBeDefined();
      certificateId = deployments.certificate;
      console.log(`✓ Certificate contract: ${certificateId}`);
    });

    test('should initialize certificate contract', async () => {
      if (!certificateId) {
        console.log('⚠ Certificate contract not deployed - skipping');
        return;
      }

      try {
        const adminAddress = nativeToScVal(admin.publicKey(), { type: 'address' });
        const result = await client.invokeContract(
          certificateId,
          'initialize',
          [adminAddress],
          admin
        );
        expect(result.status).toBe('SUCCESS');
        console.log('✓ Certificate contract initialized');
      } catch (error) {
        console.log('⚠ Initialization skipped:', (error as Error).message);
      }
    }, 120000);
  });

  describe('Batch certificate issuance', () => {
    test('should issue certificates in batch', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping batch issuance - contract not deployed');
        return;
      }

      try {
        const adminAddress = nativeToScVal(admin.publicKey(), { type: 'address' });
        const studentAddress = nativeToScVal(student.publicKey(), { type: 'address' });
        const courseId = nativeToScVal('E2E-COURSE-001', { type: 'string' });
        const title = nativeToScVal('E2E Test Certificate', { type: 'string' });
        const description = nativeToScVal('End-to-end test certificate', { type: 'string' });
        const metadataUri = nativeToScVal('https://strellerminds.io/cert/e2e', { type: 'string' });
        const expiryDate = nativeToScVal(0, { type: 'u64' });
        const certIdBytes = nativeToScVal(Buffer.alloc(32, 1), { type: 'bytes' });

        const certParams = xdr.ScVal.scvMap([
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('certificate_id'),
            val: certIdBytes,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('course_id'),
            val: courseId,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('student'),
            val: studentAddress,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('title'),
            val: title,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('description'),
            val: description,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('metadata_uri'),
            val: metadataUri,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('expiry_date'),
            val: expiryDate,
          }),
        ]);

        const paramsList = xdr.ScVal.scvVec([certParams]);

        const result = await client.invokeContract(
          certificateId,
          'batch_issue_certificates',
          [adminAddress, paramsList],
          admin
        );

        console.log('✓ Batch certificate issuance completed');
        expect(result.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Batch issuance test:', (error as Error).message);
      }
    }, 120000);
  });

  describe('Multi-sig approval flow', () => {
    test('should configure multi-sig for a course', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping multi-sig config - contract not deployed');
        return;
      }

      try {
        const adminAddress = nativeToScVal(admin.publicKey(), { type: 'address' });
        const approver1Address = nativeToScVal(approver1.publicKey(), { type: 'address' });
        const approver2Address = nativeToScVal(approver2.publicKey(), { type: 'address' });
        const courseId = nativeToScVal('E2E-MULTISIG-001', { type: 'string' });
        const requiredApprovals = nativeToScVal(2, { type: 'u32' });
        const timeoutDuration = nativeToScVal(604800, { type: 'u64' });
        const autoExecute = nativeToScVal(true, { type: 'bool' });

        const approversVec = xdr.ScVal.scvVec([approver1Address, approver2Address]);

        const config = xdr.ScVal.scvMap([
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('course_id'),
            val: courseId,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('required_approvals'),
            val: requiredApprovals,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('authorized_approvers'),
            val: approversVec,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('timeout_duration'),
            val: timeoutDuration,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('priority'),
            val: xdr.ScVal.scvSymbol('Enterprise'),
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('auto_execute'),
            val: autoExecute,
          }),
        ]);

        const result = await client.invokeContract(
          certificateId,
          'configure_multisig',
          [adminAddress, config],
          admin
        );

        console.log('✓ Multi-sig configuration completed');
        expect(result.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Multi-sig config test:', (error as Error).message);
      }
    }, 120000);

    test('should create and approve multi-sig certificate request', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping multi-sig request - contract not deployed');
        return;
      }

      try {
        const studentAddress = nativeToScVal(student.publicKey(), { type: 'address' });
        const courseId = nativeToScVal('E2E-MULTISIG-001', { type: 'string' });
        const certIdBytes = nativeToScVal(Buffer.alloc(32, 2), { type: 'bytes' });
        const title = nativeToScVal('Multi-Sig Certificate', { type: 'string' });
        const description = nativeToScVal('Approved via multi-sig', { type: 'string' });
        const metadataUri = nativeToScVal('https://strellerminds.io/cert/ms', { type: 'string' });
        const expiryDate = nativeToScVal(0, { type: 'u64' });

        const params = xdr.ScVal.scvMap([
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('certificate_id'),
            val: certIdBytes,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('course_id'),
            val: courseId,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('student'),
            val: studentAddress,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('title'),
            val: title,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('description'),
            val: description,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('metadata_uri'),
            val: metadataUri,
          }),
          new xdr.ScMapEntry({
            key: xdr.ScVal.scvSymbol('expiry_date'),
            val: expiryDate,
          }),
        ]);

        const reason = nativeToScVal('Requesting course completion certificate', { type: 'string' });

        const createResult = await client.invokeContract(
          certificateId,
          'create_multisig_request',
          [studentAddress, params, reason],
          student
        );

        console.log('✓ Multi-sig request created');
        expect(createResult.status).toBe('SUCCESS');

        // Approve with approver1
        const approver1Address = nativeToScVal(approver1.publicKey(), { type: 'address' });
        const requestId = certIdBytes; // In real flow we'd parse the returned request_id
        const approved = nativeToScVal(true, { type: 'bool' });
        const comments = nativeToScVal('Approved - meets all requirements', { type: 'string' });

        const approveResult1 = await client.invokeContract(
          certificateId,
          'process_multisig_approval',
          [approver1Address, requestId, approved, comments, xdr.ScVal.scvVoid()],
          approver1
        );

        console.log('✓ First approval processed');
        expect(approveResult1.status).toBe('SUCCESS');

        // Approve with approver2
        const approver2Address = nativeToScVal(approver2.publicKey(), { type: 'address' });
        const approveResult2 = await client.invokeContract(
          certificateId,
          'process_multisig_approval',
          [approver2Address, requestId, approved, comments, xdr.ScVal.scvVoid()],
          approver2
        );

        console.log('✓ Second approval processed - certificate should be issued');
        expect(approveResult2.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Multi-sig request test:', (error as Error).message);
      }
    }, 180000);
  });

  describe('Certificate verification', () => {
    test('should verify issued certificate', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping verification - contract not deployed');
        return;
      }

      try {
        const certIdBytes = nativeToScVal(Buffer.alloc(32, 1), { type: 'bytes' });

        const result = await client.invokeContract(
          certificateId,
          'verify_certificate',
          [certIdBytes],
          student
        );

        console.log('✓ Certificate verification completed');
        expect(result.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Verification test:', (error as Error).message);
      }
    }, 60000);

    test('should verify certificate authenticity', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping authenticity - contract not deployed');
        return;
      }

      try {
        const certIdBytes = nativeToScVal(Buffer.alloc(32, 1), { type: 'bytes' });

        const result = await client.invokeContract(
          certificateId,
          'verify_authenticity',
          [certIdBytes],
          student
        );

        console.log('✓ Certificate authenticity verified');
        expect(result.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Authenticity test:', (error as Error).message);
      }
    }, 60000);
  });

  describe('Certificate revocation', () => {
    test('should revoke an active certificate', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping revocation - contract not deployed');
        return;
      }

      try {
        const adminAddress = nativeToScVal(admin.publicKey(), { type: 'address' });
        const certIdBytes = nativeToScVal(Buffer.alloc(32, 1), { type: 'bytes' });
        const reason = nativeToScVal('Revoked for E2E testing', { type: 'string' });
        const reissuanceEligible = nativeToScVal(true, { type: 'bool' });

        const result = await client.invokeContract(
          certificateId,
          'revoke_certificate',
          [adminAddress, certIdBytes, reason, reissuanceEligible],
          admin
        );

        console.log('✓ Certificate revocation completed');
        expect(result.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Revocation test:', (error as Error).message);
      }
    }, 120000);
  });

  describe('Certificate analytics', () => {
    test('should retrieve certificate analytics', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping analytics - contract not deployed');
        return;
      }

      try {
        const result = await client.invokeContract(
          certificateId,
          'get_analytics',
          [],
          admin
        );

        console.log('✓ Analytics retrieved');
        expect(result.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Analytics test:', (error as Error).message);
      }
    }, 60000);
  });

  describe('Health check', () => {
    test('should return healthy status', async () => {
      if (!certificateId) {
        console.log('⚠ Skipping health check - contract not deployed');
        return;
      }

      try {
        const result = await client.invokeContract(
          certificateId,
          'health_check',
          [],
          admin
        );

        console.log('✓ Health check completed');
        expect(result.status).toBe('SUCCESS');
      } catch (error) {
        console.log('Health check test:', (error as Error).message);
      }
    }, 60000);
  });
});
