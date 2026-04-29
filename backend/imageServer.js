const express = require('express');
const multer = require('multer');
const ImageService = require('./services/imageService');
const path = require('path');

class ImageServer {
  constructor(port = 3000, config = {}) {
    this.app = express();
    this.port = port;
    this.imageService = new ImageService({
      uploadDir: config.uploadDir || './uploads',
      cdnEnabled: config.cdnEnabled !== false,
      cdnConfig: config.cdnConfig || {
        provider: 'mock',
        cdnUrl: 'https://cdn.example.com'
      },
      ...config
    });

    this._setupMiddleware();
    this._setupRoutes();
  }

  /**
   * Setup Express middleware
   * @private
   */
  _setupMiddleware() {
    this.app.use(express.json());
    this.app.use(express.static('public'));

    // Multer for file upload
    const storage = multer.memoryStorage();
    const fileFilter = (req, file, cb) => {
      const validMimes = ['image/jpeg', 'image/png', 'image/webp', 'image/gif'];
      if (validMimes.includes(file.mimetype)) {
        cb(null, true);
      } else {
        cb(new Error('Invalid file type. Only JPEG, PNG, WebP, and GIF allowed.'));
      }
    };

    this.upload = multer({
      storage,
      fileFilter,
      limits: { fileSize: 50 * 1024 * 1024 }
    });

    // Error handling
    this.app.use((err, req, res, next) => {
      console.error('Server error:', err.message);
      res.status(500).json({ error: err.message });
    });
  }

  /**
   * Setup Express routes
   * @private
   */
  _setupRoutes() {
    /**
     * Health check
     */
    this.app.get('/health', (req, res) => {
      res.json({
        status: 'healthy',
        service: 'Image Optimization Service',
        metrics: this.imageService.getMetrics()
      });
    });

    /**
     * Upload and process image
     * POST /api/images/upload
     * Body: multipart/form-data with 'image' field
     */
    this.app.post('/api/images/upload', this.upload.single('image'), async (req, res, next) => {
      try {
        if (!req.file) {
          return res.status(400).json({ error: 'No image file provided' });
        }

        console.log(`📤 Processing image: ${req.file.originalname}`);

        // Process image with optimizations
        const metadata = await this.imageService.processImage(req.file.buffer, {
          imageId: req.body.imageId
        });

        // Generate LQIP for progressive loading
        const lqip = await this.imageService.generateLQIP(req.file.buffer);

        // Get responsive srcset
        const srcset = this.imageService.getSrcSet(metadata, 'webp', 'jpeg');

        res.json({
          success: true,
          data: {
            imageId: metadata.imageId,
            variants: metadata.variants,
            srcset,
            lqip,
            metrics: metadata.metrics,
            cdnUploadedAt: metadata.cdnUploadedAt
          }
        });
      } catch (error) {
        next(error);
      }
    });

    /**
     * Get image metadata
     * GET /api/images/:imageId
     */
    this.app.get('/api/images/:imageId', (req, res) => {
      res.json({
        message: 'Image metadata endpoint',
        imageId: req.params.imageId,
        note: 'Implement database lookup here to retrieve stored metadata'
      });
    });

    /**
     * Get service metrics
     * GET /api/metrics
     */
    this.app.get('/api/metrics', (req, res) => {
      res.json({
        imageProcessing: this.imageService.getMetrics(),
        cache: {
          info: 'CDN cache metrics would be retrieved from provider'
        },
        timestamp: new Date().toISOString()
      });
    });

    /**
     * Cleanup (delete processed images)
     * DELETE /api/images/:imageId
     */
    this.app.delete('/api/images/:imageId', async (req, res, next) => {
      try {
        await this.imageService.cleanup(req.params.imageId);
        res.json({ success: true, message: `Image ${req.params.imageId} deleted` });
      } catch (error) {
        next(error);
      }
    });

    /**
     * Frontend: Serve optimization demo
     */
    this.app.get('/', (req, res) => {
      res.send(this._getDemoHTML());
    });
  }

