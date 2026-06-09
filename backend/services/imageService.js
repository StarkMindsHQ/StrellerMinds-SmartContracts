const sharp = require('sharp');
const fs = require('fs').promises;
const path = require('path');
const { v4: uuidv4 } = require('uuid');
const CDNUploader = require('./cdnUploader');

class ImageService {
  constructor(config = {}) {
    this.config = {
      maxFileSize: config.maxFileSize || 50 * 1024 * 1024, // 50MB
      uploadDir: config.uploadDir || './uploads',
      cdnEnabled: config.cdnEnabled !== false,
      qualitySettings: config.qualitySettings || {
        webp: 75,
        jpeg: 80,
        png: 80
      },
      imageSizes: config.imageSizes || [
        { name: 'thumbnail', width: 150, height: 150 },
        { name: 'small', width: 400, height: 400 },
        { name: 'medium', width: 800, height: 800 },
        { name: 'large', width: 1200, height: 1200 },
        { name: 'original', width: null, height: null }
      ],
      ...config
    };
    
    this.cdnUploader = new CDNUploader(config.cdnConfig);
    this.metrics = {
      processedImages: 0,
      totalProcessingTime: 0,
      failedProcesses: 0
    };
  }

  /**
   * Main image processing pipeline with parallel execution
   * @param {Buffer|string} imageBuffer - Image file buffer or path
   * @param {Object} options - Processing options
   * @returns {Promise<Object>} Processed image metadata and URLs
   */
  async processImage(imageBuffer, options = {}) {
    const startTime = Date.now();
    let imageId = options.imageId || uuidv4();
    let metadata = {};

    try {
      // Validate input
      if (Buffer.isBuffer(imageBuffer) && imageBuffer.length > this.config.maxFileSize) {
        throw new Error(`File size exceeds maximum limit of ${this.config.maxFileSize / 1024 / 1024}MB`);
      }

      // Get original image metadata
      const image = sharp(imageBuffer);
      const originalMetadata = await image.metadata();
      
      metadata = {
        imageId,
        originalFormat: originalMetadata.format,
        originalWidth: originalMetadata.width,
        originalHeight: originalMetadata.height,
        originalSize: Buffer.isBuffer(imageBuffer) ? imageBuffer.length : 0,
        processedAt: new Date().toISOString(),
        variants: {}
      };

      // PARALLEL PROCESSING: Process all sizes and formats simultaneously
      const processingTasks = this._generateProcessingTasks(image, imageId, originalMetadata);
      const processedVariants = await Promise.all(processingTasks);

      // Aggregate results
      processedVariants.forEach(variant => {
        if (!metadata.variants[variant.size]) {
          metadata.variants[variant.size] = {};
        }
        metadata.variants[variant.size][variant.format] = {
          url: variant.url,
          localPath: variant.localPath,
          size: variant.size,
          bytes: variant.bytes,
          compressionRatio: ((1 - variant.bytes / metadata.originalSize) * 100).toFixed(2)
        };
      });

      // Upload to CDN if enabled
      if (this.config.cdnEnabled) {
        metadata = await this._uploadToCDN(metadata, imageId);
      }

      // Calculate metrics
      const processingTime = Date.now() - startTime;
      const totalOptimizedSize = Object.values(metadata.variants).reduce((sum, sizeGroup) => {
        return sum + Object.values(sizeGroup).reduce((formatSum, format) => formatSum + format.bytes, 0);
      }, 0);

      metadata.metrics = {
        processingTimeMs: processingTime,
        originalSizeKB: (metadata.originalSize / 1024).toFixed(2),
        optimizedSizeKB: (totalOptimizedSize / 1024).toFixed(2),
        reductionPercent: ((1 - totalOptimizedSize / metadata.originalSize) * 100).toFixed(2),
        processingSpeed: `${(processingTime / 1000).toFixed(2)}s`
      };

      this._updateMetrics(processingTime);
      this._logPerformance(metadata);

      // Success validation
      if (processingTime > 5000) {
        console.warn(`⚠️  Processing time exceeded 5 seconds: ${processingTime}ms`);
      }
      if (processingTime < 1000) {
        console.log(`✅ Performance target achieved: ${processingTime}ms < 1000ms`);
      }

      return metadata;
    } catch (error) {
      this.metrics.failedProcesses++;
      console.error('❌ Image processing failed:', error.message);
      throw new Error(`Image processing failed: ${error.message}`);
    }
  }

  /**
   * Generate parallel processing tasks for all image sizes and formats
   * @private
   */
  _generateProcessingTasks(image, imageId, metadata) {
    const tasks = [];

    // Process each size variant
    for (const sizeConfig of this.config.imageSizes) {
      // WebP format (primary)
      tasks.push(
        this._processVariant(image, imageId, sizeConfig, 'webp', metadata)
      );

      // JPEG fallback
      tasks.push(
        this._processVariant(image, imageId, sizeConfig, 'jpeg', metadata)
      );

      // Skip PNG for smaller sizes to reduce processing load
      if (!sizeConfig.name.includes('thumbnail')) {
        tasks.push(
          this._processVariant(image, imageId, sizeConfig, 'png', metadata)
        );
      }
    }

    return tasks;
  }

