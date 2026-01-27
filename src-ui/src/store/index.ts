import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  AppSettings,
  NetworkInterface,
  Favorite,
  PendingTransfer,
  TransferProgress,
  TransferRecord,
  EngineEvent,
} from '../types';

interface AppState {
  // Server state
  serverRunning: boolean;
  serverPort: number | null;

  // Network
  interfaces: NetworkInterface[];

  // Transfers
  pendingTransfers: PendingTransfer[];
  activeTransfers: Map<string, TransferProgress>;
  transferHistory: TransferRecord[];

  // Favorites
  favorites: Favorite[];

  // Settings
  settings: AppSettings | null;

  // UI state
  currentPage: 'send' | 'receive' | 'transfers' | 'settings' | 'about';

  // Actions
  setCurrentPage: (page: AppState['currentPage']) => void;
  loadSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  loadFavorites: () => Promise<void>;
  addFavorite: (name: string, address: string) => Promise<Favorite>;
  updateFavorite: (id: string, name?: string, address?: string) => Promise<void>;
  deleteFavorite: (id: string) => Promise<void>;
  touchFavorite: (id: string) => Promise<void>;
  loadHistory: () => Promise<void>;
  clearHistory: () => Promise<void>;
  loadInterfaces: () => Promise<void>;
  loadPendingTransfers: () => Promise<void>;
  acceptTransfer: (id: string) => Promise<void>;
  rejectTransfer: (id: string) => Promise<void>;
  acceptAll: () => Promise<void>;
  rejectAll: () => Promise<void>;
  cancelTransfer: (id: string) => Promise<void>;
  sendFiles: (address: string, port: number, paths: string[]) => Promise<void>;
  sendDirectory: (address: string, port: number, path: string) => Promise<void>;
  resolveAddress: (address: string) => Promise<{ ip: string | null; error: string | null }>;
  checkPeer: (address: string, port: number) => Promise<boolean>;
  initializeEventListener: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  // Initial state
  serverRunning: false,
  serverPort: null,
  interfaces: [],
  pendingTransfers: [],
  activeTransfers: new Map(),
  transferHistory: [],
  favorites: [],
  settings: null,
  currentPage: 'send',

  setCurrentPage: (page) => set({ currentPage: page }),

  loadSettings: async () => {
    const settings = await invoke<AppSettings>('get_settings');
    set({ settings });
  },

  saveSettings: async (settings) => {
    await invoke('save_settings', { settings });
    set({ settings });
  },

  loadFavorites: async () => {
    const favorites = await invoke<Favorite[]>('list_favorites');
    set({ favorites });
  },

  addFavorite: async (name, address) => {
    const favorite = await invoke<Favorite>('add_favorite', { name, address });
    await get().loadFavorites();
    return favorite;
  },

  updateFavorite: async (id, name, address) => {
    await invoke('update_favorite', { id, name, address });
    await get().loadFavorites();
  },

  deleteFavorite: async (id) => {
    await invoke('delete_favorite', { id });
    await get().loadFavorites();
  },

  touchFavorite: async (id) => {
    await invoke('touch_favorite', { id });
    await get().loadFavorites();
  },

  loadHistory: async () => {
    const transferHistory = await invoke<TransferRecord[]>('list_history');
    set({ transferHistory });
  },

  clearHistory: async () => {
    await invoke('clear_history');
    set({ transferHistory: [] });
  },

  loadInterfaces: async () => {
    const interfaces = await invoke<NetworkInterface[]>('get_interfaces');
    set({ interfaces });
  },

  loadPendingTransfers: async () => {
    const pendingTransfers = await invoke<PendingTransfer[]>('get_pending_transfers');
    set({ pendingTransfers });
  },

  acceptTransfer: async (id) => {
    await invoke('accept_transfer', { transferId: id });
  },

  rejectTransfer: async (id) => {
    await invoke('reject_transfer', { transferId: id });
    set((state) => ({
      pendingTransfers: state.pendingTransfers.filter((t) => t.id !== id),
    }));
  },

  acceptAll: async () => {
    await invoke('accept_all');
  },

  rejectAll: async () => {
    await invoke('reject_all');
    set({ pendingTransfers: [] });
  },

  cancelTransfer: async (id) => {
    await invoke('cancel_transfer', { transferId: id });
    set((state) => {
      const activeTransfers = new Map(state.activeTransfers);
      activeTransfers.delete(id);
      return { activeTransfers };
    });
  },

  sendFiles: async (address, port, paths) => {
    await invoke('send_files', { address, port, paths });
  },

  sendDirectory: async (address, port, path) => {
    await invoke('send_directory', { address, port, path });
  },

  resolveAddress: async (address) => {
    const result = await invoke<{ ip: string | null; error: string | null }>(
      'resolve_address',
      { address }
    );
    return result;
  },

  checkPeer: async (address, port) => {
    return invoke<boolean>('check_peer', { address, port });
  },

  initializeEventListener: async () => {
    await listen<EngineEvent>('engine-event', (event) => {
      const engineEvent = event.payload;

      switch (engineEvent.type) {
        case 'ServerStarted':
          set({ serverRunning: true, serverPort: engineEvent.port });
          break;

        case 'ServerStopped':
          set({ serverRunning: false, serverPort: null });
          break;

        case 'PortChanged':
          set({ serverPort: engineEvent.newPort });
          break;

        case 'TransferRequest':
          set((state) => ({
            pendingTransfers: [...state.pendingTransfers, engineEvent.transfer],
          }));
          break;

        case 'TransferProgress':
          set((state) => {
            const activeTransfers = new Map(state.activeTransfers);
            activeTransfers.set(engineEvent.progress.transfer_id, engineEvent.progress);
            // Remove from pending if it was there
            const pendingTransfers = state.pendingTransfers.filter(
              (t) => t.id !== engineEvent.progress.transfer_id
            );
            return { activeTransfers, pendingTransfers };
          });
          break;

        case 'TransferComplete':
          set((state) => {
            const activeTransfers = new Map(state.activeTransfers);
            activeTransfers.delete(engineEvent.transferId);
            return { activeTransfers };
          });
          // Refresh history
          get().loadHistory();
          break;

        case 'TransferFailed':
          set((state) => {
            const activeTransfers = new Map(state.activeTransfers);
            activeTransfers.delete(engineEvent.transferId);
            const pendingTransfers = state.pendingTransfers.filter(
              (t) => t.id !== engineEvent.transferId
            );
            return { activeTransfers, pendingTransfers };
          });
          // Refresh history
          get().loadHistory();
          break;

        case 'TransferRetry':
          // Could show a notification or update UI to show retry status
          console.log(
            `Transfer ${engineEvent.transferId} retry ${engineEvent.attempt}/${engineEvent.maxAttempts}: ${engineEvent.error}`
          );
          break;
      }
    });

    // Initialize by loading initial data
    await Promise.all([
      get().loadSettings(),
      get().loadFavorites(),
      get().loadHistory(),
      get().loadInterfaces(),
      get().loadPendingTransfers(),
    ]);

    // Server auto-starts in backend, but check status
    await invoke('initialize');
  },
}));
