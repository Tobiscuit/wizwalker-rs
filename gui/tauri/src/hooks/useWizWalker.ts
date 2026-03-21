/**
 * useWizWalker — React hook wrapping all Tauri IPC commands.
 *
 * Per Tauri v2 docs (context7): `invoke<T>('command_name', { args })` from
 * `@tauri-apps/api/core` returns a typed Promise<T>.
 */

import { invoke } from '@tauri-apps/api/core';

// ── Type Definitions (mirror backend DTOs) ──────────────────────────────

export interface ClientInfo {
  label: string;
  pid: number;
  title: string;
  hooked: boolean;
  zone: string;
  isForeground: boolean;
  isRunning: boolean;
}

export interface Position {
  x: number;
  y: number;
  z: number;
}

export interface PlayerStats {
  maxHealth: number;
  maxMana: number;
  powerPipChance: number;
  accuracy: number;
  resist: number;
  damage: number;
  critical: number;
  pierce: number;
}

export interface CameraState {
  position: Position;
  yaw: number;
  pitch: number;
  roll: number;
  fov: number;
  distance: number;
}

export interface CombatStatus {
  inCombat: boolean;
  roundNumber: number;
  cardsCount: number;
}

export interface CardInfo {
  name: string;
  displayName: string;
  accuracy: number;
  isCastable: boolean;
  isEnchanted: boolean;
  isTreasureCard: boolean;
}

export interface CommandError {
  kind: string;
  message: string;
}

// ── Hook ────────────────────────────────────────────────────────────────

export function useWizWalker() {
  // ── Client Commands ─────────────────────────────────────────────────
  const scanClients = () =>
    invoke<ClientInfo[]>('scan_clients');

  const getClients = () =>
    invoke<ClientInfo[]>('get_clients');

  const openClient = (label: string) =>
    invoke<ClientInfo>('open_client', { label });

  const activateHooks = (label: string) =>
    invoke<void>('activate_hooks', { label });

  const closeClient = (label: string) =>
    invoke<void>('close_client', { label });

  // ── Hook Toggle Commands ────────────────────────────────────────────
  const getToggleStates = () =>
    invoke<Record<string, boolean>>('get_toggle_states');

  const toggleHook = (name: string, enabled: boolean) =>
    invoke<boolean>('toggle_hook', { name, enabled });

  const getSpeedMultiplier = () =>
    invoke<number>('get_speed_multiplier');

  const setSpeedMultiplier = (value: number) =>
    invoke<number>('set_speed_multiplier', { value });

  // ── Navigation Commands ─────────────────────────────────────────────
  const getPosition = () =>
    invoke<Position>('get_position');

  const teleportTo = (x: number, y: number, z: number) =>
    invoke<void>('teleport_to', { x, y, z });

  const xyzSync = () =>
    invoke<void>('xyz_sync');

  // ── Combat Commands ─────────────────────────────────────────────────
  const getCombatStatus = () =>
    invoke<CombatStatus>('get_combat_status');

  const getStats = () =>
    invoke<PlayerStats>('get_stats');

  const getCards = () =>
    invoke<CardInfo[]>('get_cards');

  // ── Camera Commands ─────────────────────────────────────────────────
  const getCamera = () =>
    invoke<CameraState>('get_camera');

  const setCameraPosition = (x: number, y: number, z: number) =>
    invoke<void>('set_camera_position', { x, y, z });

  const setCameraFov = (fov: number) =>
    invoke<void>('set_camera_fov', { fov });

  const setCameraRotation = (yaw: number, pitch: number, roll: number) =>
    invoke<void>('set_camera_rotation', { yaw, pitch, roll });

  return {
    // Clients
    scanClients,
    getClients,
    openClient,
    activateHooks,
    closeClient,
    // Hooks
    getToggleStates,
    toggleHook,
    getSpeedMultiplier,
    setSpeedMultiplier,
    // Navigation
    getPosition,
    teleportTo,
    xyzSync,
    // Combat
    getCombatStatus,
    getStats,
    getCards,
    // Camera
    getCamera,
    setCameraPosition,
    setCameraFov,
    setCameraRotation,
  };
}
