import { useEffect } from 'react';
import { Upload, Download, Trash2, CheckCircle, XCircle, Clock } from 'lucide-react';
import { useAppStore } from '../store';
import type { TransferRecord, TransferStatus } from '../types';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleString();
}

const statusIcons: Record<TransferStatus, typeof CheckCircle> = {
  Completed: CheckCircle,
  Failed: XCircle,
  Cancelled: XCircle,
  Pending: Clock,
  InProgress: Clock,
};

const statusColors: Record<TransferStatus, string> = {
  Completed: 'text-green-500',
  Failed: 'text-red-500',
  Cancelled: 'text-gray-400',
  Pending: 'text-yellow-500',
  InProgress: 'text-primary-500',
};

export function TransfersPage() {
  const { transferHistory, loadHistory, clearHistory } = useAppStore();

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const getTotalSize = (record: TransferRecord): number => {
    return record.files.reduce((sum, file) => sum + file.size, 0);
  };

  return (
    <div className="p-6 space-y-6">
      <div className="card p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Transfer History
          </h2>
          {transferHistory.length > 0 && (
            <button
              onClick={clearHistory}
              className="btn btn-secondary text-sm flex items-center gap-2"
            >
              <Trash2 className="w-4 h-4" />
              Clear History
            </button>
          )}
        </div>

        {transferHistory.length === 0 ? (
          <p className="text-gray-500 dark:text-gray-400 text-sm">
            No transfer history yet. Send or receive files to see them here.
          </p>
        ) : (
          <div className="space-y-3">
            {transferHistory.map((record) => {
              const StatusIcon = statusIcons[record.status];
              const statusColor = statusColors[record.status];
              const totalSize = getTotalSize(record);

              return (
                <div
                  key={record.id}
                  className="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50 border border-gray-200 dark:border-gray-600"
                >
                  <div className="flex items-start gap-3">
                    {record.direction === 'Send' ? (
                      <Upload className="w-5 h-5 text-blue-500 mt-0.5" />
                    ) : (
                      <Download className="w-5 h-5 text-green-500 mt-0.5" />
                    )}

                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between mb-1">
                        <p className="font-medium text-gray-900 dark:text-white">
                          {record.peer_hostname || record.peer_address}
                        </p>
                        <div className="flex items-center gap-2">
                          <StatusIcon className={`w-4 h-4 ${statusColor}`} />
                          <span className={`text-sm ${statusColor}`}>
                            {record.status}
                          </span>
                        </div>
                      </div>

                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {formatDate(record.timestamp)} -{' '}
                        {record.files.length} file{record.files.length !== 1 ? 's' : ''} -{' '}
                        {formatBytes(totalSize)}
                      </p>

                      {record.error && (
                        <p className="text-sm text-red-500 mt-1">{record.error}</p>
                      )}

                      <div className="mt-2 text-sm text-gray-600 dark:text-gray-300">
                        {record.files.slice(0, 2).map((file, i) => (
                          <div key={i} className="truncate">
                            {file.name}
                          </div>
                        ))}
                        {record.files.length > 2 && (
                          <div className="text-gray-400">
                            ...and {record.files.length - 2} more
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