  /**
   * Process single image variant with specific size and format
   * @private
   */
  async _processVariant(image, imageId, sizeConfig, format, metadata) {
    try {
      let processor = image.clone();

      // Resize if dimensions specified
      if (sizeConfig.width && sizeConfig.height) {
        processor = processor.resize(sizeConfig.width, sizeConfig.height, {
          fit: 'cover',
          position: 'center',
          withoutEnlargement: true
        });
      }

      // Apply format-specific compression
      const quality = this.config.qualitySettings[format] || 80;

      switch (format) {
        case 'webp':
          processor = processor.webp({ quality, effort: 6 });
          break;
        case 'jpeg':
          processor = processor.jpeg({ quality, mozjpeg: true });
          break;
        case 'png':
          processor = processor.png({ 
            compressionLevel: 9,
            progressive: true
          });
          break;
      }

      // Get processed buffer
      const buffer = await processor.toBuffer();
      
      // Generate file path
      const fileName = `${imageId}-${sizeConfig.name}.${format}`;
      const localPath = path.join(this.config.uploadDir, imageId, fileName);

      // Create directory if needed
      await fs.mkdir(path.dirname(localPath), { recursive: true });

      // Write file locally
      await fs.writeFile(localPath, buffer);

      return {
        imageId,
        size: sizeConfig.name,
        format,
        bytes: buffer.length,
        localPath,
        url: `/images/${imageId}/${fileName}` // Will be replaced by CDN URL
      };
    } catch (error) {
      console.error(`Error processing variant ${sizeConfig.name}-${format}:`, error.message);
      throw error;
    }
  }

  /**
   * Upload processed images to CDN and update URLs
   * @private
   */
  async _uploadToCDN(metadata, imageId) {
    try {
      const uploadTasks = [];

      // Collect all files to upload
      for (const [sizeKey, formats] of Object.entries(metadata.variants)) {
        for (const [format, fileData] of Object.entries(formats)) {
          uploadTasks.push(
            this.cdnUploader.uploadFile(fileData.localPath, {
              imageId,
              size: sizeKey,
              format,
              mimeType: `image/${format}`
            }).then(cdnUrl => ({
              size: sizeKey,
              format,
              cdnUrl
            }))
          );
        }
      }

      // Execute all uploads in parallel
      const uploadResults = await Promise.all(uploadTasks);

      // Update metadata with CDN URLs
      uploadResults.forEach(result => {
        if (metadata.variants[result.size] && metadata.variants[result.size][result.format]) {
          metadata.variants[result.size][result.format].url = result.cdnUrl;
          metadata.variants[result.size][result.format].cdnUploaded = true;
        }
      });

      metadata.cdnUploadedAt = new Date().toISOString();
      return metadata;
    } catch (error) {
      console.error('CDN upload failed:', error.message);
      console.warn('Falling back to local URLs');
      return metadata;
    }
  }

  /**
   * Get optimized image srcset for responsive loading
   */
  getSrcSet(metadata, format = 'webp', fallbackFormat = 'jpeg') {
    const srcsetItems = [];

    for (const [sizeName, formats] of Object.entries(metadata.variants)) {
      const formatUrl = formats[format]?.url || formats[fallbackFormat]?.url;
      if (formatUrl) {
        const width = this._getWidthFromSize(sizeName);
        if (width) {
          srcsetItems.push(`${formatUrl} ${width}w`);
        }
      }
    }

    return srcsetItems.join(', ');
  }

  /**
   * Get low-quality image placeholder (LQIP) for progressive loading
   */
  async generateLQIP(imageBuffer, maxWidth = 20) {
    try {
      const lqip = await sharp(imageBuffer)
        .resize(maxWidth, maxWidth, { fit: 'cover' })
        .blur(2)
        .webp({ quality: 30 })
        .toBuffer();

      return `data:image/webp;base64,${lqip.toString('base64')}`;
    } catch (error) {
      console.error('LQIP generation failed:', error.message);
      return null;
    }
  }

  /**
   * Get extraction width from size name
   * @private
   */
  _getWidthFromSize(sizeName) {
    const sizeConfig = this.config.imageSizes.find(s => s.name === sizeName);
    return sizeConfig?.width || null;
  }

  /**
   * Update service metrics
   * @private
   */
  _updateMetrics(processingTime) {
    this.metrics.processedImages++;
    this.metrics.totalProcessingTime += processingTime;
  }

  /**
   * Log performance data for monitoring
   * @private
   */
  _logPerformance(metadata) {
    const metrics = metadata.metrics;
    console.log('📊 Image Processing Metrics:', {
      imageId: metadata.imageId,
      originalSizeKB: metrics.originalSizeKB,
      optimizedSizeKB: metrics.optimizedSizeKB,
      reductionPercent: `${metrics.reductionPercent}%`,
      processingTimeMs: metrics.processingTimeMs,
      processingSpeed: metrics.processingSpeed
    });
  }

  /**
   * Get service statistics
   */
  getMetrics() {
    return {
      ...this.metrics,
      averageProcessingTimeMs: (this.metrics.totalProcessingTime / this.metrics.processedImages).toFixed(2)
    };
  }

  /**
   * Cleanup old uploaded files
   */
  async cleanup(imageId) {
    try {
      const dirPath = path.join(this.config.uploadDir, imageId);
      await fs.rm(dirPath, { recursive: true, force: true });
      console.log(`✅ Cleaned up image ${imageId}`);
    } catch (error) {
      console.error(`Cleanup failed for ${imageId}:`, error.message);
    }
  }
}

module.exports = ImageService;
