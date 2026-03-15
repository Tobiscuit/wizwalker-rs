import { Activity } from '../types.js';
/**
 * Represents the cache information for a single session.
 */
export type SessionCacheInfo = {
    sessionId: string;
    activityCount: number;
    lastSyncedAt: Date;
};
/**
 * Represents global cache information.
 */
export type GlobalCacheInfo = {
    lastSyncedAt: Date;
    sessionCount: number;
};
/**
 * Retrieves cache information for a specific session.
 *
 * @param sessionId - The ID of the session.
 * @returns A promise that resolves with the session's cache information, or null if not found.
 */
export declare function getSessionCacheInfo(sessionId: string, rootDirOverride?: string): Promise<SessionCacheInfo | null>;
export declare function updateGlobalCacheMetadata(rootDirOverride?: string): Promise<void>;
export declare function getCacheInfo(rootDirOverride?: string): Promise<GlobalCacheInfo>;
export declare function getSessionCount(rootDirOverride?: string): Promise<number>;
export declare function getActivityCount(sessionId: string, rootDirOverride?: string): Promise<number>;
export declare function getLatestActivities(sessionId: string, n: number, rootDirOverride?: string): Promise<Activity[]>;
