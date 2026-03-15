import { JulesClient, JulesQuery, JulesDomain, QueryResult } from '../types.js';
/**
 * Standalone query engine function.
 * Handles planning, index scanning, and hydration.
 */
export declare function select<T extends JulesDomain>(client: JulesClient, query: JulesQuery<T>): Promise<QueryResult<T>[]>;
