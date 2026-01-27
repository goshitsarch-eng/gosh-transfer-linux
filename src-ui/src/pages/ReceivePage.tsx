import { useEffect } from 'react';
import {
  Wifi,
  Network,
  Shield,
  Server,
  HelpCircle,
  Check,
  X,
  Loader2,
} from 'lucide-react';
import { useAppStore } from '../store';
import {
  getInterfaceCategory,
  shouldShowInterface,
  type InterfaceCategory,
} from '../types';

const categoryIcons: Record<InterfaceCategory, typeof Wifi> = {
  WiFi: Wifi,
  Ethernet: Network,
  Vpn: Shield,
  Docker: Server,
  Other: HelpCircle,
};

const categoryLabels: Record<InterfaceCategory, string> = {
  WiFi: 'WiFi',
  Ethernet: 'Ethernet',
  Vpn: 'VPN',
  Docker: 'Docker',
  Other: 'Other',
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function formatSpeed(bps: number): string {
  return `${formatBytes(bps)}/s`;
}

export function ReceivePage() {
  const {
    settings,
    interfaces,
    pendingTransfers,
    activeTransfers,
    loadInterfaces,
    acceptTransfer,
    rejectTransfer,
    acceptAll,
    rejectAll,
    cancelTransfer,
  } = useAppStore();

  useEffect(() => {
    loadInterfaces();
    const interval = setInterval(loadInterfaces, 5000);
    return () => clearInterval(interval);
  }, [loadInterfaces]);

  const filteredInterfaces = interfaces.filter((iface) => {
    if (iface.is_loopback) return false;
    if (!settings?.interfaceFilters) return true;
    return shouldShowInterface(iface.name, settings.interfaceFilters);
  });

  return (
    <div className="p-6 space-y-6">
      {/* Network Interfaces */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Your Addresses
        </h2>

        {filteredInterfaces.length === 0 ? (
          <p className="text-gray-500 dark:text-gray-400 text-sm">
            No network interfaces found. Check your network connection.
          </p>
        ) : (
          <div className="space-y-2">
            {filteredInterfaces.map((iface) => {
              const category = getInterfaceCategory(iface.name);
              const Icon = categoryIcons[category];
              const label = categoryLabels[category];

              return (
                <div
                  key={iface.name}
                  className="flex items-center justify-between p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50"
                >
                  <div className="flex items-center gap-3">
                    <Icon className="w-5 h-5 text-gray-500 dark:text-gray-400" />
                    <div>
                      <p className="font-medium text-gray-900 dark:text-white">
                        {iface.ip}
                      </p>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {label} ({iface.name})
                      </p>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        <p className="text-xs text-gray-500 dark:text-gray-400 mt-4">
          Share one of these addresses with peers to receive files. Port:{' '}
          <span className="font-mono">{settings?.port || 53317}</span>
        </p>
      </div>

      {/* Pending Transfers */}
      <div className="card p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Pending Requests
          </h2>
          {pendingTransfers.length > 1 && (
            <div className="flex gap-2">
              <button onClick={acceptAll} className="btn btn-primary text-sm">
                Accept All
              </button>
              <button onClick={rejectAll} className="btn btn-secondary text-sm">
                Reject All
              </button>
            </div>
          )}
        </div>

        {pendingTransfers.length === 0 ? (
          <p className="text-gray-500 dark:text-gray-400 text-sm">
            No pending transfer requests. Share your address with peers to receive files.
          </p>
        ) : (
          <div className="space-y-3">
            {pendingTransfers.map((transfer) => (
              <div
                key={transfer.id}
                className="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50 border border-gray-200 dark:border-gray-600"
              >
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <p className="font-medium text-gray-900 dark:text-white">
                      {transfer.peer_hostname || transfer.peer_address}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {transfer.files.length} file{transfer.files.length !== 1 ? 's' : ''} -{' '}
                      {formatBytes(transfer.total_size)}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => acceptTransfer(transfer.id)}
                      className="p-2 rounded-lg bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400"
                    >
                      <Check className="w-5 h-5" />
                    </button>
                    <button
                      onClick={() => rejectTransfer(transfer.id)}
                      className="p-2 rounded-lg bg-red-100 text-red-700 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400"
                    >
                      <X className="w-5 h-5" />
                    </button>
                  </div>
                </div>

                <div className="text-sm text-gray-600 dark:text-gray-300">
                  {transfer.files.slice(0, 3).map((file, i) => (
                    <div key={i} className="truncate">
                      {file.name} ({formatBytes(file.size)})
                    </div>
                  ))}
                  {transfer.files.length > 3 && (
                    <div className="text-gray-400">
                      ...and {transfer.files.length - 3} more
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Active Transfers */}
      {activeTransfers.size > 0 && (
        <div className="card p-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Active Transfers
          </h2>

          <div className="space-y-3">
            {Array.from(activeTransfers.values()).map((progress) => {
              const percent = progress.total_bytes > 0
                ? Math.round((progress.bytes_transferred / progress.total_bytes) * 100)
                : 0;

              return (
                <div
                  key={progress.transfer_id}
                  className="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50"
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
                      <span className="font-medium text-gray-900 dark:text-white truncate">
                        {progress.current_file}
                      </span>
                    </div>
                    <button
                      onClick={() => cancelTransfer(progress.transfer_id)}
                      className="p-1 text-gray-400 hover:text-red-500 transition-colors"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>

                  <div className="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2 mb-2">
                    <div
                      className="bg-primary-500 h-2 rounded-full transition-all duration-300"
                      style={{ width: `${percent}%` }}
                    />
                  </div>

                  <div className="flex justify-between text-sm text-gray-500 dark:text-gray-400">
                    <span>
                      File {progress.current_file_index + 1} of {progress.total_files}
                    </span>
                    <span>
                      {formatBytes(progress.bytes_transferred)} / {formatBytes(progress.total_bytes)} -{' '}
                      {formatSpeed(progress.speed_bps)}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
