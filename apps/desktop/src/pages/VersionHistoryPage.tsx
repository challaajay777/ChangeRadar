import React, { useState, useEffect } from 'react';
import {
  History,
  Pin,
  RotateCcw,
  GitCompare,
  FileText,
  Calendar,
  HardDrive,
  Hash,
  Download,
} from 'lucide-react';
import { FileRecord, FileVersion } from '../types';
import { api } from '../services/api';

interface VersionHistoryPageProps {
  files: FileRecord[];
  selectedFileId: string | null;
  onSelectFile: (fileId: string) => void;
  onRestoreVersion: (versionId: string) => void;
  onCompareVersions: (verA: string, verB: string) => void;
}

export const VersionHistoryPage: React.FC<VersionHistoryPageProps> = ({
  files,
  selectedFileId,
  onSelectFile,
  onRestoreVersion,
  onCompareVersions,
}) => {
  const [versions, setVersions] = useState<FileVersion[]>([]);
  const [selectedVerIds, setSelectedVerIds] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  const activeFile = files.find((f) => f.id === selectedFileId) || files[0];

  useEffect(() => {
    if (activeFile) {
      setLoading(true);
      api
        .listFileVersions(activeFile.id)
        .then((vers) => {
          setVersions(vers);
          setSelectedVerIds(vers.slice(0, 2).map((v) => v.id));
        })
        .finally(() => setLoading(false));
    }
  }, [activeFile?.id]);

  const toggleSelectForCompare = (verId: string) => {
    if (selectedVerIds.includes(verId)) {
      setSelectedVerIds(selectedVerIds.filter((id) => id !== verId));
    } else {
      if (selectedVerIds.length >= 2) {
        setSelectedVerIds([selectedVerIds[1], verId]);
      } else {
        setSelectedVerIds([...selectedVerIds, verId]);
      }
    }
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
      {/* File Selector Sidebar */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 space-y-3">
        <h3 className="text-xs font-bold text-[#A1A6B2] uppercase tracking-wider">
          Tracked Files ({files.length})
        </h3>
        <div className="space-y-1 max-h-[600px] overflow-y-auto">
          {files.map((file) => {
            const isSelected = activeFile?.id === file.id;
            return (
              <button
                key={file.id}
                onClick={() => onSelectFile(file.id)}
                className={`w-full text-left p-2.5 rounded-lg text-xs font-mono transition-colors flex items-center justify-between ${
                  isSelected
                    ? 'bg-[#6C8CFF]/15 text-[#6C8CFF] font-semibold border border-[#6C8CFF]/30'
                    : 'text-[#A1A6B2] hover:bg-[#20232B] hover:text-[#F2F3F5]'
                }`}
              >
                <span className="truncate max-w-[200px]">{file.relative_path}</span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#20232B] text-[#A1A6B2]">
                  {file.extension}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Version Timeline Panel */}
      <div className="md:col-span-2 space-y-4">
        {/* Active File Header */}
        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 flex items-center justify-between">
          <div>
            <h3 className="text-sm font-bold text-[#F2F3F5] font-mono flex items-center gap-2">
              <FileText className="w-4 h-4 text-[#6C8CFF]" />
              {activeFile?.relative_path || 'No File Selected'}
            </h3>
            <p className="text-xs text-[#A1A6B2] mt-0.5">
              {versions.length} historical revisions preserved in Content-Addressed Storage
            </p>
          </div>

          {selectedVerIds.length === 2 && (
            <button
              onClick={() => onCompareVersions(selectedVerIds[1], selectedVerIds[0])}
              className="flex items-center space-x-1.5 px-3 py-1.5 bg-[#6C8CFF] hover:bg-[#5876E8] text-white rounded-lg text-xs font-semibold shadow-sm transition-colors"
            >
              <GitCompare className="w-3.5 h-3.5" />
              <span>Compare 2 Selected Versions</span>
            </button>
          )}
        </div>

        {/* Timeline Items */}
        <div className="space-y-3">
          {loading ? (
            <div className="p-12 text-center text-xs text-[#A1A6B2]">
              Loading historical revisions...
            </div>
          ) : versions.length === 0 ? (
            <div className="p-8 text-center text-xs text-[#A1A6B2] bg-[#181A20] border border-[#2C303A] rounded-xl">
              No historical versions found for this file.
            </div>
          ) : (
            versions.map((ver, idx) => {
              const isChecked = selectedVerIds.includes(ver.id);
              return (
                <div
                  key={ver.id}
                  className={`bg-[#181A20] border rounded-xl p-4 transition-all ${
                    isChecked
                      ? 'border-[#6C8CFF] bg-[#20232B]/50 shadow-md'
                      : 'border-[#2C303A] hover:border-[#3E4351]'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start space-x-3">
                      <input
                        type="checkbox"
                        checked={isChecked}
                        onChange={() => toggleSelectForCompare(ver.id)}
                        className="mt-1 rounded bg-[#20232B] border-[#2C303A] text-[#6C8CFF] focus:ring-0"
                      />

                      <div className="space-y-1">
                        <div className="flex items-center space-x-2">
                          <span className="text-sm font-bold text-[#F2F3F5]">
                            Revision #{ver.version_number}
                          </span>
                          <span
                            className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                              ver.change_kind === 'CREATED'
                                ? 'bg-[#42C98A]/20 text-[#42C98A]'
                                : ver.change_kind === 'RESTORED'
                                ? 'bg-[#6C8CFF]/20 text-[#6C8CFF]'
                                : 'bg-[#E9B65B]/20 text-[#E9B65B]'
                            }`}
                          >
                            {ver.change_kind}
                          </span>
                          {ver.is_pinned && (
                            <span className="px-2 py-0.5 rounded bg-[#42C98A]/20 text-[#42C98A] text-[10px] font-bold flex items-center gap-1">
                              <Pin className="w-3 h-3" /> PINNED
                            </span>
                          )}
                        </div>

                        <div className="text-xs text-[#A1A6B2]">
                          {ver.notes || 'Automatic change capture'}
                        </div>

                        <div className="flex flex-wrap items-center gap-3 text-[11px] text-[#A1A6B2] pt-1 font-mono">
                          <span className="flex items-center gap-1">
                            <Calendar className="w-3.5 h-3.5 text-[#6C8CFF]" />
                            {new Date(ver.created_at).toLocaleString()}
                          </span>
                          <span className="flex items-center gap-1">
                            <HardDrive className="w-3.5 h-3.5 text-[#E9B65B]" />
                            {ver.file_size} bytes
                          </span>
                          <span className="flex items-center gap-1">
                            <Hash className="w-3.5 h-3.5 text-[#42C98A]" />
                            {ver.content_hash.slice(0, 16)}...
                          </span>
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center space-x-2">
                      <button
                        onClick={() => onRestoreVersion(ver.id)}
                        className="flex items-center space-x-1 px-3 py-1.5 bg-[#20232B] hover:bg-[#6C8CFF] text-[#F2F3F5] hover:text-white rounded-lg border border-[#2C303A] text-xs font-semibold transition-colors"
                      >
                        <RotateCcw className="w-3.5 h-3.5" />
                        <span>Restore</span>
                      </button>
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
