import { useState, useEffect } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, Save, Plus, X, Loader2 } from 'lucide-react';
import { useAppStore } from '../store';
import type { AppSettings } from '../types';

export function SettingsPage() {
  const { settings, saveSettings } = useAppStore();
  const [localSettings, setLocalSettings] = useState<AppSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [newTrustedHost, setNewTrustedHost] = useState('');

  useEffect(() => {
    if (settings) {
      setLocalSettings({ ...settings });
    }
  }, [settings]);

  if (!localSettings) {
    return (
      <div className="p-6 flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    );
  }

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveSettings(localSettings);
    } finally {
      setSaving(false);
    }
  };

  const handleSelectDownloadDir = async () => {
    const folder = await open({
      multiple: false,
      directory: true,
    });
    if (folder) {
      setLocalSettings({ ...localSettings, downloadDir: folder });
    }
  };

  const handleAddTrustedHost = () => {
    if (!newTrustedHost.trim()) return;
    setLocalSettings({
      ...localSettings,
      trustedHosts: [...localSettings.trustedHosts, newTrustedHost.trim()],
    });
    setNewTrustedHost('');
  };

  const handleRemoveTrustedHost = (index: number) => {
    setLocalSettings({
      ...localSettings,
      trustedHosts: localSettings.trustedHosts.filter((_, i) => i !== index),
    });
  };

  const hasChanges = JSON.stringify(settings) !== JSON.stringify(localSettings);

  return (
    <div className="p-6 space-y-6">
      {/* Network Settings */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Network
        </h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Server Port
            </label>
            <input
              type="number"
              value={localSettings.port}
              onChange={(e) =>
                setLocalSettings({ ...localSettings, port: Number(e.target.value) })
              }
              className="input w-32"
              min={1}
              max={65535}
            />
            <p className="text-xs text-gray-500 mt-1">
              Changing the port requires a restart.
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Device Name
            </label>
            <input
              type="text"
              value={localSettings.deviceName}
              onChange={(e) =>
                setLocalSettings({ ...localSettings, deviceName: e.target.value })
              }
              className="input"
              placeholder="My Computer"
            />
            <p className="text-xs text-gray-500 mt-1">
              This name is shown to other devices.
            </p>
          </div>
        </div>
      </div>

      {/* Transfer Settings */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Transfers
        </h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Download Directory
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={localSettings.downloadDir}
                onChange={(e) =>
                  setLocalSettings({ ...localSettings, downloadDir: e.target.value })
                }
                className="input flex-1"
              />
              <button onClick={handleSelectDownloadDir} className="btn btn-secondary">
                <FolderOpen className="w-4 h-4" />
              </button>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Receive-only Mode
              </label>
              <p className="text-xs text-gray-500">Disable sending files to others</p>
            </div>
            <input
              type="checkbox"
              checked={localSettings.receiveOnly}
              onChange={(e) =>
                setLocalSettings({ ...localSettings, receiveOnly: e.target.checked })
              }
              className="w-5 h-5 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Max Retries
            </label>
            <input
              type="number"
              value={localSettings.maxRetries}
              onChange={(e) =>
                setLocalSettings({ ...localSettings, maxRetries: Number(e.target.value) })
              }
              className="input w-24"
              min={0}
              max={10}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Bandwidth Limit (MB/s)
            </label>
            <input
              type="number"
              value={localSettings.bandwidthLimitBps ? localSettings.bandwidthLimitBps / 1000000 : ''}
              onChange={(e) =>
                setLocalSettings({
                  ...localSettings,
                  bandwidthLimitBps: e.target.value ? Number(e.target.value) * 1000000 : null,
                })
              }
              className="input w-32"
              min={0}
              placeholder="Unlimited"
            />
          </div>
        </div>
      </div>

      {/* Trusted Hosts */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Trusted Hosts
        </h2>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
          Transfers from trusted hosts are automatically accepted.
        </p>

        <div className="flex gap-2 mb-4">
          <input
            type="text"
            value={newTrustedHost}
            onChange={(e) => setNewTrustedHost(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAddTrustedHost()}
            placeholder="IP address or hostname"
            className="input flex-1"
          />
          <button onClick={handleAddTrustedHost} className="btn btn-secondary">
            <Plus className="w-4 h-4" />
          </button>
        </div>

        {localSettings.trustedHosts.length === 0 ? (
          <p className="text-sm text-gray-400">No trusted hosts configured.</p>
        ) : (
          <div className="space-y-2">
            {localSettings.trustedHosts.map((host, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-2 rounded-lg bg-gray-50 dark:bg-gray-700/50"
              >
                <span className="text-sm text-gray-700 dark:text-gray-300">{host}</span>
                <button
                  onClick={() => handleRemoveTrustedHost(index)}
                  className="p-1 text-gray-400 hover:text-red-500 transition-colors"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Interface Filters */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Interface Visibility
        </h2>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
          Choose which network interface types to show on the Receive page.
        </p>

        <div className="space-y-3">
          {[
            { key: 'showWifi' as const, label: 'WiFi' },
            { key: 'showEthernet' as const, label: 'Ethernet' },
            { key: 'showVpn' as const, label: 'VPN (Tailscale, etc.)' },
            { key: 'showDocker' as const, label: 'Docker / Bridge' },
            { key: 'showOther' as const, label: 'Other' },
          ].map(({ key, label }) => (
            <div key={key} className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">{label}</span>
              <input
                type="checkbox"
                checked={localSettings.interfaceFilters[key]}
                onChange={(e) =>
                  setLocalSettings({
                    ...localSettings,
                    interfaceFilters: {
                      ...localSettings.interfaceFilters,
                      [key]: e.target.checked,
                    },
                  })
                }
                className="w-5 h-5 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
              />
            </div>
          ))}
        </div>
      </div>

      {/* Appearance */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Appearance
        </h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Theme
            </label>
            <div className="flex gap-2">
              {(['system', 'light', 'dark'] as const).map((theme) => (
                <button
                  key={theme}
                  onClick={() => setLocalSettings({ ...localSettings, theme })}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                    localSettings.theme === theme
                      ? 'bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-400'
                      : 'bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
                  }`}
                >
                  {theme.charAt(0).toUpperCase() + theme.slice(1)}
                </button>
              ))}
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Notifications
              </label>
              <p className="text-xs text-gray-500">Show system notifications for transfers</p>
            </div>
            <input
              type="checkbox"
              checked={localSettings.notificationsEnabled}
              onChange={(e) =>
                setLocalSettings({ ...localSettings, notificationsEnabled: e.target.checked })
              }
              className="w-5 h-5 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
            />
          </div>
        </div>
      </div>

      {/* Save Button */}
      {hasChanges && (
        <div className="sticky bottom-6">
          <button
            onClick={handleSave}
            disabled={saving}
            className="btn btn-primary w-full flex items-center justify-center gap-2 py-3"
          >
            {saving ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="w-5 h-5" />
                Save Changes
              </>
            )}
          </button>
        </div>
      )}
    </div>
  );
}
