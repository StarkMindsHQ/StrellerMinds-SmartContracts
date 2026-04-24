# Analytics Dashboard

## Description

Build comprehensive analytics dashboard for platform administrators.

## Metrics Needed

- Active users
- Certificates issued
- Revenue metrics
- Performance metrics
- System health

## Acceptance Criteria

- Dashboard loads <2s
- Data updates real-time
- Export functionality working

## Implementation Overview

The analytics dashboard will be implemented as a web-based application using React.js for the frontend, with data sourced from the existing smart contracts via API endpoints. The dashboard will provide real-time metrics visualization and export capabilities.

## Architecture

### Components
- **Frontend**: React application with Chart.js for visualizations
- **Backend API**: Node.js/Express server to aggregate data from smart contracts
- **Data Sources**: Existing analytics smart contract and related contracts (certificate, token, etc.)
- **Database**: Optional caching layer with Redis for performance

### Data Flow
1. Smart contracts emit events for key metrics
2. Off-chain indexer listens to events and stores data
3. API server queries indexed data
4. Frontend fetches data via REST/WebSocket APIs
5. Real-time updates via WebSocket connections

## Data Collection

### Active Users
- Track unique wallet addresses interacting with platform contracts
- Daily/weekly/monthly active user counts
- User engagement metrics (transactions per user)

### Certificates Issued
- Total certificates minted
- Certificates by type/category
- Issuance trends over time

### Revenue Metrics
- Token transfers to platform treasury
- Transaction fees collected
- Revenue by service type

### Performance Metrics
- Transaction throughput
- Gas usage statistics
- Contract execution times

### System Health
- Contract status (active/inactive)
- Error rates
- Network connectivity

## Dashboard Components

### Main Dashboard Layout
```
+------------------+--------------------+
| Header           | Real-time Stats    |
+------------------+--------------------+
| Metrics Cards    | Charts/Graphs      |
|                  |                    |
| Active Users     | User Growth Chart  |
| Certificates     | Revenue Trends     |
| Revenue          | Performance        |
| System Health    | System Health      |
+------------------+--------------------+
| Export Controls  | Data Tables        |
+------------------+--------------------+
```

### Key Components
1. **Metrics Cards**: Display current values for each metric
2. **Time Series Charts**: Historical data visualization
3. **Real-time Updates**: WebSocket integration for live data
4. **Export Functionality**: CSV/PDF export of data
5. **Filters**: Date ranges, metric types

## Code Implementation

### Frontend (React)

```jsx
// Dashboard.js
import React, { useState, useEffect } from 'react';
import { Line, Bar, Pie } from 'react-chartjs-2';
import io from 'socket.io-client';

const Dashboard = () => {
  const [metrics, setMetrics] = useState({});
  const [socket, setSocket] = useState(null);

  useEffect(() => {
    // Initial data load
    fetchMetrics();

    // Real-time updates
    const newSocket = io('http://localhost:3001');
    newSocket.on('metrics-update', (data) => {
      setMetrics(prev => ({ ...prev, ...data }));
    });
    setSocket(newSocket);

    return () => newSocket.close();
  }, []);

  const fetchMetrics = async () => {
    try {
      const response = await fetch('/api/metrics');
      const data = await response.json();
      setMetrics(data);
    } catch (error) {
      console.error('Failed to fetch metrics:', error);
    }
  };

  const exportData = async (format) => {
    try {
      const response = await fetch(`/api/export?format=${format}`);
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `analytics.${format}`;
      a.click();
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  return (
    <div className="dashboard">
      <header>
        <h1>Platform Analytics Dashboard</h1>
        <div className="real-time-indicator">
          Live Data: {socket ? 'Connected' : 'Disconnected'}
        </div>
      </header>

      <div className="metrics-grid">
        <div className="metric-card">
          <h3>Active Users</h3>
          <div className="value">{metrics.activeUsers || 0}</div>
        </div>
        <div className="metric-card">
          <h3>Certificates Issued</h3>
          <div className="value">{metrics.certificatesIssued || 0}</div>
        </div>
        <div className="metric-card">
          <h3>Revenue</h3>
          <div className="value">${metrics.revenue || 0}</div>
        </div>
        <div className="metric-card">
          <h3>System Health</h3>
          <div className="value">{metrics.systemHealth || 'Good'}</div>
        </div>
      </div>

      <div className="charts-section">
        <div className="chart-container">
          <h3>User Growth</h3>
          <Line data={metrics.userGrowthData} />
        </div>
        <div className="chart-container">
          <h3>Revenue Trends</h3>
          <Bar data={metrics.revenueData} />
        </div>
      </div>

      <div className="export-section">
        <button onClick={() => exportData('csv')}>Export CSV</button>
        <button onClick={() => exportData('pdf')}>Export PDF</button>
      </div>
    </div>
  );
};

export default Dashboard;
```

