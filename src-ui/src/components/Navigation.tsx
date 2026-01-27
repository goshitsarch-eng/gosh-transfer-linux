import {
  Send,
  Download,
  History,
  Settings,
  Info,
  Circle,
} from 'lucide-react';
import { useAppStore } from '../store';

const navItems = [
  { id: 'send' as const, label: 'Send', icon: Send },
  { id: 'receive' as const, label: 'Receive', icon: Download },
  { id: 'transfers' as const, label: 'History', icon: History },
  { id: 'settings' as const, label: 'Settings', icon: Settings },
  { id: 'about' as const, label: 'About', icon: Info },
];

export function Navigation() {
  const { currentPage, setCurrentPage, serverRunning, serverPort, pendingTransfers } =
    useAppStore();

  return (
    <nav className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <div className="px-4">
        <div className="flex items-center justify-between h-14">
          <div className="flex items-center gap-1">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = currentPage === item.id;
              const hasBadge = item.id === 'receive' && pendingTransfers.length > 0;

              return (
                <button
                  key={item.id}
                  onClick={() => setCurrentPage(item.id)}
                  className={`relative flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                    isActive
                      ? 'bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-400'
                      : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
                  }`}
                >
                  <Icon className="w-4 h-4" />
                  <span>{item.label}</span>
                  {hasBadge && (
                    <span className="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
                      {pendingTransfers.length}
                    </span>
                  )}
                </button>
              );
            })}
          </div>

          <div className="flex items-center gap-2 text-sm">
            <Circle
              className={`w-3 h-3 ${
                serverRunning ? 'text-green-500 fill-green-500' : 'text-gray-400'
              }`}
            />
            <span className="text-gray-600 dark:text-gray-300">
              {serverRunning ? `Port ${serverPort}` : 'Offline'}
            </span>
          </div>
        </div>
      </div>
    </nav>
  );
}
