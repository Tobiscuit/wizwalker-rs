import { Platform, PlatformResponse } from './types.js';
/**
 * Node.js implementation of the Platform interface.
 */
export declare class NodePlatform implements Platform {
    /**
     * Saves a file to the local filesystem using `node:fs/promises`.
     *
     * **Side Effects:**
     * - Writes a file to disk.
     * - Overwrites the file if it already exists.
     */
    saveFile(filepath: string, data: string, encoding: 'base64', activityId?: string): Promise<void>;
    sleep(ms: number): Promise<void>;
    createDataUrl(data: string, mimeType: string): string;
    fetch(input: string, init?: any): Promise<PlatformResponse>;
    crypto: {
        randomUUID: () => `${string}-${string}-${string}-${string}-${string}`;
        sign(text: string, secret: string): Promise<string>;
        verify(text: string, signature: string, secret: string): Promise<boolean>;
    };
    encoding: {
        base64Encode: (text: string) => string;
        base64Decode: (text: string) => string;
    };
    getEnv(key: string): string | undefined;
    readFile(path: string): Promise<string>;
    writeFile(path: string, content: string): Promise<void>;
    deleteFile(path: string): Promise<void>;
}