  /**
   * Get demo HTML with progressive image loading example
   * @private
   */
  _getDemoHTML() {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>Image Optimization Demo</title>
      <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          min-height: 100vh;
          padding: 40px 20px;
        }
        .container {
          max-width: 1200px;
          margin: 0 auto;
        }
        h1 {
          color: white;
          text-align: center;
          margin-bottom: 40px;
          font-size: 2.5em;
          text-shadow: 0 2px 10px rgba(0,0,0,0.2);
        }
        .section {
          background: white;
          border-radius: 12px;
          padding: 30px;
          margin-bottom: 30px;
          box-shadow: 0 10px 40px rgba(0,0,0,0.2);
        }
        h2 {
          color: #333;
          margin-bottom: 20px;
          font-size: 1.5em;
        }
        .upload-area {
          border: 3px dashed #667eea;
          border-radius: 8px;
          padding: 40px;
          text-align: center;
          background: #f8f9ff;
          cursor: pointer;
          transition: all 0.3s ease;
        }
        .upload-area:hover {
          border-color: #764ba2;
          background: #f0f2ff;
        }
        .upload-area.dragover {
          border-color: #764ba2;
          background: #e8ebff;
        }
        input[type="file"] {
          display: none;
        }
        button {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          border: none;
          padding: 12px 30px;
          border-radius: 6px;
          font-size: 1em;
          cursor: pointer;
          transition: transform 0.2s ease, box-shadow 0.2s ease;
        }
        button:hover {
          transform: translateY(-2px);
          box-shadow: 0 5px 20px rgba(102, 126, 234, 0.4);
        }
        button:active {
          transform: translateY(0);
        }
        .loading {
          display: none;
          text-align: center;
          padding: 20px;
        }
        .spinner {
          border: 4px solid #f3f3f3;
          border-top: 4px solid #667eea;
          border-radius: 50%;
          width: 40px;
          height: 40px;
          animation: spin 1s linear infinite;
          margin: 0 auto;
        }
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
        .results {
          display: none;
          margin-top: 30px;
        }
        .metrics {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 15px;
          margin-bottom: 30px;
        }
        .metric {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          padding: 20px;
          border-radius: 8px;
          text-align: center;
        }
        .metric-label {
          font-size: 0.9em;
          opacity: 0.9;
          margin-bottom: 5px;
        }
        .metric-value {
          font-size: 1.8em;
          font-weight: bold;
        }
        .images-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: 20px;
          margin-top: 20px;
        }
        .image-card {
          border-radius: 8px;
          overflow: hidden;
          background: #f5f5f5;
        }
        .image-wrapper {
          position: relative;
          background: #ddd;
          aspect-ratio: 1;
          overflow: hidden;
        }
        .image-placeholder {
          position: absolute;
          inset: 0;
          background-size: cover;
          background-position: center;
          filter: blur(10px);
          transition: opacity 0.3s ease;
        }
        .image-main {
          position: absolute;
          inset: 0;
          width: 100%;
          height: 100%;
          opacity: 0;
          transition: opacity 0.3s ease;
        }
        .image-main.loaded {
          opacity: 1;
        }
        .image-info {
          padding: 15px;
          background: white;
        }
        .image-info p {
          color: #666;
          font-size: 0.9em;
          margin: 5px 0;
        }
        .error {
          background: #fee;
          color: #c33;
          padding: 15px;
          border-radius: 6px;
          margin-top: 15px;
          display: none;
        }
        .error.show {
          display: block;
        }
      </style>
    </head>
    <body>
      <div class="container">
        <h1>🖼️ Image Optimization Demo</h1>

        <div class="section">
          <h2>Upload & Process Image</h2>
          <div class="upload-area" id="uploadArea">
            <p style="font-size: 1.2em; margin-bottom: 10px;">📁 Drop image here or click to select</p>
            <p style="color: #999;">Supported: JPEG, PNG, WebP (max 50MB)</p>
            <input type="file" id="fileInput" accept="image/*">
          </div>
          <div class="loading" id="loading">
            <div class="spinner"></div>
            <p style="margin-top: 10px; color: #667eea;">Processing image...</p>
          </div>
          <div class="error" id="error"></div>
          <div class="results" id="results">
            <div class="metrics" id="metricsContainer"></div>
            <div class="images-grid" id="imagesGrid"></div>
          </div>
        </div>
      </div>

      <script>
        const uploadArea = document.getElementById('uploadArea');
        const fileInput = document.getElementById('fileInput');
        const loading = document.getElementById('loading');
        const results = document.getElementById('results');
        const error = document.getElementById('error');
        const metricsContainer = document.getElementById('metricsContainer');
        const imagesGrid = document.getElementById('imagesGrid');

        // Drag and drop
        uploadArea.addEventListener('dragover', (e) => {
          e.preventDefault();
          uploadArea.classList.add('dragover');
        });

        uploadArea.addEventListener('dragleave', () => {
          uploadArea.classList.remove('dragover');
        });

        uploadArea.addEventListener('drop', async (e) => {
          e.preventDefault();
          uploadArea.classList.remove('dragover');
          const file = e.dataTransfer.files[0];
          if (file) await processFile(file);
        });

        uploadArea.addEventListener('click', () => fileInput.click());
        fileInput.addEventListener('change', (e) => {
          const file = e.target.files[0];
          if (file) processFile(file);
        });

        async function processFile(file) {
          error.classList.remove('show');
          results.style.display = 'none';
          loading.style.display = 'block';
          uploadArea.style.display = 'none';

          try {
            const formData = new FormData();
            formData.append('image', file);

            const response = await fetch('/api/images/upload', {
              method: 'POST',
              body: formData
            });

            if (!response.ok) throw new Error('Upload failed');
            const result = await response.json();

            displayResults(result.data);
            loading.style.display = 'none';
            results.style.display = 'block';
          } catch (err) {
            loading.style.display = 'none';
            error.textContent = '❌ Error: ' + err.message;
            error.classList.add('show');
            uploadArea.style.display = 'block';
          }
        }

        function displayResults(data) {
          const metrics = data.metrics;
          metricsContainer.innerHTML = \`
            <div class="metric">
              <div class="metric-label">Processing Time</div>
              <div class="metric-value">\${metrics.processingTimeMs}ms</div>
            </div>
            <div class="metric">
              <div class="metric-label">Original Size</div>
              <div class="metric-value">\${metrics.originalSizeKB}KB</div>
            </div>
            <div class="metric">
              <div class="metric-label">Optimized Size</div>
              <div class="metric-value">\${metrics.optimizedSizeKB}KB</div>
            </div>
            <div class="metric">
              <div class="metric-label">Size Reduction</div>
              <div class="metric-value">\${metrics.reductionPercent}%</div>
            </div>
          \`;

          imagesGrid.innerHTML = '';
          for (const [sizeName, formats] of Object.entries(data.variants)) {
            const webpFormat = formats.webp || formats.jpeg;
            if (webpFormat) {
              const card = document.createElement('div');
              card.className = 'image-card';
              card.innerHTML = \`
                <div class="image-wrapper">
                  <img class="image-placeholder" src="\${data.lqip}" alt="blur">
                  <img class="image-main" src="\${webpFormat.url}" alt="\${sizeName}" loading="lazy">
                </div>
                <div class="image-info">
                  <p><strong>\${sizeName}</strong></p>
                  <p>Size: \${webpFormat.size}KB</p>
                  <p>Format: \${webpFormat.format}</p>
                  <p>Compression: \${webpFormat.compressionRatio}%</p>
                </div>
              \`;
              
              const mainImg = card.querySelector('.image-main');
              mainImg.addEventListener('load', () => {
                mainImg.classList.add('loaded');
              });
              
              imagesGrid.appendChild(card);
            }
          }
        }
      </script>
    </body>
    </html>
    `;
  }

  /**
   * Start the server
   */
  start() {
    this.app.listen(this.port, () => {
      console.log(`\n✅ Image Optimization Server running on http://localhost:${this.port}`);
      console.log('📊 Health check: http://localhost:${this.port}/health');
      console.log('🚀 API endpoint: POST http://localhost:${this.port}/api/images/upload\n');
    });
  }
}

module.exports = ImageServer;

// If running directly
if (require.main === module) {
  const server = new ImageServer(3000, {
    uploadDir: './uploads',
    cdnEnabled: true,
    cdnConfig: {
      provider: 'mock', // Change to 'cloudflare', 's3', or 'azure' for real CDN
      cdnUrl: 'https://cdn.example.com'
    }
  });

  server.start();
}
