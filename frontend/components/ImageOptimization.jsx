import React, { useState, useCallback, useEffect } from 'react';

/**
 * Progressive Image Component with LQIP (Low-Quality Image Placeholder)
 * Features:
 * - Lazy loading
 * - Blurred placeholder
 * - Responsive srcset
 * - No layout shift
 * - Accessibility support
 */
export const ProgressiveImage = ({
  imageId,
  variants,
  lqip,
  srcset,
  alt = 'Optimized image',
  sizes = '(max-width: 600px) 100vw, (max-width: 1200px) 50vw, 33vw',
  onLoad = () => {},
  onError = () => {}
}) => {
  const [isLoaded, setIsLoaded] = useState(false);
  const [hasError, setHasError] = useState(false);

  const handleLoad = useCallback(() => {
    setIsLoaded(true);
    onLoad();
  }, [onLoad]);

  const handleError = useCallback(() => {
    setHasError(true);
    onError();
  }, [onError]);

  // Get the largest available WebP or fallback to JPEG
  const mainSrc = variants.large?.webp?.url ||
    variants.medium?.webp?.url ||
    variants.original?.webp?.url ||
    variants.large?.jpeg?.url ||
    variants.medium?.jpeg?.url;

  if (!mainSrc) {
    return <div role="img" aria-label={alt}>Image not available</div>;
  }

  return (
    <div
      style={{
        position: 'relative',
        overflow: 'hidden',
        backgroundColor: '#f0f0f0'
      }}
      className="progressive-image-container"
    >
      {/* Blurred placeholder (LQIP) */}
      {lqip && (
        <img
          src={lqip}
          alt={`${alt} placeholder`}
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            opacity: isLoaded ? 0 : 1,
            filter: 'blur(10px)',
            transition: 'opacity 0.3s ease',
            pointerEvents: 'none'
          }}
          aria-hidden="true"
        />
      )}

      {/* Main image with responsive srcset */}
      <img
        id={`img-${imageId}`}
        src={mainSrc}
        srcSet={srcset}
        sizes={sizes}
        alt={alt}
        loading="lazy"
        onLoad={handleLoad}
        onError={handleError}
        style={{
          width: '100%',
          height: 'auto',
          display: 'block',
          opacity: isLoaded ? 1 : 0,
          transition: 'opacity 0.3s ease'
        }}
      />

      {/* Error state */}
      {hasError && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: '#f5f5f5',
            color: '#999',
            fontSize: '0.9em'
          }}
        >
          Image failed to load
        </div>
      )}
    </div>
  );
};

/**
 * Image Upload Component with processing feedback
 * Features:
 * - Drag and drop
 * - Progress indication
 * - Error handling
 * - Response display
 */
