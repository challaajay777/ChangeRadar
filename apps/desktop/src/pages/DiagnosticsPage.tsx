import React, { useState, useEffect } from 'react';
import {
  Stethoscope,
  CheckCircle2,
  AlertTriangle,
  HardDrive,
  Database,
  Cpu,
  RefreshCw,
  Download,
  ShieldCheck,
} from 'lucide-react';
import { DoctorReport } from '../types';
import { api } from '../services/api';

export const DiagnosticsPage: React.FC = () => {
  const [report, setReport] = useState<DoctorReport | null>(null);
  const [loading, setLoading] = useState(false);

  const runDoctor = () => {
    setLoading(true);
    api
      .runDoctor()
      .then((rep) => setReport(rep))
      .catch((e) => console.warn('Doctor run failed:', e))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    runDoctor();
  }, []);

  const exportReport = () => {
    if (!report) return;
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(report, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', `changeradar-diagnostics-${new Date().toISOString().slice(0, 10)}.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 flex items-center justify-between">
        <div>
          <h3 className="text-base font-bold text-[#F2F3F5] flex items-center gap-2">
            <Stethoscope className="w-5 h-5 text-[#6C8CFF]" />
            <span>ChangeRadar Doctor & System Diagnostics</span>
          </h3>
          <p className="text-xs text-[#A1A6B2]">
            Comprehensive self-test of SQLite metadata integrity, Content-Addressed Storage (CAS) checksums, and watcher threads.
          </p>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={runDoctor}
            disabled={loading}
            className="flex items-center space-x-1.5 px-3 py-1.5 bg-[#20232B] hover:bg-[#252933] border border-[#2C303A] text-xs font-semibold text-[#F2F3F5] rounded-lg transition-colors"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
            <span>{loading ? 'Running Tests...' : 'Re-Run Diagnostics'}</span>
          </button>

          <button
            onClick={exportReport}
            disabled={!report}
            className="flex items-center space-x-1.5 px-3 py-1.5 bg-[#6C8CFF] hover:bg-[#5876E8] text-white text-xs font-semibold rounded-lg shadow-sm transition-colors"
          >
            <Download className="w-3.5 h-3.5" />
            <span>Export Diagnostic Report</span>
          </button>
        </div>
      </div>

      {report && (
        <div className="space-y-4">
          {/* Status Alert */}
          <div
            className={`p-4 rounded-xl border flex items-center justify-between ${
              report.all_healthy
                ? 'bg-[#42C98A]/10 border-[#42C98A]/30 text-[#42C98A]'
                : 'bg-[#EA6868]/10 border-[#EA6868]/30 text-[#EA6868]'
            }`}
          >
            <div className="flex items-center space-x-3">
              {report.all_healthy ? (
                <CheckCircle2 className="w-6 h-6 flex-shrink-0" />
              ) : (
                <AlertTriangle className="w-6 h-6 flex-shrink-0" />
              )}
              <div>
                <h4 className="text-sm font-bold">
                  {report.all_healthy ? 'All Systems Fully Operational' : 'Attention Needed'}
                </h4>
                <p className="text-xs text-[#F2F3F5]/80">
                  {report.all_healthy
                    ? 'SQLite database integrity check passed and all CAS content objects verified with cryptographic hashes.'
                    : 'Discrepancies detected during diagnostic run. Review items below.'}
                </p>
              </div>
            </div>

            <span className="text-xs font-mono font-bold px-2.5 py-1 rounded bg-[#20232B]">
              Platform: {report.platform}
            </span>
          </div>

          {/* Diagnostic Cards */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Database Check */}
            <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <Database className="w-4 h-4 text-[#6C8CFF]" />
                  <h4 className="text-sm font-bold text-[#F2F3F5]">SQLite Metadata Engine</h4>
                </div>
                <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-[#42C98A]/20 text-[#42C98A] font-bold">
                  PRAGMA: {report.database_integrity.toUpperCase()}
                </span>
              </div>

              <div className="space-y-1.5 text-xs text-[#A1A6B2]">
                <div className="flex justify-between">
                  <span>Database Path:</span>
                  <span className="text-[#F2F3F5] font-mono truncate max-w-[240px]">
                    {report.database_path}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>File Size:</span>
                  <span className="text-[#F2F3F5] font-mono">
                    {(report.storage_stats.database_bytes / 1024).toFixed(1)} KB
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Tracked Files:</span>
                  <span className="text-[#F2F3F5] font-mono">
                    {report.total_tracked_files.toLocaleString()}
                  </span>
                </div>
              </div>
            </div>

            {/* CAS Storage Check */}
            <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <HardDrive className="w-4 h-4 text-[#E9B65B]" />
                  <h4 className="text-sm font-bold text-[#F2F3F5]">Content-Addressed Storage</h4>
                </div>
                <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-[#42C98A]/20 text-[#42C98A] font-bold">
                  {report.cas_integrity.valid_count} Objects Verified
                </span>
              </div>

              <div className="space-y-1.5 text-xs text-[#A1A6B2]">
                <div className="flex justify-between">
                  <span>Total CAS Objects:</span>
                  <span className="text-[#F2F3F5] font-mono">
                    {report.cas_integrity.total_checked}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Corrupted Objects:</span>
                  <span className={`font-mono ${report.cas_integrity.corrupted_count === 0 ? 'text-[#42C98A]' : 'text-[#EA6868]'}`}>
                    {report.cas_integrity.corrupted_count}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Deduplication Ratio:</span>
                  <span className="text-[#42C98A] font-mono font-bold">
                    {report.storage_stats.deduplication_ratio}x
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
