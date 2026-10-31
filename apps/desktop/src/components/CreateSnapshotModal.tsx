import React, { useState } from 'react';
import { Camera, X } from 'lucide-react';
import { MonitoredFolder } from '../types';
import { api } from '../services/api';

interface CreateSnapshotModalProps {
  isOpen: boolean;
  folders: MonitoredFolder[];
  onClose: () => void;
  onSuccess: () => void;
}

export const CreateSnapshotModal: React.FC<CreateSnapshotModalProps> = ({
  isOpen,
  folders,
  onClose,
  onSuccess,
}) => {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [folderId, setFolderId] = useState<string>('ALL');
  const [loading, setLoading] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    setLoading(true);
    try {
      await api.createSnapshot(
        name.trim(),
        description.trim() || undefined,
        folderId === 'ALL' ? undefined : folderId
      );
      onSuccess();
      onClose();
      setName('');
      setDescription('');
    } catch (err) {
      alert(`Failed to create snapshot: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="w-full max-w-md bg-[#181A20] border border-[#2C303A] rounded-xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95">
        <div className="p-4 border-b border-[#2C303A] flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Camera className="w-5 h-5 text-[#42C98A]" />
            <h3 className="text-sm font-bold text-[#F2F3F5]">Create Directory Snapshot</h3>
          </div>
          <button onClick={onClose} className="p-1 rounded text-[#A1A6B2] hover:text-[#F2F3F5]">
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 space-y-4 text-xs">
          <div className="space-y-1.5">
            <label className="font-semibold text-[#A1A6B2]">Snapshot Name:</label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Pre-Release Milestone"
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2.5 text-[#F2F3F5] placeholder-[#A1A6B2] focus:outline-none focus:border-[#6C8CFF]"
            />
          </div>

          <div className="space-y-1.5">
            <label className="font-semibold text-[#A1A6B2]">Scope:</label>
            <select
              value={folderId}
              onChange={(e) => setFolderId(e.target.value)}
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2.5 text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            >
              <option value="ALL">Global Snapshot (All Monitored Folders)</option>
              {folders.map((f) => (
                <option key={f.id} value={f.id}>
                  {f.display_name}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1.5">
            <label className="font-semibold text-[#A1A6B2]">Description (Optional):</label>
            <textarea
              rows={2}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Notes about why this state capture was created..."
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2.5 text-[#F2F3F5] placeholder-[#A1A6B2] focus:outline-none focus:border-[#6C8CFF]"
            />
          </div>

          <div className="pt-2 flex justify-end space-x-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 bg-[#20232B] hover:bg-[#252933] text-[#F2F3F5] rounded-lg font-semibold transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 bg-[#42C98A] hover:bg-[#36B278] text-white rounded-lg font-semibold shadow-sm transition-colors"
            >
              {loading ? 'Creating Snapshot...' : 'Create Snapshot'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
