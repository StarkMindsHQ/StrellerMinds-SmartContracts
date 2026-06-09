const express = require('express');
const router = express.Router();
const shareService = require('./shareService');

// Placeholder auth middleware - replace with actual auth
const auth = (req, res, next) => {
  req.user = { id: 'testOwner' }; // Replace with actual user from auth
  next();
};

router.post('/share', auth, async (req, res) => {
  try {
    const { resourceId, expiresAt } = req.body;
    const ownerId = req.user.id;
    const result = await shareService.createShareLink({ resourceId, ownerId, expiresAt });
    res.json(result);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

router.get('/share/:token', async (req, res) => {
  try {
    const { token } = req.params;
    const ip = req.ip || req.connection.remoteAddress;
    const resource = await shareService.getSharedResource(token, ip);
    res.json(resource);
  } catch (error) {
    res.status(403).json({ error: error.message });
  }
});

router.delete('/share/:token', auth, async (req, res) => {
  try {
    const { token } = req.params;
    await shareService.revokeShare(token);
    res.json({ message: 'Revoked' });
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

router.get('/share/:token/logs', auth, async (req, res) => {
  try {
    const { token } = req.params;
    const logs = await shareService.getAccessLogs(token);
    res.json(logs);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

module.exports = router;