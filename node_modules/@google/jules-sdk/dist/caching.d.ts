import { CachedSession } from './storage/types.js';
export type CacheTier = 'hot' | 'warm' | 'frozen';
/**
 * Determines the cache tier for a session based on its state and age.
 *
 * Strategy:
 * - **Frozen (Tier 3):** > 30 days old. Immutable.
 * - **Warm (Tier 2):** Terminal state + Verified < 24h ago. High read performance.
 * - **Hot (Tier 1):** Active or Stale. Requires network sync.
 */
export declare function determineCacheTier(cached: CachedSession, now?: number): CacheTier;
/**
 * Helper to check if a cached session is valid to return immediately.
 * Returns true if the session is Frozen or Warm.
 */
export declare function isCacheValid(cached: CachedSession | undefined, now?: number): cached is CachedSession;