### Backend API (Node.js/Express)

```javascript
// server.js
const express = require('express');
const http = require('http');
const socketIo = require('socket.io');
const { Web3 } = require('web3');
const AnalyticsContract = require('./contracts/Analytics.json');

const app = express();
const server = http.createServer(app);
const io = socketIo(server);

const web3 = new Web3('ws://localhost:8545'); // Connect to blockchain
const analyticsContract = new web3.eth.Contract(
  AnalyticsContract.abi,
  '0x...' // Contract address
);

// In-memory cache for performance
let metricsCache = {};
const CACHE_DURATION = 30000; // 30 seconds

// API endpoints
app.get('/api/metrics', async (req, res) => {
  try {
    const metrics = await getMetrics();
    res.json(metrics);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.get('/api/export', async (req, res) => {
  const format = req.query.format;
  const data = await getExportData();

  if (format === 'csv') {
    res.setHeader('Content-Type', 'text/csv');
    res.setHeader('Content-Disposition', 'attachment; filename="analytics.csv"');
    // Convert data to CSV format
    res.send(convertToCSV(data));
  } else if (format === 'pdf') {
    // Generate PDF
    const pdfBuffer = await generatePDF(data);
    res.setHeader('Content-Type', 'application/pdf');
    res.send(pdfBuffer);
  }
});

// Real-time updates
setInterval(async () => {
  const newMetrics = await getMetrics();
  const changes = getChanges(metricsCache, newMetrics);
  if (Object.keys(changes).length > 0) {
    io.emit('metrics-update', changes);
    metricsCache = newMetrics;
  }
}, 5000); // Update every 5 seconds

async function getMetrics() {
  // Query smart contracts for metrics
  const activeUsers = await analyticsContract.methods.getActiveUsers().call();
  const certificatesIssued = await analyticsContract.methods.getCertificatesIssued().call();
  const revenue = await analyticsContract.methods.getRevenue().call();
  const systemHealth = await checkSystemHealth();

  return {
    activeUsers: parseInt(activeUsers),
    certificatesIssued: parseInt(certificatesIssued),
    revenue: parseFloat(web3.utils.fromWei(revenue, 'ether')),
    systemHealth,
    timestamp: Date.now()
  };
}

async function checkSystemHealth() {
  try {
    // Check contract responsiveness
    await analyticsContract.methods.getActiveUsers().call();
    return 'Good';
  } catch (error) {
    return 'Degraded';
  }
}

function getChanges(oldMetrics, newMetrics) {
  const changes = {};
  for (const key in newMetrics) {
    if (oldMetrics[key] !== newMetrics[key]) {
      changes[key] = newMetrics[key];
    }
  }
  return changes;
}

server.listen(3001, () => {
  console.log('Server running on port 3001');
});
```

### Smart Contract Integration

Assuming the analytics contract has methods like:

```rust
// In analytics contract
#[ink(message)]
pub fn get_active_users(&self) -> u64 {
    // Implementation to count active users
}

#[ink(message)]
pub fn get_certificates_issued(&self) -> u64 {
    // Implementation to count issued certificates
}

#[ink(message)]
pub fn get_revenue(&self) -> Balance {
    // Implementation to calculate revenue
}
```

## Performance Optimization

### Loading <2s
- Implement data caching with Redis
- Use lazy loading for charts
- Optimize API responses with compression
- Pre-render critical metrics

### Real-time Updates
- WebSocket connections for live data
- Event-driven architecture
- Debounced updates to prevent UI thrashing

### Export Functionality
- Background processing for large exports
- Streaming for large datasets
- Multiple format support (CSV, PDF, JSON)

## Testing

### Unit Tests
- Test metric calculations
- Test API endpoints
- Test export functionality

### Integration Tests
- End-to-end dashboard loading
- Real-time update verification
- Export feature testing

### Performance Tests
- Load testing for <2s requirement
- Stress testing for real-time updates
- Export performance validation

## Deployment

1. Deploy smart contracts to testnet/mainnet
2. Set up indexer for event listening
3. Deploy backend API to cloud (AWS/GCP/Azure)
4. Deploy frontend to CDN (Vercel/Netlify)
5. Configure monitoring and alerting