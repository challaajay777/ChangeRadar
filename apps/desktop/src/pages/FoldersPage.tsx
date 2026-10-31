import React, { useState } from 'react';
import {
  FolderOpen,
  FolderPlus,
  Play,
  Pause,
  Trash2,
  ExternalLink,
  Shield,
  Clock,
  CheckCircle2,
} from 'lucide-react';
import { MonitoredFolder } from '../types';
import { api } from '../services/api';

interface FoldersPageProps {
  folders: MonitoredFolder[];
  onRefresh: () => void;
  onOpenAddModal: () => void;
}

export const FoldersPage: React.FC<FoldersPageProps> = ({
  folders,
  onRefresh,
  onOpenAddModal,
}) => {
  const [loadingId, setLoadingId] = useState<string | null>(null);

  const togglePause = async (folder: MonitoredFolder) => {
    setLoadingId(folder.id);
    try {
      if (folder.is_active) {
        await api.pauseFolder(folder.id);
      } else {
        await api.resumeFolder(folder.id);
      }
      onRefresh();
    } catch (e) {
      alert(`Failed to update folder status: ${e}`);
    } finally {
      setLoadingId(null);
    }
  };

  const removeFolder = async (folder: MonitoredFolder) => {
    if (confirm(`Stop monitoring and remove '${folder.display_name}'? Historical versions remain safely in storage.`)) {
      setLoadingId(folder.id);
      try {
        await api.removeFolder(folder.id);
        onRefresh();
      } catch (e) {
        alert(`Failed to remove folder: ${e}`);
      } finally {
        setLoadingId(null);
      }
    }
  };

  const openFolder = async (path: string) => {
    try {
      await api.revealInExplorer(path);
    } catch (e) {
      console.warn('Could not open folder in native explorer:', e);
    }
  };

  return (
    <div className="space-y-4">
      {/* Top Action Bar */}
      <div className="flex items-center justify-between bg-[#181A20] border border-[#2C303A] rounded-xl p-4">
        <div>
          <h3 className="text-sm font-bold text-[#F2F3F5]">Configured Watch Trees</h3>
          <p className="text-xs text-[#A1A6B2]">
            Folders currently monitored for real-time creation, deletion, rename, and content shifts.
          </p>
        </div>

        <button
          onClick={onOpenAddModal}
          className="flex items-center space-x-1.5 px-3.5 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white rounded-lg text-xs font-semibold shadow-sm transition-colors"
        >
          <FolderPlus className="w-4 h-4" />
          <span>Add Folder</span>
        </button>
      </div>

      {/* Folder Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {folders.length === 0 ? (
          <div className="col-span-2 p-12 bg-[#181A20] border border-[#2C303A] rounded-xl text-center space-y-3">
            <FolderOpen className="w-10 h-10 text-[#6C8CFF] mx-auto opacity-70" />
            <h4 className="text-base font-bold text-[#F2F3F5]">No Monitored Folders</h4>
            <p className="text-xs text-[#A1A6B2] max-w-md mx-auto">
              Add your first workspace, documents directory, or code project to immediately begin tracking file changes and preserving history.
            </p>
            <button
              onClick={onOpenAddModal}
              className="px-4 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white text-xs font-semibold rounded-lg transition-colors"
            >
              Add Folder Now
            </button>
          </div>
        ) : (
          folders.map((folder) => (
            <div
              key={folder.id}
              className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-4 hover:border-[#6C8CFF]/50 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start space-x-3">
                  <div className="w-9 h-9 rounded-lg bg-[#20232B] flex items-center justify-center flex-shrink-0">
                    <FolderOpen className="w-5 h-5 text-[#6C8CFF]" />
                  </div>
                  <div>
                    <h4 className="text-sm font-bold text-[#F2F3F5] flex items-center gap-2">
                      {folder.display_name}
                      <span
                        className={`text-[10px] font-bold px-2 py-0.5 rounded ${
                          folder.is_active
                            ? 'bg-[#42C98A]/20 text-[#42C98A]'
                            : 'bg-[#A1A6B2]/20 text-[#A1A6B2]'
                        }`}
                      >
                        {folder.is_active ? 'ACTIVE' : 'PAUSED'}
                      </span>
                    </h4>
                    <p className="text-xs text-[#A1A6B2] font-mono mt-0.5 break-all">
                      {folder.path}
                    </p>
                  </div>
                </div>

                <div className="flex items-center space-x-1.5">
                  <button
                    onClick={() => togglePause(folder)}
                    disabled={loadingId === folder.id}
                    title={folder.is_active ? 'Pause monitoring' : 'Resume monitoring'}
                    className="p-1.5 rounded-lg bg-[#20232B] hover:bg-[#252933] text-[#A1A6B2] hover:text-[#F2F3F5] border border-[#2C303A] transition-colors"
                  >
                    {folder.is_active ? (
                      <Pause className="w-4 h-4 text-[#E9B65B]" />
                    ) : (
                      <Play className="w-4 h-4 text-[#42C98A]" />
                    )}
                  </button>

                  <button
                    onClick={() => openFolder(folder.path)}
                    title="Open in System File Explorer"
                    className="p-1.5 rounded-lg bg-[#20232B] hover:bg-[#252933] text-[#A1A6B2] hover:text-[#F2F3F5] border border-[#2C303A] transition-colors"
                  >
                    <ExternalLink className="w-4 h-4" />
                  </button>

                  <button
                    onClick={() => removeFolder(folder)}
                    disabled={loadingId === folder.id}
                    title="Remove from monitoring"
                    className="p-1.5 rounded-lg bg-[#20232B] hover:bg-[#EA6868]/20 text-[#A1A6B2] hover:text-[#EA6868] border border-[#2C303A] transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>

              {/* Exclusion Patterns */}
              <div className="space-y-1.5 pt-2 border-t border-[#2C303A]">
                <div className="text-[11px] text-[#A1A6B2] flex items-center gap-1">
                  <Shield className="w-3.5 h-3.5 text-[#6C8CFF]" />
                  <span>Exclusion Rules:</span>
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {folder.ignore_patterns.map((pat, idx) => (
                    <span
                      key={idx}
                      className="px-2 py-0.5 bg-[#20232B] rounded text-[11px] text-[#A1A6B2] font-mono border border-[#2C303A]"
                    >
                      {pat}
                    </span>
                  ))}
                </div>
              </div>

              {/* Reconciliation & Settings Footprint */}
              <div className="flex items-center justify-between text-[11px] text-[#A1A6B2] pt-2 border-t border-[#2C303A]">
                <span className="flex items-center gap-1">
                  <Clock className="w-3.5 h-3.5" />
                  Reconciliation: Every {folder.scan_interval_secs}s
                </span>
                <span>ID: {folder.id.slice(0, 8)}...</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
