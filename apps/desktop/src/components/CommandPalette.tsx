import React, { useState, useEffect } from 'react';
import {
  Search,
  LayoutDashboard,
  Activity,
  FolderOpen,
  Files,
  History,
  GitCompare,
  Camera,
  RotateCcw,
  Radio,
  Bell,
  HardDrive,
  Blocks,
  Settings,
  Stethoscope,
  X,
  ArrowRight,
} from 'lucide-react';
import { NavSection } from './Sidebar';

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectSection: (section: NavSection) => void;
  onAddFolder: () => void;
  onCreateSnapshot: () => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  onSelectSection,
  onAddFolder,
  onCreateSnapshot,
}) => {
  const [query, setQuery] = useState('');

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        if (isOpen) onClose();
      } else if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const commands = [
    { id: 'overview', title: 'Go to Overview Dashboard', icon: LayoutDashboard, action: () => onSelectSection('overview') },
    { id: 'activity', title: 'Go to Activity Feed', icon: Activity, action: () => onSelectSection('activity') },
    { id: 'folders', title: 'Go to Monitored Folders', icon: FolderOpen, action: () => onSelectSection('folders') },
    { id: 'files', title: 'Go to File Explorer', icon: Files, action: () => onSelectSection('files') },
    { id: 'versions', title: 'Go to Version History', icon: History, action: () => onSelectSection('versions') },
    { id: 'compare', title: 'Go to File Comparison', icon: GitCompare, action: () => onSelectSection('compare') },
    { id: 'snapshots', title: 'Go to Directory Snapshots', icon: Camera, action: () => onSelectSection('snapshots') },
    { id: 'recovery', title: 'Go to Recovery Center', icon: RotateCcw, action: () => onSelectSection('recovery') },
    { id: 'intelligence', title: 'Go to Change Intelligence', icon: Radio, action: () => onSelectSection('intelligence') },
    { id: 'notifications', title: 'Go to Notifications', icon: Bell, action: () => onSelectSection('notifications') },
    { id: 'storage', title: 'Go to Storage & Retention', icon: HardDrive, action: () => onSelectSection('storage') },
    { id: 'plugins', title: 'Go to Plugins', icon: Blocks, action: () => onSelectSection('plugins') },
    { id: 'settings', title: 'Go to Settings', icon: Settings, action: () => onSelectSection('settings') },
    { id: 'diagnostics', title: 'Run ChangeRadar Doctor', icon: Stethoscope, action: () => onSelectSection('diagnostics') },
    { id: 'act-add-folder', title: 'Action: Add new monitored folder', icon: FolderOpen, action: () => { onClose(); onAddFolder(); } },
    { id: 'act-create-snap', title: 'Action: Create instant point-in-time snapshot', icon: Camera, action: () => { onClose(); onCreateSnapshot(); } },
  ];

  const filtered = commands.filter((c) =>
    c.title.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-start justify-center pt-24">
      <div className="w-full max-w-xl bg-[#181A20] border border-[#2C303A] rounded-xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        <div className="p-3 border-b border-[#2C303A] flex items-center space-x-2">
          <Search className="w-4 h-4 text-[#6C8CFF]" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Type a command or search..."
            className="flex-1 bg-transparent text-sm text-[#F2F3F5] placeholder-[#A1A6B2] focus:outline-none"
            autoFocus
          />
          <button
            onClick={onClose}
            className="p-1 rounded text-[#A1A6B2] hover:text-[#F2F3F5]"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="max-h-80 overflow-y-auto p-2 space-y-1">
          {filtered.length === 0 ? (
            <div className="p-4 text-center text-xs text-[#A1A6B2]">
              No commands matching "{query}"
            </div>
          ) : (
            filtered.map((cmd) => {
              const Icon = cmd.icon;
              return (
                <button
                  key={cmd.id}
                  onClick={() => {
                    cmd.action();
                    onClose();
                  }}
                  className="w-full flex items-center justify-between p-2.5 rounded-lg text-sm text-[#F2F3F5] hover:bg-[#20232B] hover:text-[#6C8CFF] transition-colors text-left group"
                >
                  <div className="flex items-center space-x-3">
                    <Icon className="w-4 h-4 text-[#A1A6B2] group-hover:text-[#6C8CFF]" />
                    <span>{cmd.title}</span>
                  </div>
                  <ArrowRight className="w-3.5 h-3.5 text-[#A1A6B2] opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>
              );
            })
          )}
        </div>

        <div className="p-2.5 bg-[#101114] border-t border-[#2C303A] flex items-center justify-between text-[11px] text-[#A1A6B2]">
          <span>Use <b>↑</b> <b>↓</b> to navigate</span>
          <span>Press <b>ESC</b> to exit</span>
        </div>
      </div>
    </div>
  );
};
