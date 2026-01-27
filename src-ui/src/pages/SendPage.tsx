import { useState, useEffect } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Send,
  FolderOpen,
  File,
  Star,
  Plus,
  Trash2,
  Check,
  X,
  Loader2,
} from 'lucide-react';
import { useAppStore } from '../store';
import type { Favorite } from '../types';

export function SendPage() {
  const {
    settings,
    favorites,
    sendFiles,
    sendDirectory,
    resolveAddress,
    checkPeer,
    addFavorite,
    deleteFavorite,
    touchFavorite,
  } = useAppStore();

  const [destination, setDestination] = useState('');
  const [port, setPort] = useState(53317);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [resolvedIp, setResolvedIp] = useState<string | null>(null);
  const [resolving, setResolving] = useState(false);
  const [peerReachable, setPeerReachable] = useState<boolean | null>(null);
  const [sending, setSending] = useState(false);
  const [showAddFavorite, setShowAddFavorite] = useState(false);
  const [newFavoriteName, setNewFavoriteName] = useState('');

  // Resolve address when destination changes
  useEffect(() => {
    const timeout = setTimeout(async () => {
      if (!destination.trim()) {
        setResolvedIp(null);
        setPeerReachable(null);
        return;
      }

      setResolving(true);
      try {
        const result = await resolveAddress(destination);
        setResolvedIp(result.ip);

        if (result.ip) {
          const reachable = await checkPeer(result.ip, port);
          setPeerReachable(reachable);
        } else {
          setPeerReachable(false);
        }
      } catch {
        setResolvedIp(null);
        setPeerReachable(false);
      } finally {
        setResolving(false);
      }
    }, 500);

    return () => clearTimeout(timeout);
  }, [destination, port, resolveAddress, checkPeer]);

  const handleSelectFiles = async () => {
    const files = await open({
      multiple: true,
      directory: false,
    });
    if (files) {
      const paths = Array.isArray(files) ? files : [files];
      setSelectedPaths((prev) => [...prev, ...paths]);
    }
  };

  const handleSelectFolder = async () => {
    const folder = await open({
      multiple: false,
      directory: true,
    });
    if (folder) {
      setSelectedPaths((prev) => [...prev, folder]);
    }
  };

  const handleRemovePath = (index: number) => {
    setSelectedPaths((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSend = async () => {
    if (!resolvedIp || !selectedPaths.length) return;

    setSending(true);
    try {
      // Check if it's a single directory
      if (selectedPaths.length === 1) {
        const path = selectedPaths[0];
        // Simple heuristic: if path doesn't have extension, treat as directory
        // In production, you'd want to check this properly
        await sendDirectory(resolvedIp, port, path);
      } else {
        await sendFiles(resolvedIp, port, selectedPaths);
      }

      // Touch favorite if selected from favorites
      const matchingFavorite = favorites.find(
        (f) => f.address === destination || f.last_resolved_ip === resolvedIp
      );
      if (matchingFavorite) {
        await touchFavorite(matchingFavorite.id);
      }

      setSelectedPaths([]);
    } finally {
      setSending(false);
    }
  };

  const handleAddFavorite = async () => {
    if (!newFavoriteName.trim() || !destination.trim()) return;
    await addFavorite(newFavoriteName, destination);
    setNewFavoriteName('');
    setShowAddFavorite(false);
  };

  const handleSelectFavorite = (favorite: Favorite) => {
    setDestination(favorite.address);
  };

  if (settings?.receiveOnly) {
    return (
      <div className="p-6">
        <div className="card p-8 text-center">
          <p className="text-gray-500 dark:text-gray-400">
            Sending is disabled in receive-only mode.
          </p>
          <p className="text-sm text-gray-400 dark:text-gray-500 mt-2">
            You can enable sending in Settings.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Destination */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Destination
        </h2>

        <div className="flex gap-3">
          <div className="flex-1">
            <div className="relative">
              <input
                type="text"
                value={destination}
                onChange={(e) => setDestination(e.target.value)}
                placeholder="Hostname or IP address"
                className="input pr-10"
              />
              {resolving && (
                <Loader2 className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 animate-spin text-gray-400" />
              )}
              {!resolving && peerReachable === true && (
                <Check className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-green-500" />
              )}
              {!resolving && peerReachable === false && destination && (
                <X className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-red-500" />
              )}
            </div>
            {resolvedIp && resolvedIp !== destination && (
              <p className="text-xs text-gray-500 mt-1">Resolved to {resolvedIp}</p>
            )}
          </div>

          <div className="w-24">
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(Number(e.target.value))}
              className="input"
              min={1}
              max={65535}
            />
          </div>
        </div>
      </div>

      {/* Favorites */}
      <div className="card p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
            <Star className="w-5 h-5" />
            Favorites
          </h2>
          <button
            onClick={() => setShowAddFavorite(true)}
            className="btn btn-secondary text-sm flex items-center gap-1"
            disabled={!destination.trim()}
          >
            <Plus className="w-4 h-4" />
            Add
          </button>
        </div>

        {showAddFavorite && (
          <div className="flex gap-2 mb-4">
            <input
              type="text"
              value={newFavoriteName}
              onChange={(e) => setNewFavoriteName(e.target.value)}
              placeholder="Name for this favorite"
              className="input flex-1"
              autoFocus
            />
            <button onClick={handleAddFavorite} className="btn btn-primary">
              Save
            </button>
            <button
              onClick={() => {
                setShowAddFavorite(false);
                setNewFavoriteName('');
              }}
              className="btn btn-secondary"
            >
              Cancel
            </button>
          </div>
        )}

        {favorites.length === 0 ? (
          <p className="text-gray-500 dark:text-gray-400 text-sm">
            No favorites yet. Add a destination above to save it.
          </p>
        ) : (
          <div className="space-y-2">
            {favorites.map((favorite) => (
              <div
                key={favorite.id}
                className="flex items-center justify-between p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer"
                onClick={() => handleSelectFavorite(favorite)}
              >
                <div>
                  <p className="font-medium text-gray-900 dark:text-white">
                    {favorite.name}
                  </p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {favorite.address}
                  </p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    deleteFavorite(favorite.id);
                  }}
                  className="p-2 text-gray-400 hover:text-red-500 transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Files */}
      <div className="card p-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Files to Send
        </h2>

        <div className="flex gap-3 mb-4">
          <button onClick={handleSelectFiles} className="btn btn-secondary flex items-center gap-2">
            <File className="w-4 h-4" />
            Select Files
          </button>
          <button onClick={handleSelectFolder} className="btn btn-secondary flex items-center gap-2">
            <FolderOpen className="w-4 h-4" />
            Select Folder
          </button>
        </div>

        {selectedPaths.length > 0 ? (
          <div className="space-y-2">
            {selectedPaths.map((path, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50"
              >
                <span className="text-sm text-gray-700 dark:text-gray-300 truncate flex-1">
                  {path}
                </span>
                <button
                  onClick={() => handleRemovePath(index)}
                  className="p-1 text-gray-400 hover:text-red-500 transition-colors ml-2"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-sm">
            No files selected. Click the buttons above to choose files or folders.
          </p>
        )}
      </div>

      {/* Send Button */}
      <button
        onClick={handleSend}
        disabled={!peerReachable || !selectedPaths.length || sending}
        className="btn btn-primary w-full flex items-center justify-center gap-2 py-3"
      >
        {sending ? (
          <>
            <Loader2 className="w-5 h-5 animate-spin" />
            Sending...
          </>
        ) : (
          <>
            <Send className="w-5 h-5" />
            Send Files
          </>
        )}
      </button>
    </div>
  );
}
