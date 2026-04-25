# TypeScript Code Examples

Code examples for interacting with StrellerMinds smart contracts using TypeScript and the Stellar SDK.

## Prerequisites

```bash
npm install @stellar/stellar-sdk
```

## Setup

```typescript
import { Networks, Keypair, Contract, SorobanDataProvider } from '@stellar/stellar-sdk';

const networkPassphrase = Networks.TESTNET;
const rpcUrl = 'https://soroban-testnet.stellar.org';

// Create contract client
const contractId = 'CDLZFC3SYJBDYDAJSKJWZCF7HGH4GZ3C6DBJJTYWA7WZ4WMZBFHCNPOB';
const contract = new Contract(contractId);

// User keypair
const userKeypair = Keypair.fromSecret('SDRXE2BURVHYV6KBB2SSE5FXQPCS7K4LN5YSMC4T3JNB2CECZ7RLLOA');
```

## Token Contract

### Initialize and Mint

```typescript
import { u64 } from '@stellar/stellar-sdk';

async function initializeAndMint(
  contract: Contract,
  admin: string,
  recipient: string,
  amount: bigint
) {
  // Initialize
  await contract.call('initialize', admin);

  // Mint tokens
  await contract.call('mint', recipient, u64.unsafeFromBigInt(amount));

  console.log(`Minted ${amount} tokens to ${recipient}`);
}
```

### Transfer Tokens

```typescript
async function transferTokens(
  contract: Contract,
  from: Keypair,
  to: string,
  amount: bigint
) {
  const result = await contract.call(
    'transfer',
    from.publicKey(),  // from (requires auth)
    to,                 // to
    u64.unsafeFromBigInt(amount)
  );

  console.log('Transfer successful:', result);
}
```

### Check Balance

```typescript
async function checkBalance(contract: Contract, account: string) {
  const balance = await contract.call('balance', account);
  console.log(`Balance: ${balance}`);
  return balance;
}
```

## Certificate Contract

### Initialize and Issue Certificate

```typescript
async function issueCertificate(
  contract: Contract,
  admin: Keypair,
  student: string,
  courseId: string,
  certificateId: Uint8Array
) {
  // Initialize (admin only)
  await contract.call('initialize', admin.publicKey());

  // Create issuance request
  const params = {
    certificate_id: certificateId,
    course_id: courseId,
    student: student,
    title: 'Course Completion Certificate',
    description: 'Successfully completed the course',
    metadata_uri: new Uint8Array(32),
    expiry_date: 0n,
  };

  const requestId = await contract.call(
    'create_multisig_request',
    student,
    params,
    'Certificate issuance request'
  );

  // Approve (in real scenario, multiple approvers needed)
  await contract.call(
    'process_multisig_approval',
    approver1.publicKey(),
    requestId,
    true,
    'Approved',
    null
  );

  // Execute
  await contract.call(
    'execute_multisig_request',
    admin.publicKey(),
    requestId
  );

  console.log('Certificate issued:', certificateId);
}
```

### Verify Certificate

```typescript
async function verifyCertificate(
  contract: Contract,
  certificateId: Uint8Array
): Promise<boolean> {
  const isValid: boolean = await contract.call(
    'verify_certificate',
    certificateId
  );

  console.log('Certificate valid:', isValid);
  return isValid;
}
```

### Batch Issue

```typescript
async function batchIssueCertificates(
  contract: Contract,
  admin: string,
  paramsList: any[]
) {
  const result = await contract.call(
    'batch_issue_certificates',
    admin,
    paramsList
  );

  console.log('Batch result:', result);
  return result;
}
```

## Progress Contract

### Record Progress

```typescript
async function recordProgress(
  contract: Contract,
  student: Keypair,
  courseId: string,
  progress: number
) {
  try {
    await contract.call(
      'record_progress',
      student.publicKey(),  // requires auth
      courseId,
      progress
    );
    console.log(`Recorded progress: ${progress}%`);
  } catch (e) {
    if (e.message.includes('Rate limit')) {
      console.log('Rate limit exceeded. Try again tomorrow.');
    }
    throw e;
  }
}
```

### Get Progress

```typescript
async function getProgress(
  contract: Contract,
  student: string,
  courseId: string
): Promise<number> {
  const progress = await contract.call(
    'get_progress',
    student,
    courseId
  );

  console.log(`Progress: ${progress}%`);
  return Number(progress);
}
```

### Get Student's Courses

