const mongoose = require('mongoose');

const accessLogSchema = new mongoose.Schema({
  tokenHash: { type: String, required: true, index: true },
  ip: { type: String, required: true },
  timestamp: { type: Date, default: Date.now },
  success: { type: Boolean, required: true }
});

module.exports = mongoose.model('AccessLog', accessLogSchema);