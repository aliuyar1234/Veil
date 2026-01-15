/**
 * Veil WASM Module - TypeScript Type Definitions
 *
 * @packageDocumentation
 * @module @veil/wasm
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * A single PII detection finding.
 */
export interface Finding {
  /** PII category (e.g., "email", "phone", "iban") */
  category: string;

  /** The matched text value */
  value: string;

  /** Byte offset of match start in source */
  start: number;

  /** Byte offset of match end in source (exclusive) */
  end: number;

  /** Confidence score (0.0 - 1.0) */
  confidence: number;
}

/**
 * Statistics from a scan operation.
 */
export interface ScanStats {
  /** Total bytes processed */
  bytesProcessed: number;

  /** Processing time in milliseconds */
  durationMs: number;

  /** Number of findings by category */
  categoryCounts: Record<string, number>;
}

/**
 * Result of a scan operation.
 */
export interface ScanResult {
  /** All findings from the scan */
  findings: Finding[];

  /** Processing statistics */
  stats: ScanStats;
}

/**
 * Statistics from a protect operation.
 */
export interface ProtectStats {
  /** Number of replacements made */
  replacements: number;

  /** Categories that were protected */
  protectedCategories: string[];

  /** Processing time in milliseconds */
  durationMs: number;
}

/**
 * Result of a protect operation.
 */
export interface ProtectResult {
  /** Protected content as bytes */
  data: Uint8Array;

  /** Statistics about the protection */
  stats: ProtectStats;
}

// ============================================================================
// Options Types
// ============================================================================

/**
 * Options for configuring a scan operation.
 */
export interface ScanOptions {
  /**
   * Original filename, used for format detection.
   * @example "document.txt"
   */
  filename?: string;

  /**
   * Categories to detect. Empty array detects all categories.
   * @example ["email", "phone"]
   */
  categories?: string[];

  /**
   * Minimum confidence threshold (0.0 - 1.0).
   * @default 0.5
   */
  minConfidence?: number;

  /**
   * Progress callback, called with percentage (0-100).
   */
  onProgress?: ProgressCallback;
}

/**
 * Protection style for redacting PII.
 */
export type ProtectStyle = 'labels' | 'redact' | 'mask';

/**
 * Options for configuring a protect operation.
 */
export interface ProtectOptions {
  /**
   * Original filename, used for format detection.
   */
  filename?: string;

  /**
   * Protection style.
   * - "labels": Replace with [CATEGORY] labels (e.g., [EMAIL])
   * - "redact": Replace with ████ characters
   * - "mask": Replace with partial masking (e.g., j***@***.com)
   * @default "labels"
   */
  style?: ProtectStyle;

  /**
   * Categories to protect. Empty array protects all found categories.
   */
  categories?: string[];

  /**
   * Progress callback, called with percentage (0-100).
   */
  onProgress?: ProgressCallback;
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * Error codes returned by WASM operations.
 */
export type ErrorCode =
  | 'InvalidInput'
  | 'UnsupportedFormat'
  | 'FileTooLarge'
  | 'InvalidConfig'
  | 'InternalError';

/**
 * Error object thrown by WASM operations.
 */
export interface VeilError extends Error {
  /** Error code for programmatic handling */
  code: ErrorCode;
}

// ============================================================================
// Callback Types
// ============================================================================

/**
 * Progress callback function.
 * @param progress - Percentage complete (0-100)
 */
export type ProgressCallback = (progress: number) => void;

// ============================================================================
// Module API
// ============================================================================

/**
 * Initialize the WASM module.
 *
 * Must be called before any other function.
 * Safe to call multiple times; subsequent calls are no-ops.
 *
 * @returns Promise that resolves when module is ready
 * @throws {VeilError} If module fails to initialize
 *
 * @example
 * ```typescript
 * import { init, scan } from '@veil/wasm';
 *
 * await init();
 * const result = await scan(buffer);
 * ```
 */
export function init(): Promise<void>;

/**
 * Scan a file for PII.
 *
 * @param data - File content as ArrayBuffer or Uint8Array
 * @param options - Scan configuration
 * @returns Promise resolving to scan results
 * @throws {VeilError} On invalid input or processing error
 *
 * @example
 * ```typescript
 * const buffer = await file.arrayBuffer();
 * const result = await scan(buffer, {
 *   filename: 'document.txt',
 *   onProgress: (p) => console.log(`${p}%`)
 * });
 * console.log(`Found ${result.findings.length} items`);
 * ```
 */
export function scan(
  data: ArrayBuffer | Uint8Array,
  options?: ScanOptions
): Promise<ScanResult>;

/**
 * Protect a file by redacting PII.
 *
 * @param data - File content as ArrayBuffer or Uint8Array
 * @param options - Protection configuration
 * @returns Promise resolving to protected file and stats
 * @throws {VeilError} On invalid input or processing error
 *
 * @example
 * ```typescript
 * const buffer = await file.arrayBuffer();
 * const result = await protect(buffer, {
 *   style: 'labels',
 *   onProgress: (p) => updateProgressBar(p)
 * });
 *
 * // Download protected file
 * const blob = new Blob([result.data], { type: 'text/plain' });
 * const url = URL.createObjectURL(blob);
 * ```
 */
export function protect(
  data: ArrayBuffer | Uint8Array,
  options?: ProtectOptions
): Promise<ProtectResult>;

/**
 * Get supported PII categories.
 *
 * @returns Array of category identifiers
 *
 * @example
 * ```typescript
 * const categories = getCategories();
 * // ['email', 'phone', 'iban', 'credit_card', ...]
 * ```
 */
export function getCategories(): string[];

/**
 * Get module version.
 *
 * @returns Version string (semver)
 *
 * @example
 * ```typescript
 * console.log(`Veil WASM v${getVersion()}`);
 * // "Veil WASM v0.1.0"
 * ```
 */
export function getVersion(): string;

/**
 * Check if a file type is supported.
 *
 * @param filename - Filename with extension
 * @returns True if the file type can be processed
 *
 * @example
 * ```typescript
 * if (isSupported('document.pdf')) {
 *   // PDF processing available
 * }
 * ```
 */
export function isSupported(filename: string): boolean;
