import React from 'react';
import { Search, FolderPlus, Bell, Activity, Sun, Moon, Stethoscope } from 'lucide-react';
import { NavSection } from './Sidebar';

interface HeaderProps {
  currentSection: NavSection;
  onOpenCommandPalette: () => void;
  onQuickAddFolder: () => void;
  onOpenNotifications: () => void;
  onOpenDiagnostics: () => void;
  radarScore?: number;
}

export const Header: React.FC<HeaderProps> = ({
  currentSection,
  onOpenCommandPalette,
  onQuickAddFolder,
  onOpenNotifications,
  onOpenDiagnostics,
  radarScore = 38,
}) => {
  const titles: Record<NavSection, { title: string; subtitle: string }> = {
    overview: { title: 'Overview Dashboard', subtitle: 'Live monitoring state, filesystem velocity and activity metrics' },
    activity: { title: 'Activity Timeline', subtitle: 'Chronological audit feed of all detected filesystem events' },
    folders: { title: 'Monitored Folders', subtitle: 'Manage watched directory trees and exclusion rules' },
    files: { title: 'File Explorer', subtitle: 'Explore tracked file records, metadata, and version count' },
    versions: { title: 'Version History', subtitle: 'Navigate historical content revisions, hashes, notes and pins' },
    compare: { title: 'Universal File Comparison', subtitle: 'Inspect side-by-side and structural diffs across versions' },
    snapshots: { title: 'Directory Snapshots', subtitle: 'Consistent point-in-time state captures and rollbacks' },
    recovery: { title: 'Recovery Center', subtitle: 'Safe, verified, reversible restoration of files and folders' },
    intelligence: { title: 'Change Intelligence', subtitle: 'Radar score metrics, anomaly alerts, and activity patterns' },
    notifications: { title: 'Notification Center', subtitle: 'Custom alert rules and priority event dispatches' },
    storage: { title: 'Storage & Retention', subtitle: 'Content-Addressed Storage, deduplication, and cleanup' },
    plugins: { title: 'Plugin System', subtitle: 'Extensible analyzers, metadata extractors and format providers' },
    settings: { title: 'Settings', subtitle: 'Application preferences, developer API, and privacy controls' },
    diagnostics: { title: 'System Diagnostics', subtitle: 'ChangeRadar Doctor, database checks and CAS integrity' },
  };

  const meta = titles[currentSection];

  return (
    <header className="h-16 border-b border-[#2C303A] bg-[#181A20] px-6 flex items-center justify-between flex-shrink-0">
      <div>
        <h2 className="text-base font-bold text-[#F2F3F5]">{meta.title}</h2>
        <p className="text-xs text-[#A1A6B2]">{meta.subtitle}</p>
      </div>

      <div className="flex items-center space-x-3">
        {/* Command Palette Trigger */}
        <button
          onClick={onOpenCommandPalette}
          className="flex items-center space-x-2 px-3 py-1.5 rounded-md bg-[#20232B] hover:bg-[#252933] text-xs text-[#A1A6B2] border border-[#2C303A] transition-colors"
        >
          <Search className="w-3.5 h-3.5 text-[#6C8CFF]" />
          <span>Quick search or command...</span>
          <kbd className="px-1.5 py-0.5 text-[10px] bg-[#101114] rounded border border-[#2C303A] text-[#A1A6B2]">
            Ctrl K
          </kbd>
        </button>

        {/* Quick Add Folder */}
        <button
          onClick={onQuickAddFolder}
          className="flex items-center space-x-1.5 px-3 py-1.5 rounded-md bg-[#6C8CFF] hover:bg-[#5876E8] text-xs font-semibold text-white shadow-sm transition-colors"
        >
          <FolderPlus className="w-3.5 h-3.5" />
          <span>Add Folder</span>
        </button>

        {/* Radar Score Pill */}
        <div className="flex items-center space-x-1.5 px-2.5 py-1 rounded-md bg-[#20232B] border border-[#2C303A] text-xs">
          <Activity className="w-3.5 h-3.5 text-[#42C98A]" />
          <span className="text-[#A1A6B2]">Radar:</span>
          <span className="font-semibold text-[#42C98A]">{radarScore}/100</span>
        </div>

        {/* Diagnostics Button */}
        <button
          onClick={onOpenDiagnostics}
          title="System Diagnostics & Integrity"
          className="p-2 rounded-md bg-[#20232B] hover:bg-[#252933] text-[#A1A6B2] hover:text-[#F2F3F5] border border-[#2C303A] transition-colors"
        >
          <Stethoscope className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
