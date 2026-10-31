import React, { useState } from 'react';
import { FolderPlus, X } from 'lucide-react';
import { api } from '../services/api';

interface AddFolderModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export const AddFolderModal: React.FC<AddFolderModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
}) => {
  const [path, setPath] = useState('');
  const [name, setName] = useState('');
  const [loading, setLoading] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!path.trim()) return;

    setLoading(true);
    try {
      await api.addFolder(path.trim(), name.trim() || undefined);
      onSuccess();
      onClose();
      setPath('');
      setName('');
    } catch (err) {
      alert(`Failed to add folder: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="w-full max-w-md bg-[#181A20] border border-[#2C303A] rounded-xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95">
        <div className="p-4 border-b border-[#2C303A] flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <FolderPlus className="w-5 h-5 text-[#6C8CFF]" />
            <h3 className="text-sm font-bold text-[#F2F3F5]">Add Monitored Folder</h3>
          </div>
          <button onClick={onClose} className="p-1 rounded text-[#A1A6B2] hover:text-[#F2F3F5]">
            <X className="w-4 h-4" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 space-y-4 text-xs">
          <div className="space-y-1.5">
            <label className="font-semibold text-[#A1A6B2]">Filesystem Path:</label>
            <input
              type="text"
              required
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="e.g. C:/Users/name/Documents or /home/user/project"
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2.5 text-[#F2F3F5] placeholder-[#A1A6B2] focus:outline-none focus:border-[#6C8CFF]"
            />
          </div>

          <div className="space-y-1.5">
            <label className="font-semibold text-[#A1A6B2]">Display Name (Optional):</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Work Documents"
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2.5 text-[#F2F3F5] placeholder-[#A1A6B2] focus:outline-none focus:border-[#6C8CFF]"
            />
          </div>

          <p className="text-[11px] text-[#A1A6B2]">
            ChangeRadar will immediately start recursive filesystem watching, apply default exclusion rules (`.git`, `node_modules`, temp files), and run an initial reconciliation scan.
          </p>

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
              className="px-4 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white rounded-lg font-semibold shadow-sm transition-colors"
            >
              {loading ? 'Adding...' : 'Start Monitoring'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
