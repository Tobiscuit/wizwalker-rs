import { Activity, SessionResource } from '../types.js';
import { ActivityStorage, SessionStorage, CachedSession, SessionIndexEntry } from './types.js';
/**
 * In-memory implementation of ActivityStorage.
 * Useful for testing or environments where persistence is not required.
 */
export declare class MemoryStorage implements ActivityStorage {
    private activities;
    private indices;
    /**
     * Initializes the storage. No-op for memory storage.
     */
    init(): Promise<void>;
    /**
     * Closes the storage and clears memory.
     */
    close(): Promise<void>;
    /**
     * Appends an activity to the in-memory list.
     *
     * **Guarantee:**
     * - Idempotent: If an activity with the same ID exists, it updates it in place.
     * - Append-only: New activities are always added to the end.
     *
     * **Side Effects:**
     * - Modifies the internal `activities` array.
     */
    append(activity: Activity): Promise<void>;
    /**
     * Retrieves an activity by ID.
     */
    get(activityId: string): Promise<Activity | undefined>;
    /**
     * Retrieves the latest activity.
     */
    latest(): Promise<Activity | undefined>;
    /**
     * Yields all activities in chronological order.
     */
    scan(): AsyncIterable<Activity>;
}
/**
 * In-memory implementation of SessionStorage.
 */
export declare class MemorySessionStorage implements SessionStorage {
    private sessions;
    private index;
    init(): Promise<void>;
    upsert(session: SessionResource): Promise<void>;
    upsertMany(sessions: SessionResource[]): Promise<void>;
    get(sessionId: string): Promise<CachedSession | undefined>;
    delete(sessionId: string): Promise<void>;
    scanIndex(): AsyncIterable<SessionIndexEntry>;
}
