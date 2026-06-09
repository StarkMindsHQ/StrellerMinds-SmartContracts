/**
 * CSV Export Utility
 * 
 * Implements RFC 4180 compliant CSV generation with:
 * - No field truncation (handles fields of any length)
 * - Proper escaping for commas, quotes, newlines
 * - UTF-8 BOM for Excel compatibility
 * - Streaming support for large datasets
 */

/**
 * Escape a field value for CSV according to RFC 4180
 * - Fields containing commas, quotes, or newlines are enclosed in double quotes
 * - Double quotes within fields are escaped by doubling them
 */
export function escapeCSVField(value: any): string {
  if (value === null || value === undefined) {
    return '';
  }

  const stringValue = String(value);
  
  // Check if field needs quoting (contains comma, double quote, or newline)
  const needsQuoting = /[",\n\r]/.test(stringValue);
  
  if (needsQuoting) {
    // Escape double quotes by doubling them
    const escaped = stringValue.replace(/"/g, '""');
    return `"${escaped}"`;
  }
  
  return stringValue;
}

/**
 * Convert array of objects to CSV format
 * 
 * @param data Array of objects to convert
 * @param headers Optional array of header names (uses object keys if not provided)
 * @param includeBOM Whether to include UTF-8 BOM for Excel compatibility
 * @returns CSV string
 */
export function convertToCSV(
  data: Record<string, any>[],
  headers?: string[],
  includeBOM: boolean = true
): string {
  if (!data || data.length === 0) {
    return includeBOM ? '\uFEFF' : '';
  }

  // Determine headers from first object if not provided
  const resolvedHeaders = headers || Object.keys(data[0]);

  // Build CSV
  const lines: string[] = [];

  // Add BOM if requested (for Excel UTF-8 compatibility)
  if (includeBOM) {
    lines.push('\uFEFF');
  }

  // Add header row
  lines.push(resolvedHeaders.map(escapeCSVField).join(','));

  // Add data rows
  for (const row of data) {
    const values = resolvedHeaders.map(header => {
      const value = row[header];
      return escapeCSVField(value);
    });
    lines.push(values.join(','));
  }

  // Join with CRLF (RFC 4180 standard)
  return lines.join('\r\n');
}

/**
 * Convert array of objects to CSV as a stream (for large datasets)
 * 
 * @param data Iterator/generator of objects
 * @param headers Array of header names
 * @param includeBOM Whether to include UTF-8 BOM
 * @returns Readable stream of CSV data
 */
export function convertToCSVStream(
  data: AsyncIterable<Record<string, any>> | Iterable<Record<string, any>>,
  headers: string[],
  includeBOM: boolean = true
): AsyncGenerator<string> {
  return (async function* () {
    // Add BOM if requested
    if (includeBOM) {
      yield '\uFEFF';
    }

    // Add header row
    yield headers.map(escapeCSVField).join(',') + '\r\n';

    // Add data rows
    for await (const row of data) {
      const values = headers.map(header => {
        const value = row[header];
        return escapeCSVField(value);
      });
      yield values.join(',') + '\r\n';
    }
  })();
}

/**
 * Generate CSV filename with timestamp
 */
export function generateCSVFilename(prefix: string, extension: string = 'csv'): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
  return `${prefix}_${timestamp}.${extension}`;
}

/**
 * Validate that data can be exported (check size limits)
 */
export function validateExportData(
  data: Record<string, any>[],
  maxRecords: number
): { valid: boolean; error?: string } {
  if (!data || data.length === 0) {
    return { valid: false, error: 'No data to export' };
  }

  if (data.length > maxRecords) {
    return {
      valid: false,
      error: `Data exceeds maximum record limit (${maxRecords}). Please use filters to reduce the dataset size.`,
    };
  }

  return { valid: true };
}

/**
 * Get estimated CSV size in bytes (rough estimate)
 */
export function estimateCSVSize(data: Record<string, any>[], headers: string[]): number {
  if (!data || data.length === 0) {
    return 0;
  }

  // Estimate average row size from first 10 rows
  const sampleSize = Math.min(10, data.length);
  let totalSize = 0;

  for (let i = 0; i < sampleSize; i++) {
    const row = data[i];
    const rowSize = headers.reduce((size, header) => {
      return size + String(row[header] || '').length + 1; // +1 for comma
    }, 0);
    totalSize += rowSize;
  }

  const avgRowSize = totalSize / sampleSize;
  const estimatedTotal = avgRowSize * data.length;

  return Math.round(estimatedTotal);
}

/**
 * Chunk data for streaming export
 */
export function* chunkData<T>(data: T[], chunkSize: number): Generator<T[]> {
  for (let i = 0; i < data.length; i += chunkSize) {
    yield data.slice(i, i + chunkSize);
  }
}

/**
 * Format bytes to human-readable format
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}
