import React, { useState } from 'react';
import {
  Camera,
  RotateCcw,
  Calendar,
  Files,
  HardDrive,
  GitCompare,
  Plus,
  CheckCircle2,
  FileCheck,
} from 'lucide-react';
import { Snapshot, MonitoredFolder } from '../types';
import { api } from '../services/api';

interface SnapshotsPageProps {
  snapshots: Snapshot[];
  folders: MonitoredFolder[];
  onRefresh: () => void;
  onCreateSnapshot: () => void;
}

export const SnapshotsPage: React.FC<SnapshotsPageProps> = ({
  snapshots,
  folders,
  onRefresh,
  onCreateSnapshot,
}) => {
  const [restoringId, setRestoringId] = useState<string | null>(null);

  const restoreSnapshot = async (snap: Snapshot) => {
    if (
      confirm(
        `Roll back to snapshot '${snap.name}' (${snap.file_count} files)? Current destination files will be backed up into history prior to overwrite.`
      )
    ) {
      setRestoringId(snap.id);
      try {
        const results = await api.restoreSnapshot(snap.id);
        alert(`Successfully restored ${results.length} files from snapshot '${snap.name}'.`);
        onRefresh();
      } catch (e) {
        alert(`Snapshot restoration failed: ${e}`);
      } finally {
        setRestoringId(null);
      }
    }
  };

  return (
    <div className="space-y-4">
      {/* Top Header */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-[#F2F3F5]">Directory Point-in-Time Snapshots</h3>
          <p className="text-xs text-[#A1A6B2]">
            Consistent manifests capturing the exact version state of all files at an instant.
          </p>
        </div>

        <button
          onClick={onCreateSnapshot}
          className="flex items-center space-x-1.5 px-3.5 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white rounded-lg text-xs font-semibold shadow-sm transition-colors"
        >
          <Camera className="w-4 h-4" />
          <span>New Snapshot</span>
        </button>
      </div>

      {/* Snapshots Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {snapshots.length === 0 ? (
          <div className="col-span-2 p-12 bg-[#181A20] border border-[#2C303A] rounded-xl text-center space-y-3">
            <Camera className="w-10 h-10 text-[#6C8CFF] mx-auto opacity-70" />
            <h4 className="text-base font-bold text-[#F2F3F5]">No Snapshots Created Yet</h4>
            <p className="text-xs text-[#A1A6B2] max-w-md mx-auto">
              Create a snapshot before making major edits, running migrations, or doing batch operations to easily revert your entire folder state with one click.
            </p>
            <button
              onClick={onCreateSnapshot}
              className="px-4 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white text-xs font-semibold rounded-lg transition-colors"
            >
              Create First Snapshot
            </button>
          </div>
        ) : (
          snapshots.map((snap) => (
            <div
              key={snap.id}
              className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-3 hover:border-[#6C8CFF]/50 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start space-x-3">
                  <div className="w-9 h-9 rounded-lg bg-[#20232B] flex items-center justify-center flex-shrink-0">
                    <Camera className="w-5 h-5 text-[#42C98A]" />
                  </div>
                  <div>
                    <h4 className="text-sm font-bold text-[#F2F3F5]">{snap.name}</h4>
                    <p className="text-xs text-[#A1A6B2]">
                      {snap.description || 'Manual state capture'}
                    </p>
                  </div>
                </div>

                <button
                  onClick={() => restoreSnapshot(snap)}
                  disabled={restoringId === snap.id}
                  className="flex items-center space-x-1 px-3 py-1.5 bg-[#20232B] hover:bg-[#6C8CFF] text-[#F2F3F5] hover:text-white rounded-lg border border-[#2C303A] text-xs font-semibold transition-colors"
                >
                  <RotateCcw className="w-3.5 h-3.5" />
                  <span>{restoringId === snap.id ? 'Restoring...' : 'Restore'}</span>
                </button>
              </div>

              <div className="grid grid-cols-2 gap-2 text-xs pt-2 border-t border-[#2C303A]">
                <div className="flex items-center space-x-1.5 text-[#A1A6B2]">
                  <Files className="w-3.5 h-3.5 text-[#6C8CFF]" />
                  <span>Files: <strong className="text-[#F2F3F5]">{snap.file_count}</strong></span>
                </div>
                <div className="flex items-center space-x-1.5 text-[#A1A6B2]">
                  <HardDrive className="w-3.5 h-3.5 text-[#E9B65B]" />
                  <span>
                    Size:{' '}
                    <strong className="text-[#F2F3F5]">
                      {(snap.total_bytes / (1024 * 1024)).toFixed(1)} MB
                    </strong>
                  </span>
                </div>
              </div>

              <div className="flex items-center justify-between text-[11px] text-[#A1A6B2] pt-2 border-t border-[#2C303A] font-mono">
                <span className="flex items-center gap-1">
                  <Calendar className="w-3.5 h-3.5" />
                  {new Date(snap.created_at).toLocaleString()}
                </span>
                <span>ID: {snap.id.slice(0, 8)}...</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
