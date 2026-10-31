import React from 'react';
import {
  FolderOpen,
  Files,
  Activity,
  History,
  HardDrive,
  Radio,
  AlertTriangle,
  FolderPlus,
  Camera,
  RotateCcw,
  CheckCircle2,
  Clock,
  ArrowUpRight,
} from 'lucide-react';
import { SystemOverview, FileEvent } from '../types';
import { NavSection } from '../components/Sidebar';

interface OverviewPageProps {
  overview: SystemOverview;
  recentEvents: FileEvent[];
  onNavigate: (section: NavSection) => void;
  onAddFolder: () => void;
  onCreateSnapshot: () => void;
}

export const OverviewPage: React.FC<OverviewPageProps> = ({
  overview,
  recentEvents,
  onNavigate,
  onAddFolder,
  onCreateSnapshot,
}) => {
  return (
    <div className="space-y-6">
      {/* Top Banner: Status & Radar Score */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Monitoring State Card */}
        <div className="lg:col-span-2 bg-[#181A20] border border-[#2C303A] rounded-xl p-5 flex items-center justify-between">
          <div className="space-y-1">
            <div className="flex items-center space-x-2">
              <span className="w-2.5 h-2.5 rounded-full bg-[#42C98A] inline-block animate-pulse" />
              <span className="text-xs font-bold text-[#42C98A] uppercase tracking-wider">
                Monitoring Engine Active
              </span>
            </div>
            <h3 className="text-xl font-bold text-[#F2F3F5]">
              Realtime Filesystem Watcher Running
            </h3>
            <p className="text-xs text-[#A1A6B2] max-w-lg">
              ChangeRadar is continuously monitoring {overview.active_folders} active directory trees, capturing atomic versions, deduplicating with Content-Addressed Storage, and validating file integrity.
            </p>
          </div>
          <div className="flex flex-col items-end space-y-2">
            <button
              onClick={onAddFolder}
              className="flex items-center space-x-1.5 px-3 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white rounded-lg text-xs font-semibold shadow-md transition-colors"
            >
              <FolderPlus className="w-4 h-4" />
              <span>Watch New Folder</span>
            </button>
            <span className="text-[11px] text-[#A1A6B2]">
              Reconciliation scan: every 5m
            </span>
          </div>
        </div>

        {/* Change Radar Score Meter */}
        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 flex flex-col justify-between">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Radio className="w-4 h-4 text-[#6C8CFF]" />
              <span className="text-xs font-bold text-[#A1A6B2] uppercase">Change Radar Score</span>
            </div>
            <span className="text-[11px] px-2 py-0.5 rounded bg-[#42C98A]/20 text-[#42C98A] font-semibold">
              Normal Activity
            </span>
          </div>

          <div className="my-2 flex items-baseline space-x-2">
            <span className="text-4xl font-extrabold text-[#F2F3F5] tracking-tight">
              {overview.radar_score}
            </span>
            <span className="text-sm font-semibold text-[#A1A6B2]">/ 100</span>
          </div>

          <div className="w-full bg-[#20232B] h-2 rounded-full overflow-hidden">
            <div
              className="bg-gradient-to-r from-[#42C98A] via-[#E9B65B] to-[#EA6868] h-full transition-all duration-500"
              style={{ width: `${Math.min(overview.radar_score, 100)}%` }}
            />
          </div>

          <p className="text-[11px] text-[#A1A6B2] mt-2">
            Transparent filesystem volatility index calculated from daily change volume, active trees, and churn.
          </p>
        </div>
      </div>

      {/* Key Metric Cards */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
        <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3.5 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Monitored Trees</span>
            <FolderOpen className="w-4 h-4 text-[#6C8CFF]" />
          </div>
          <div className="text-xl font-bold text-[#F2F3F5]">{overview.active_folders}</div>
          <div className="text-[10px] text-[#A1A6B2]">All recursive watchers active</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3.5 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Tracked Files</span>
            <Files className="w-4 h-4 text-[#6C8CFF]" />
          </div>
          <div className="text-xl font-bold text-[#F2F3F5]">{overview.total_tracked_files.toLocaleString()}</div>
          <div className="text-[10px] text-[#A1A6B2]">Unique paths tracked</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3.5 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Changes Today</span>
            <Activity className="w-4 h-4 text-[#42C98A]" />
          </div>
          <div className="text-xl font-bold text-[#42C98A]">{overview.changes_today}</div>
          <div className="text-[10px] text-[#A1A6B2]">Events since 00:00 UTC</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3.5 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Stored Versions</span>
            <History className="w-4 h-4 text-[#6C8CFF]" />
          </div>
          <div className="text-xl font-bold text-[#F2F3F5]">{overview.versions_stored.toLocaleString()}</div>
          <div className="text-[10px] text-[#A1A6B2]">Historical revisions stored</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3.5 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">CAS Storage</span>
            <HardDrive className="w-4 h-4 text-[#E9B65B]" />
          </div>
          <div className="text-xl font-bold text-[#F2F3F5]">
            {(overview.total_cas_bytes / (1024 * 1024)).toFixed(1)} MB
          </div>
          <div className="text-[10px] text-[#A1A6B2]">Zstd compressed storage</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3.5 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Deduplication</span>
            <CheckCircle2 className="w-4 h-4 text-[#42C98A]" />
          </div>
          <div className="text-xl font-bold text-[#42C98A]">
            {overview.deduplication_savings_pct.toFixed(1)}%
          </div>
          <div className="text-[10px] text-[#A1A6B2]">Disk storage saved</div>
        </div>
      </div>

      {/* Quick Action Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
        <button
          onClick={onAddFolder}
          className="p-4 bg-[#181A20] hover:bg-[#20232B] border border-[#2C303A] rounded-xl text-left transition-colors group flex items-start justify-between"
        >
          <div>
            <FolderPlus className="w-5 h-5 text-[#6C8CFF] mb-2" />
            <h4 className="text-sm font-semibold text-[#F2F3F5]">Add Monitored Folder</h4>
            <p className="text-xs text-[#A1A6B2]">Track documents, codebases or downloads</p>
          </div>
          <ArrowUpRight className="w-4 h-4 text-[#A1A6B2] group-hover:text-[#6C8CFF] transition-colors" />
        </button>

        <button
          onClick={onCreateSnapshot}
          className="p-4 bg-[#181A20] hover:bg-[#20232B] border border-[#2C303A] rounded-xl text-left transition-colors group flex items-start justify-between"
        >
          <div>
            <Camera className="w-5 h-5 text-[#42C98A] mb-2" />
            <h4 className="text-sm font-semibold text-[#F2F3F5]">Create Snapshot</h4>
            <p className="text-xs text-[#A1A6B2]">Freeze entire folder state for rollback</p>
          </div>
          <ArrowUpRight className="w-4 h-4 text-[#A1A6B2] group-hover:text-[#42C98A] transition-colors" />
        </button>

        <button
          onClick={() => onNavigate('compare')}
          className="p-4 bg-[#181A20] hover:bg-[#20232B] border border-[#2C303A] rounded-xl text-left transition-colors group flex items-start justify-between"
        >
          <div>
            <Activity className="w-5 h-5 text-[#E9B65B] mb-2" />
            <h4 className="text-sm font-semibold text-[#F2F3F5]">Universal Compare</h4>
            <p className="text-xs text-[#A1A6B2]">Inspect diffs in text, JSON, CSV and images</p>
          </div>
          <ArrowUpRight className="w-4 h-4 text-[#A1A6B2] group-hover:text-[#E9B65B] transition-colors" />
        </button>

        <button
          onClick={() => onNavigate('recovery')}
          className="p-4 bg-[#181A20] hover:bg-[#20232B] border border-[#2C303A] rounded-xl text-left transition-colors group flex items-start justify-between"
        >
          <div>
            <RotateCcw className="w-5 h-5 text-[#EA6868] mb-2" />
            <h4 className="text-sm font-semibold text-[#F2F3F5]">Safe Recovery Center</h4>
            <p className="text-xs text-[#A1A6B2]">Revert deleted or corrupted revisions</p>
          </div>
          <ArrowUpRight className="w-4 h-4 text-[#A1A6B2] group-hover:text-[#EA6868] transition-colors" />
        </button>
      </div>

      {/* Recent Activity Table Preview */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl overflow-hidden">
        <div className="p-4 border-b border-[#2C303A] flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Clock className="w-4 h-4 text-[#6C8CFF]" />
            <h3 className="text-sm font-bold text-[#F2F3F5]">Live Change Activity</h3>
          </div>
          <button
            onClick={() => onNavigate('activity')}
            className="text-xs text-[#6C8CFF] hover:underline font-medium"
          >
            View Full Timeline →
          </button>
        </div>

        <div className="divide-y divide-[#2C303A]">
          {recentEvents.length === 0 ? (
            <div className="p-8 text-center text-xs text-[#A1A6B2]">
              No change events detected yet. Monitored folders will stream changes here in real time.
            </div>
          ) : (
            recentEvents.slice(0, 6).map((ev) => (
              <div
                key={ev.id}
                className="p-3.5 flex items-center justify-between hover:bg-[#20232B]/50 transition-colors"
              >
                <div className="flex items-center space-x-3">
                  <span
                    className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                      ev.event_type === 'CREATED'
                        ? 'bg-[#42C98A]/20 text-[#42C98A]'
                        : ev.event_type === 'DELETED'
                        ? 'bg-[#EA6868]/20 text-[#EA6868]'
                        : ev.event_type === 'RESTORED'
                        ? 'bg-[#6C8CFF]/20 text-[#6C8CFF]'
                        : 'bg-[#E9B65B]/20 text-[#E9B65B]'
                    }`}
                  >
                    {ev.event_type}
                  </span>
                  <div>
                    <div className="text-xs font-semibold text-[#F2F3F5] font-mono">
                      {ev.new_path}
                    </div>
                    <div className="text-[11px] text-[#A1A6B2]">{ev.summary}</div>
                  </div>
                </div>

                <div className="text-right">
                  <div className="text-[11px] text-[#A1A6B2]">
                    {new Date(ev.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                  </div>
                  <span className="text-[10px] text-[#A1A6B2] font-mono">
                    {ev.confidence}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
