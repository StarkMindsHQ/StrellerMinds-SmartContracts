const mongoose = require('mongoose');

const shareSchema = new mongoose.Schema({
  tokenHash: { type: String, required: true, unique: true, index: true },
  resourceId: { type: String, required: true },
  ownerId: { type: String, required: true },
  expiresAt: { type: Date, required: true },
  revoked: { type: Boolean, default: false },
  createdAt: { type: Date, default: Date.now }
});

module.exports = mongoose.model('Share', shareSchema);