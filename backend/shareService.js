const crypto = require('crypto');
const Share = require('./models/Share');
const AccessLog = require('./models/AccessLog');

async function createShareLink({ resourceId, ownerId, expiresAt }) {
  const token = crypto.randomBytes(32).toString('hex');
  const tokenHash = crypto.createHash('sha256').update(token).digest('hex');
  const share = new Share({
    tokenHash,
    resourceId,
    ownerId,
    expiresAt: new Date(expiresAt),
    revoked: false
  });
  await share.save();
  return { token, url: `/share/${token}` };
}

async function getSharedResource(token, ip) {
  const tokenHash = crypto.createHash('sha256').update(token).digest('hex');
  const share = await Share.findOne({ tokenHash });
  if (!share) {
    await logAccess(tokenHash, ip, false);
    throw new Error('Invalid token');
  }
  if (share.revoked) {
    await logAccess(tokenHash, ip, false);
    throw new Error('Link revoked');
  }
  if (new Date() > share.expiresAt) {
    await logAccess(tokenHash, ip, false);
    throw new Error('Link expired');
  }
  await logAccess(tokenHash, ip, true);
  return { resourceId: share.resourceId, ownerId: share.ownerId };
}

async function revokeShare(token) {
  const tokenHash = crypto.createHash('sha256').update(token).digest('hex');
  await Share.updateOne({ tokenHash }, { revoked: true });
}

async function getAccessLogs(token) {
  const tokenHash = crypto.createHash('sha256').update(token).digest('hex');
  return await AccessLog.find({ tokenHash }).sort({ timestamp: -1 });
}

async function logAccess(tokenHash, ip, success) {
  const log = new AccessLog({
    tokenHash,
    ip,
    success
  });
  await log.save();
}

module.exports = {
  createShareLink,
  getSharedResource,
  revokeShare,
  getAccessLogs
};