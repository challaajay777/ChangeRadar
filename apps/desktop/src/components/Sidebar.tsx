import React from 'react';
import {
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
  LayoutDashboard,
  ShieldCheck,
} from 'lucide-react';

export type NavSection =
  | 'overview'
  | 'activity'
  | 'folders'
  | 'files'
  | 'versions'
  | 'compare'
  | 'snapshots'
  | 'recovery'
  | 'intelligence'
  | 'notifications'
  | 'storage'
  | 'plugins'
  | 'settings'
  | 'diagnostics';

interface SidebarProps {
  currentSection: NavSection;
  onSelectSection: (section: NavSection) => void;
  unreadNotificationsCount: number;
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentSection,
  onSelectSection,
  unreadNotificationsCount,
}) => {
  const navItems = [
    { id: 'overview', label: 'Overview', icon: LayoutDashboard },
    { id: 'activity', label: 'Activity', icon: Activity },
    { id: 'folders', label: 'Monitored Folders', icon: FolderOpen },
    { id: 'files', label: 'File Explorer', icon: Files },
    { id: 'versions', label: 'Version History', icon: History },
    { id: 'compare', label: 'Compare', icon: GitCompare },
    { id: 'snapshots', label: 'Snapshots', icon: Camera },
    { id: 'recovery', label: 'Recovery Center', icon: RotateCcw },
    { id: 'intelligence', label: 'Change Intelligence', icon: Radio },
    { id: 'notifications', label: 'Notifications', icon: Bell, badge: unreadNotificationsCount },
    { id: 'storage', label: 'Storage', icon: HardDrive },
    { id: 'plugins', label: 'Plugins', icon: Blocks },
    { id: 'settings', label: 'Settings', icon: Settings },
    { id: 'diagnostics', label: 'Diagnostics', icon: Stethoscope },
  ] as const;

  return (
    <aside className="w-64 bg-[#181A20] border-r border-[#2C303A] flex flex-col h-screen select-none flex-shrink-0">
      {/* Brand Header */}
      <div className="p-4 border-b border-[#2C303A] flex items-center space-x-3">
        <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-[#6C8CFF] to-[#3B54D4] flex items-center justify-center shadow-lg shadow-[#6C8CFF]/20">
          <Radio className="w-5 h-5 text-white animate-pulse" />
        </div>
        <div>
          <h1 className="text-base font-bold text-[#F2F3F5] tracking-tight flex items-center gap-1.5">
            ChangeRadar
            <span className="text-[10px] uppercase font-semibold px-1.5 py-0.5 rounded bg-[#6C8CFF]/20 text-[#6C8CFF] border border-[#6C8CFF]/30">
              v1.0
            </span>
          </h1>
          <p className="text-xs text-[#A1A6B2]">Universal File Intelligence</p>
        </div>
      </div>

      {/* Navigation List */}
      <nav className="flex-1 overflow-y-auto p-2 space-y-0.5">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentSection === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onSelectSection(item.id as NavSection)}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                isActive
                  ? 'bg-[#20232B] text-[#6C8CFF] font-semibold border-l-2 border-[#6C8CFF]'
                  : 'text-[#A1A6B2] hover:bg-[#20232B]/60 hover:text-[#F2F3F5]'
              }`}
            >
              <div className="flex items-center space-x-3">
                <Icon className={`w-4 h-4 ${isActive ? 'text-[#6C8CFF]' : 'text-[#A1A6B2]'}`} />
                <span>{item.label}</span>
              </div>
              {'badge' in item && item.badge ? (
                <span className="px-1.5 py-0.2 text-[11px] font-bold rounded-full bg-[#EA6868] text-white">
                  {item.badge}
                </span>
              ) : null}
            </button>
          );
        })}
      </nav>

      {/* Footer System Status */}
      <div className="p-3 border-t border-[#2C303A] bg-[#101114]/50">
        <div className="flex items-center justify-between text-xs text-[#A1A6B2]">
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-[#42C98A] inline-block animate-ping" />
            <span className="text-[#42C98A] font-medium">Local-first & Offline</span>
          </span>
          <ShieldCheck className="w-4 h-4 text-[#42C98A]" />
        </div>
      </div>
    </aside>
  );
};
