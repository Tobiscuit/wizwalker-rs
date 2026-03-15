import { ApiClient } from './api.js';
import { SourceManager } from './types.js';
/**
 * Creates a SourceManager instance.
 * The SourceManager is a callable object (an async iterator) with a `get` method attached.
 * @internal
 */
export declare function createSourceManager(apiClient: ApiClient): SourceManager;
