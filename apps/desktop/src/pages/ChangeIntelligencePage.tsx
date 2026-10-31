import React, { useState, useEffect } from 'react';
import {
  Radio,
  Activity,
  AlertTriangle,
  TrendingUp,
  FileText,
  FolderOpen,
  Calendar,
  CheckCircle2,
} from 'lucide-react';
import { ActivityStats, AnomalyAlert } from '../types';
import { api } from '../services/api';

interface ChangeIntelligencePageProps {
  radarScore?: number;
}

export const ChangeIntelligencePage: React.FC<ChangeIntelligencePageProps> = ({
  radarScore = 38,
}) => {
  const [stats, setStats] = useState<ActivityStats | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    api
      .getActivityStats()
      .then((s) => setStats(s))
      .catch((e) => console.warn('Activity stats error:', e))
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="space-y-5">
      {/* Top Banner: Radar Score Breakdown */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 grid grid-cols-1 md:grid-cols-3 gap-6 items-center">
        <div className="space-y-1">
          <div className="flex items-center space-x-2">
            <Radio className="w-5 h-5 text-[#6C8CFF]" />
            <h3 className="text-base font-bold text-[#F2F3F5]">Change Radar Score</h3>
          </div>
          <p className="text-xs text-[#A1A6B2]">
            A transparent metric of filesystem change velocity, calculated from daily change counts, active watch trees, and file churn. Normal activity score is typically between 10 and 50.
          </p>
        </div>

        <div className="flex flex-col items-center justify-center p-4 bg-[#20232B] rounded-xl border border-[#2C303A]">
          <div className="text-4xl font-extrabold text-[#F2F3F5] tracking-tight">
            {radarScore} <span className="text-lg text-[#A1A6B2] font-normal">/ 100</span>
          </div>
          <span className="text-xs text-[#42C98A] font-semibold mt-1">
            Normal & Healthy Activity
          </span>
        </div>

        <div className="space-y-2 text-xs text-[#A1A6B2]">
          <div className="flex justify-between">
            <span>Change Volume Weight:</span>
            <span className="text-[#F2F3F5] font-mono">4 pts / event</span>
          </div>
          <div className="flex justify-between">
            <span>Active Tree Weight:</span>
            <span className="text-[#F2F3F5] font-mono">6 pts / folder</span>
          </div>
          <div className="flex justify-between">
            <span>Anomaly Alert Weight:</span>
            <span className="text-[#F2F3F5] font-mono">25 pts / burst</span>
          </div>
        </div>
      </div>

      {/* 7-Day Change Activity Chart */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <TrendingUp className="w-4 h-4 text-[#42C98A]" />
            <h4 className="text-sm font-bold text-[#F2F3F5]">7-Day Change Velocity</h4>
          </div>
          <span className="text-xs text-[#A1A6B2]">Events / 24 Hours</span>
        </div>

        {/* Visual Bar Chart */}
        <div className="h-44 flex items-end justify-between gap-3 pt-6 px-4">
          {stats?.daily_changes.map((day) => {
            const maxVal = Math.max(...(stats?.daily_changes.map((d) => d.count) || [1]), 10);
            const heightPct = Math.max((day.count / maxVal) * 100, 6);
            return (
              <div key={day.date} className="flex-1 flex flex-col items-center gap-2 group">
                <span className="text-[10px] font-mono font-bold text-[#A1A6B2] group-hover:text-[#6C8CFF] transition-colors">
                  {day.count}
                </span>
                <div className="w-full bg-[#20232B] h-32 rounded-t-md overflow-hidden flex items-end">
                  <div
                    className="w-full bg-[#6C8CFF] group-hover:bg-[#5876E8] rounded-t-md transition-all duration-300"
                    style={{ height: `${heightPct}%` }}
                  />
                </div>
                <span className="text-[10px] text-[#A1A6B2] font-mono">
                  {day.date.slice(5)}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Folder Activity Breakdown & Top Modified Files */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
        {/* Folder Distribution */}
        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-3">
          <div className="flex items-center space-x-2">
            <FolderOpen className="w-4 h-4 text-[#6C8CFF]" />
            <h4 className="text-sm font-bold text-[#F2F3F5]">Folder Distribution</h4>
          </div>

          <div className="space-y-2.5">
            {stats?.folder_breakdown.map((f) => (
              <div key={f.folder_id} className="p-3 bg-[#20232B] rounded-lg space-y-1.5">
                <div className="flex items-center justify-between text-xs">
                  <span className="font-semibold text-[#F2F3F5]">{f.folder_name}</span>
                  <span className="font-mono text-[#6C8CFF]">{f.change_count} changes</span>
                </div>
                <div className="w-full bg-[#181A20] h-1.5 rounded-full overflow-hidden">
                  <div
                    className="bg-[#6C8CFF] h-full rounded-full"
                    style={{ width: `${Math.min(f.change_count * 2, 100)}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Top Modified Files */}
        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-3">
          <div className="flex items-center space-x-2">
            <FileText className="w-4 h-4 text-[#E9B65B]" />
            <h4 className="text-sm font-bold text-[#F2F3F5]">Most Active Files</h4>
          </div>

          <div className="space-y-2">
            {stats?.top_modified_files.length === 0 ? (
              <div className="p-6 text-center text-xs text-[#A1A6B2]">
                No files with multiple revisions yet.
              </div>
            ) : (
              stats?.top_modified_files.map((file) => (
                <div
                  key={file.file_id}
                  className="p-2.5 bg-[#20232B] rounded-lg flex items-center justify-between text-xs font-mono"
                >
                  <span className="text-[#F2F3F5] truncate max-w-[260px]">
                    {file.relative_path}
                  </span>
                  <span className="px-2 py-0.5 rounded bg-[#E9B65B]/20 text-[#E9B65B] font-bold text-[10px]">
                    {file.version_count} versions
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