```typescript
async function getStudentCourses(contract: Contract, student: string) {
  const courses = await contract.call('get_student_courses', student);
  console.log('Student courses:', courses);
  return courses;
}
```

## Assessment Contract

### Create Assessment

```typescript
async function createAssessment(
  contract: Contract,
  admin: string,
  title: string,
  courseId: string
) {
  const metadata = {
    title,
    description: 'Course assessment',
    course_id: courseId,
  };

  const config = {
    time_limit_seconds: 3600n,
    max_score: 100n,
    passing_score: 70n,
    randomization_seed: 0n,
  };

  const assessmentId = await contract.call(
    'create_assessment',
    admin,
    metadata,
    config,
    []  // initial question IDs
  );

  console.log('Assessment created:', assessmentId);
  return Number(assessmentId);
}
```

### Add Questions

```typescript
async function addQuestions(
  contract: Contract,
  admin: string,
  assessmentId: number
) {
  const questions = [
    {
      question_id: 1n,
      question_type: 'multiple_choice',
      prompt: 'What is the capital of France?',
      options: ['London', 'Paris', 'Berlin', 'Madrid'],
      correct_answer: 'Paris',
      points: 10n,
    },
  ];

  await contract.call(
    'add_questions',
    admin,
    BigInt(assessmentId),
    questions
  );

  console.log('Questions added');
}
```

### Submit Answers

```typescript
async function submitAnswers(
  contract: Contract,
  student: Keypair,
  submissionId: string,
  answers: { question_id: bigint; answer: string }[]
) {
  try {
    await contract.call(
      'submit_answers',
      student.publicKey(),  // requires auth
      submissionId,
      answers
    );
    console.log('Answers submitted');
  } catch (e) {
    if (e.message.includes('Rate limit')) {
      console.log('Daily answer limit reached.');
    }
    throw e;
  }
}
```

## Gamification Contract

### Record Activity

```typescript
async function recordActivity(
  contract: Contract,
  user: Keypair,
  activityType: string,
  courseId: string
) {
  const activity = {
    activity_type: activityType,
    course_id: courseId,
    module_id: 'module_1',
    timestamp: Math.floor(Date.now() / 1000),
  };

  const achievementIds = await contract.call(
    'record_activity',
    user.publicKey(),  // requires auth
    activity
  );

  console.log('Achievements earned:', achievementIds);
  return achievementIds;
}
```

### Get User Profile

```typescript
async function getUserProfile(contract: Contract, user: string) {
  const profile = await contract.call('get_user_profile', user);
  console.log('User profile:', profile);
  return profile;
}
```

### Create Guild

```typescript
async function createGuild(
  contract: Contract,
  creator: Keypair,
  name: string,
  description: string
) {
  const guildId = await contract.call(
    'create_guild',
    creator.publicKey(),  // requires auth
    name,
    description,
    50,   // max_members
    true  // is_public
  );

  console.log('Guild created:', guildId);
  return Number(guildId);
}
```

### Join Guild

```typescript
async function joinGuild(
  contract: Contract,
  user: Keypair,
  guildId: number
) {
  await contract.call(
    'join_guild',
    user.publicKey(),  // requires auth
    BigInt(guildId)
  );

  console.log('Joined guild:', guildId);
}
```

### Endorse Peer

```typescript
async function endorsePeer(
  contract: Contract,
  endorser: Keypair,
  endorsee: string,
  skill: string
) {
  await contract.call(
    'endorse_peer',
    endorser.publicKey(),  // requires auth
    endorsee,
    skill
  );

  console.log(`Endorsed ${endorsee} for ${skill}`);
}
```

### Get Leaderboard

```typescript
async function getLeaderboard(
  contract: Contract,
  category: string = 'xp',
  limit: number = 50
) {
  const entries = await contract.call(
    'get_leaderboard',
    category,
    BigInt(limit)
  );

  console.log('Leaderboard:', entries);
  return entries;
}
```

## Error Handling

```typescript
async function handleErrors() {
  try {
    const result = await contract.call('verify_certificate', certificateId);
    console.log('Valid:', result);
  } catch (e) {
    if (e.message.includes('CertificateNotFound')) {
      console.log('Certificate not found');
    } else if (e.message.includes('Unauthorized')) {
      console.log('Not authorized');
    } else {
      console.error('Error:', e.message);
    }
  }
}
```

## Health Check

```typescript
async function healthCheck(contract: Contract) {
  const report = await contract.call('health_check');
  console.log('Health report:', report);
  return report;
}
```