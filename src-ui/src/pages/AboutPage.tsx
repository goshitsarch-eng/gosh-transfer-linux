import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import { ExternalLink, Github, Heart } from 'lucide-react';

export function AboutPage() {
  const [version, setVersion] = useState('');

  useEffect(() => {
    invoke<string>('get_version').then(setVersion);
  }, []);

  const openLink = (url: string) => {
    open(url);
  };

  return (
    <div className="p-6">
      <div className="card p-8 max-w-md mx-auto text-center">
        <div className="mb-6">
          <div className="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-primary-400 to-primary-600 flex items-center justify-center">
            <span className="text-3xl font-bold text-white">G</span>
          </div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Gosh Transfer
          </h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Version {version}
          </p>
        </div>

        <p className="text-gray-600 dark:text-gray-300 mb-6">
          Simple, explicit file transfers over LAN, Tailscale, and VPNs.
        </p>

        <div className="space-y-3">
          <button
            onClick={() => openLink('https://github.com/goshitsarch-eng/gosh-transfer-linux')}
            className="w-full btn btn-secondary flex items-center justify-center gap-2"
          >
            <Github className="w-4 h-4" />
            View on GitHub
          </button>

          <button
            onClick={() => openLink('https://github.com/goshitsarch-eng/gosh-transfer-linux/issues')}
            className="w-full btn btn-secondary flex items-center justify-center gap-2"
          >
            <ExternalLink className="w-4 h-4" />
            Report an Issue
          </button>
        </div>

        <div className="mt-8 pt-6 border-t border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400 flex items-center justify-center gap-1">
            Made with <Heart className="w-4 h-4 text-red-500" /> by Goshitsarch
          </p>
          <p className="text-xs text-gray-400 dark:text-gray-500 mt-2">
            Licensed under AGPL-3.0
          </p>
        </div>
      </div>
    </div>
  );
}
