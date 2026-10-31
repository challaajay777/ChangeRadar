import React, { useState, useEffect } from 'react';
import {
  HardDrive,
  Database,
  Pin,
  CheckCircle2,
  Trash2,
  ShieldCheck,
  AlertTriangle,
  RotateCcw,
} from 'lucide-react';
import { StorageStats, CleanupPreview } from '../types';
import { api } from '../services/api';

export const StoragePage: React.FC = () => {
  const [stats, setStats] = useState<StorageStats | null>(null);
  const [maxVersions, setMaxVersions] = useState(50);
  const [maxAgeDays, setMaxAgeDays] = useState(90);
  const [preview, setPreview] = useState<CleanupPreview | null>(null);
  const [loading, setLoading] = useState(false);
  const [cleaning, setCleaning] = useState(false);

  const loadStats = () => {
    setLoading(true);
    api
      .getStorageStats()
      .then((s) => setStats(s))
      .catch((e) => console.warn('Storage stats error:', e))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    loadStats();
  }, []);

  const handlePreview = async () => {
    setLoading(true);
    try {
      const p = await api.previewCleanup(maxVersions, maxAgeDays);
      setPreview(p);
    } catch (e) {
      alert(`Preview failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">CAS Objects</span>
            <HardDrive className="w-4 h-4 text-[#6C8CFF]" />
          </div>
          <div className="text-2xl font-bold text-[#F2F3F5]">
            {stats ? stats.total_objects.toLocaleString() : '...'}
          </div>
          <div className="text-[11px] text-[#A1A6B2]">Deduplicated content objects</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Compressed Disk Size</span>
            <HardDrive className="w-4 h-4 text-[#E9B65B]" />
          </div>
          <div className="text-2xl font-bold text-[#E9B65B]">
            {stats ? (stats.compressed_bytes / (1024 * 1024)).toFixed(1) : '...'} MB
          </div>
          <div className="text-[11px] text-[#A1A6B2]">Zstandard compressed on disk</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Deduplication Ratio</span>
            <CheckCircle2 className="w-4 h-4 text-[#42C98A]" />
          </div>
          <div className="text-2xl font-bold text-[#42C98A]">
            {stats ? stats.deduplication_ratio.toFixed(2) : '...'}x
          </div>
          <div className="text-[11px] text-[#A1A6B2]">Raw size vs compressed size</div>
        </div>

        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 space-y-1">
          <div className="flex items-center justify-between text-[#A1A6B2]">
            <span className="text-xs">Pinned Versions</span>
            <Pin className="w-4 h-4 text-[#6C8CFF]" />
          </div>
          <div className="text-2xl font-bold text-[#F2F3F5]">
            {stats ? stats.pinned_versions : '...'}
          </div>
          <div className="text-[11px] text-[#A1A6B2]">Permanently protected</div>
        </div>
      </div>

      {/* Retention Policy & Cleanup Preview */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-5">
        <div>
          <h3 className="text-sm font-bold text-[#F2F3F5]">Retention & Storage Cleanup</h3>
          <p className="text-xs text-[#A1A6B2]">
            Configure retention rules to prune older unpinned versions. ChangeRadar always generates a preview before deleting anything, and pinned versions are permanently exempt from pruning.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-[#A1A6B2]">
              Maximum Revisions Per File:
            </label>
            <input
              type="number"
              min="5"
              max="500"
              value={maxVersions}
              onChange={(e) => setMaxVersions(Number(e.target.value))}
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2 text-xs text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            />
            <span className="text-[10px] text-[#A1A6B2]">
              Keep up to {maxVersions} newest revisions per tracked file.
            </span>
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-[#A1A6B2]">
              Maximum Version Age (Days):
            </label>
            <input
              type="number"
              min="7"
              max="365"
              value={maxAgeDays}
              onChange={(e) => setMaxAgeDays(Number(e.target.value))}
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2 text-xs text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            />
            <span className="text-[10px] text-[#A1A6B2]">
              Prune unpinned revisions older than {maxAgeDays} days.
            </span>
          </div>
        </div>

        <button
          onClick={handlePreview}
          disabled={loading}
          className="px-4 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white text-xs font-semibold rounded-lg shadow-sm transition-colors"
        >
          {loading ? 'Evaluating Policy...' : 'Generate Cleanup Preview'}
        </button>

        {/* Cleanup Preview Results */}
        {preview && (
          <div className="p-4 bg-[#20232B] border border-[#2C303A] rounded-xl space-y-3 animate-in fade-in">
            <div className="flex items-center space-x-2 text-xs font-bold text-[#F2F3F5]">
              <ShieldCheck className="w-4 h-4 text-[#42C98A]" />
              <span>Retention Cleanup Preview Result</span>
            </div>

            <div className="grid grid-cols-3 gap-3 text-xs">
              <div className="p-2.5 bg-[#181A20] rounded border border-[#2C303A]">
                <div className="text-[#A1A6B2]">Candidate Versions:</div>
                <div className="text-base font-bold text-[#EA6868]">
                  {preview.candidate_versions_count}
                </div>
              </div>

              <div className="p-2.5 bg-[#181A20] rounded border border-[#2C303A]">
                <div className="text-[#A1A6B2]">Estimated Space Reclaimed:</div>
                <div className="text-base font-bold text-[#42C98A]">
                  {(preview.estimated_reclaimed_bytes / (1024 * 1024)).toFixed(1)} MB
                </div>
              </div>

              <div className="p-2.5 bg-[#181A20] rounded border border-[#2C303A]">
                <div className="text-[#A1A6B2]">Files Affected:</div>
                <div className="text-base font-bold text-[#F2F3F5]">
                  {preview.affected_files_count}
                </div>
              </div>
            </div>

            <p className="text-[11px] text-[#A1A6B2]">
              Pinned revisions and all currently referenced content objects will remain untouched.
            </p>
          </div>
        )}
      </div>
    </div>
  );
};
