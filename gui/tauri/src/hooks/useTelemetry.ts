/**
 * useTelemetry — React hook for real-time game telemetry from the Rust backend.
 *
 * Listens to the `telemetry-update` event emitted by the background thread
 * in events.rs every 500ms. Per Tauri v2 docs (context7):
 * - `listen<T>(event, callback)` returns a Promise<UnlistenFn>
 * - Must call unlisten in useEffect cleanup to prevent memory leaks
 */

import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

export interface TelemetryData {
  activeClient: string | null;
  position: { x: number; y: number; z: number };
  zone: string;
  inCombat: boolean;
}

export function useTelemetry() {
  const [telemetry, setTelemetry] = useState<TelemetryData>({
    activeClient: null,
    position: { x: 0, y: 0, z: 0 },
    zone: '—',
    inCombat: false,
  });

  useEffect(() => {
    // Subscribe to telemetry events from the Rust backend.
    // listen() returns a Promise<UnlistenFn> per Tauri v2 docs.
    const unlistenPromise = listen<TelemetryData>(
      'telemetry-update',
      (event) => {
        setTelemetry(event.payload);
      }
    );

    // Cleanup: unsubscribe when component unmounts (SPA router support)
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return telemetry;
}