export const ImageUploader = ({
  apiEndpoint = '/api/images/upload',
  onSuccess = () => {},
  onError = () => {},
  maxFileSize = 50 * 1024 * 1024
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [progress, setProgress] = useState(0);
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);

  const handleDragOver = useCallback((e) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = (e) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const processFile = useCallback(async (file) => {
    setError(null);
    setResult(null);

    // Validation
    if (!file.type.startsWith('image/')) {
      setError('Please upload an image file');
      onError(new Error('Invalid file type'));
      return;
    }

    if (file.size > maxFileSize) {
      setError(`File size exceeds ${maxFileSize / 1024 / 1024}MB limit`);
      onError(new Error('File too large'));
      return;
    }

    setIsProcessing(true);
    setProgress(0);

    try {
      const formData = new FormData();
      formData.append('image', file);

      // Simulate progress
      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 10, 90));
      }, 200);

      const response = await fetch(apiEndpoint, {
        method: 'POST',
        body: formData
      });

      clearInterval(progressInterval);

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Upload failed');
      }

      const data = await response.json();
      setProgress(100);
      setResult(data.data);
      onSuccess(data.data);
    } catch (err) {
      setError(err.message);
      onError(err);
    } finally {
      setIsProcessing(false);
    }
  }, [apiEndpoint, maxFileSize, onSuccess, onError]);

  const handleDrop = (e) => {
    e.preventDefault();
    setIsDragging(false);
    const file = e.dataTransfer.files[0];
    if (file) processFile(file);
  };

  const handleFileSelect = (e) => {
    const file = e.target.files?.[0];
    if (file) processFile(file);
  };

  return (
    <div className="image-uploader">
      {/* Upload Area */}
      {!isProcessing && !result && (
        <div
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          style={{
            border: `3px dashed ${isDragging ? '#667eea' : '#ccc'}`,
            borderRadius: '8px',
            padding: '40px',
            textAlign: 'center',
            backgroundColor: isDragging ? '#f0f2ff' : '#f9f9f9',
            cursor: 'pointer',
            transition: 'all 0.3s ease'
          }}
        >
          <p style={{ fontSize: '1.2em', marginBottom: '10px' }}>
            📁 Drop image here or click to select
          </p>
          <p style={{ color: '#999', fontSize: '0.9em' }}>
            Supported: JPEG, PNG, WebP (max {maxFileSize / 1024 / 1024}MB)
          </p>
          <input
            type="file"
            accept="image/*"
            onChange={handleFileSelect}
            style={{ display: 'none' }}
            id="file-input"
          />
          <label
            htmlFor="file-input"
            style={{
              display: 'inline-block',
              marginTop: '15px',
              padding: '10px 20px',
              backgroundColor: '#667eea',
              color: 'white',
              borderRadius: '6px',
              cursor: 'pointer'
            }}
          >
            Select File
          </label>
        </div>
      )}

      {/* Processing State */}
      {isProcessing && (
        <div style={{ textAlign: 'center', padding: '40px' }}>
          <div
            style={{
              border: '4px solid #f3f3f3',
              borderTop: '4px solid #667eea',
              borderRadius: '50%',
              width: '50px',
              height: '50px',
              animation: 'spin 1s linear infinite',
              margin: '0 auto 20px'
            }}
          />
          <p>Processing image...</p>
          <p style={{ fontSize: '0.9em', color: '#999' }}>
            Progress: {progress}%
          </p>
          <style>{`
            @keyframes spin {
              0% { transform: rotate(0deg); }
              100% { transform: rotate(360deg); }
            }
          `}</style>
        </div>
      )}

      {/* Error State */}
      {error && (
        <div
          style={{
            backgroundColor: '#fee',
            color: '#c33',
            padding: '15px',
            borderRadius: '6px'
          }}
        >
          ❌ {error}
        </div>
      )}

      {/* Results */}
      {result && (
        <div style={{ marginTop: '20px' }}>
          <h3 style={{ marginBottom: '15px' }}>✅ Processing Complete</h3>

          {/* Metrics Grid */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
              gap: '15px',
              marginBottom: '20px'
            }}
          >
            {[
              { label: 'Processing Time', value: `${result.metrics.processingTimeMs}ms` },
              { label: 'Original Size', value: `${result.metrics.originalSizeKB}KB` },
              { label: 'Optimized Size', value: `${result.metrics.optimizedSizeKB}KB` },
              { label: 'Size Reduction', value: `${result.metrics.reductionPercent}%` }
            ].map(metric => (
              <div
                key={metric.label}
                style={{
                  backgroundColor: '#667eea',
                  color: 'white',
                  padding: '15px',
                  borderRadius: '6px',
                  textAlign: 'center'
                }}
              >
                <div style={{ fontSize: '0.9em', marginBottom: '5px' }}>
                  {metric.label}
                </div>
                <div style={{ fontSize: '1.5em', fontWeight: 'bold' }}>
                  {metric.value}
                </div>
              </div>
            ))}
          </div>

          {/* Image Preview */}
          {result.lqip && (
            <div style={{ marginTop: '20px' }}>
              <p style={{ marginBottom: '10px', fontWeight: 'bold' }}>Preview:</p>
              <ProgressiveImage
                imageId={result.imageId}
                variants={result.variants}
                lqip={result.lqip}
                srcset={result.srcset}
                alt="Processed image"
              />
            </div>
          )}

          {/* Reset Button */}
          <button
            onClick={() => {
              setResult(null);
              setProgress(0);
            }}
            style={{
              marginTop: '20px',
              padding: '10px 20px',
              backgroundColor: '#667eea',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer'
            }}
          >
            Upload Another Image
          </button>
        </div>
      )}
    </div>
  );
};

export default ProgressiveImage;
