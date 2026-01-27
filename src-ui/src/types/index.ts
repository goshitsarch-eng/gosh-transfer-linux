// Type definitions matching Rust backend

export interface AppSettings {
  port: number;
  deviceName: string;
  downloadDir: string;
  trustedHosts: string[];
  receiveOnly: boolean;
  notificationsEnabled: boolean;
  theme: 'dark' | 'light' | 'system';
  maxRetries: number;
  retryDelayMs: number;
  bandwidthLimitBps: number | null;
  interfaceFilters: InterfaceFilters;
}

export interface InterfaceFilters {
  showWifi: boolean;
  showEthernet: boolean;
  showVpn: boolean;
  showDocker: boolean;
  showOther: boolean;
}

export interface NetworkInterface {
  name: string;
  ip: string;
  is_loopback: boolean;
}

export interface Favorite {
  id: string;
  name: string;
  address: string;
  last_resolved_ip: string | null;
  last_used: string | null;
}

export interface TransferFile {
  name: string;
  size: number;
  is_directory: boolean;
}

export interface PendingTransfer {
  id: string;
  peer_address: string;
  peer_hostname: string;
  files: TransferFile[];
  total_size: number;
  created_at: string;
}

export interface TransferProgress {
  transfer_id: string;
  current_file: string;
  current_file_index: number;
  total_files: number;
  bytes_transferred: number;
  total_bytes: number;
  speed_bps: number;
}

export interface TransferRecord {
  id: string;
  direction: 'Send' | 'Receive';
  peer_address: string;
  peer_hostname: string;
  timestamp: string;
  files: TransferFile[];
  status: TransferStatus;
  error: string | null;
}

export type TransferStatus = 'Pending' | 'InProgress' | 'Completed' | 'Failed' | 'Cancelled';

export interface ResolveResult {
  hostname: string;
  ip: string | null;
  error: string | null;
}

export interface PeerInfo {
  device_name: string;
  version: string;
}

// Engine events from backend
export type EngineEvent =
  | { type: 'TransferRequest'; transfer: PendingTransfer }
  | { type: 'TransferProgress'; progress: TransferProgress }
  | { type: 'TransferComplete'; transferId: string }
  | { type: 'TransferFailed'; transferId: string; error: string }
  | { type: 'TransferRetry'; transferId: string; attempt: number; maxAttempts: number; error: string }
  | { type: 'ServerStarted'; port: number }
  | { type: 'ServerStopped' }
  | { type: 'PortChanged'; oldPort: number; newPort: number };

// Interface category for filtering
export type InterfaceCategory = 'WiFi' | 'Ethernet' | 'Vpn' | 'Docker' | 'Other';

export function getInterfaceCategory(name: string): InterfaceCategory {
  if (name.startsWith('tailscale') || name.startsWith('tun')) return 'Vpn';
  if (name.startsWith('wl')) return 'WiFi';
  if (name.startsWith('en') || name.startsWith('eth')) return 'Ethernet';
  if (name.startsWith('docker') || name.startsWith('br-')) return 'Docker';
  return 'Other';
}

export function shouldShowInterface(
  name: string,
  filters: InterfaceFilters
): boolean {
  const category = getInterfaceCategory(name);
  switch (category) {
    case 'WiFi': return filters.showWifi;
    case 'Ethernet': return filters.showEthernet;
    case 'Vpn': return filters.showVpn;
    case 'Docker': return filters.showDocker;
    case 'Other': return filters.showOther;
  }
}
