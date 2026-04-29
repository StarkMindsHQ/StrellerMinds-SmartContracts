const express = require('express');
const mongoose = require('mongoose');
const shareRoutes = require('./shareRoutes');

const app = express();

app.use(express.json());
app.use('/', shareRoutes);

mongoose.connect(process.env.MONGODB_URI || 'mongodb://localhost:27017/shares');

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => console.log(`Server running on port ${PORT}`));