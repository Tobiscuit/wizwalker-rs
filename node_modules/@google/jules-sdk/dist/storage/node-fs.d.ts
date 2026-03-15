import { Activity, SessionResource } from '../types.js';
import { ActivityStorage, SessionStorage, CachedSession, SessionIndexEntry } from './types.js';
/**
 * Node.js filesystem implementation of ActivityStorage.
 * Stores activities in a JSONL file located at `.jules/cache/<sessionId>/activities.jsonl`.
 */
export declare class NodeFileStorage implements ActivityStorage {
    private filePath;
    private metadataPath;
    private initialized;
    private writeStream;
    private index;
    private indexBuilt;
    private indexBuildPromise;
    private currentFileSize;
    constructor(sessionId: string, rootDir: string);
    /**
     * Initializes the storage by ensuring the cache directory exists.
     *
     * **Side Effects:**
     * - Creates the `.jules/cache/<sessionId>` directory if it does not exist.
     * - Sets the internal `initialized` flag.
     */
    init(): Promise<void>;
    /**
     * Closes the storage.
     */
    close(): Promise<void>;
    private _readMetadata;
    private _writeMetadata;
    /**
     * Appends an activity to the file.
     *
     * **Side Effects:**
     * - Appends a new line containing the JSON representation of the activity to `activities.jsonl`.
     * - Implicitly calls `init()` if not already initialized.
     */
    append(activity: Activity): Promise<void>;
    /**
     * Builds the in-memory index by scanning the file once.
     * Handles concurrency by coalescing multiple calls into a single promise.
     */
    private buildIndex;
    /**
     * Retrieves an activity by ID.
     * Uses an in-memory index (ID -> Offset) to seek directly to the line.
     */
    get(activityId: string): Promise<Activity | undefined>;
    /**
     * Retrieves the latest activity.
     * Efficiently reads the file backwards to find the last valid entry.
     */
    latest(): Promise<Activity | undefined>;
    /**
     * Yields all activities in the file.
     *
     * **Behavior:**
     * - Opens a read stream to `activities.jsonl`.
     * - Reads line-by-line using `readline`.
     * - Parses each line as JSON.
     *
     * **Edge Cases:**
     * - Logs a warning and skips lines if JSON parsing fails (corrupt data).
     * - Returns immediately (yields nothing) if the file does not exist.
     */
    scan(): AsyncIterable<Activity>;
}
export declare class NodeSessionStorage implements SessionStorage {
    private cacheDir;
    private indexFilePath;
    private initialized;
    constructor(rootDir: string);
    init(): Promise<void>;
    private getSessionPath;
    upsert(session: SessionResource): Promise<void>;
    upsertMany(sessions: SessionResource[]): Promise<void>;
    get(sessionId: string): Promise<CachedSession | undefined>;
    delete(sessionId: string): Promise<void>;
    scanIndex(): AsyncIterable<SessionIndexEntry>;
}
