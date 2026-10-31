import React, { useState } from 'react';
import {
  RotateCcw,
  ShieldCheck,
  AlertTriangle,
  CheckCircle2,
  FileCheck,
  FolderOpen,
  ArrowRight,
  HardDrive,
} from 'lucide-react';
import { FileRecord, FileVersion, RecoveryPreflight, RecoveryResult } from '../types';
import { api } from '../services/api';

interface RecoveryCenterPageProps {
  files: FileRecord[];
  initialVersionId?: string;
}

export const RecoveryCenterPage: React.FC<RecoveryCenterPageProps> = ({
  files,
  initialVersionId,
}) => {
  const [selectedFileId, setSelectedFileId] = useState<string>(files[0]?.id || '');
  const [versions, setVersions] = useState<FileVersion[]>([]);
  const [selectedVersionId, setSelectedVersionId] = useState<string>(initialVersionId || '');
  const [customDestination, setCustomDestination] = useState<string>('');
  const [preflight, setPreflight] = useState<RecoveryPreflight | null>(null);
  const [recoveryResult, setRecoveryResult] = useState<RecoveryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [restoring, setRestoring] = useState(false);

  // Load versions when file changes
  React.useEffect(() => {
    if (selectedFileId) {
      setLoading(true);
      api
        .listFileVersions(selectedFileId)
        .then((vers) => {
          setVersions(vers);
          if (vers.length > 0) {
            setSelectedVersionId(vers[0].id);
          }
        })
        .finally(() => setLoading(false));
    }
  }, [selectedFileId]);

  // Run pre-flight check when selected version changes
  const runPreflight = async (verId: string, dest?: string) => {
    setLoading(true);
    try {
      const res = await api.recoveryPreflight(verId, dest || undefined);
      setPreflight(res);
      setRecoveryResult(null);
    } catch (e) {
      alert(`Pre-flight check failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  React.useEffect(() => {
    if (selectedVersionId) {
      runPreflight(selectedVersionId, customDestination);
    }
  }, [selectedVersionId, customDestination]);

  const executeRestore = async () => {
    if (!selectedVersionId) return;
    setRestoring(true);
    try {
      const res = await api.recoveryRestore(
        selectedVersionId,
        customDestination.trim() !== '' ? customDestination : undefined
      );
      setRecoveryResult(res);
      // Re-run preflight
      runPreflight(selectedVersionId, customDestination);
    } catch (e) {
      alert(`Recovery failed: ${e}`);
    } finally {
      setRestoring(false);
    }
  };

  const activeFile = files.find((f) => f.id === selectedFileId);

  return (
    <div className="space-y-6">
      {/* Top Banner */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 flex items-center justify-between">
        <div className="space-y-1">
          <div className="flex items-center space-x-2">
            <ShieldCheck className="w-5 h-5 text-[#42C98A]" />
            <h3 className="text-base font-bold text-[#F2F3F5]">Safe & Reversible Recovery Center</h3>
          </div>
          <p className="text-xs text-[#A1A6B2] max-w-xl">
            ChangeRadar performs pre-flight verification, validates Content-Addressed Storage checksums, preserves current destination files into history prior to overwrite, and writes atomically.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
        {/* Step 1 & 2: Selection Panel */}
        <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 space-y-4">
          <h4 className="text-xs font-bold text-[#A1A6B2] uppercase tracking-wider">
            1. Select File & Version
          </h4>

          {/* File Dropdown */}
          <div className="space-y-1">
            <label className="text-xs text-[#A1A6B2]">Target File:</label>
            <select
              value={selectedFileId}
              onChange={(e) => setSelectedFileId(e.target.value)}
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2 text-xs text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            >
              {files.map((f) => (
                <option key={f.id} value={f.id}>
                  {f.relative_path} {f.is_deleted ? '(DELETED)' : ''}
                </option>
              ))}
            </select>
          </div>

          {/* Version Selection */}
          <div className="space-y-1">
            <label className="text-xs text-[#A1A6B2]">Historical Revision to Restore:</label>
            <div className="space-y-1.5 max-h-60 overflow-y-auto">
              {versions.map((v) => (
                <button
                  key={v.id}
                  onClick={() => setSelectedVersionId(v.id)}
                  className={`w-full text-left p-2.5 rounded-lg text-xs font-mono transition-colors flex items-center justify-between ${
                    selectedVersionId === v.id
                      ? 'bg-[#6C8CFF]/15 text-[#6C8CFF] font-semibold border border-[#6C8CFF]/30'
                      : 'text-[#A1A6B2] hover:bg-[#20232B] hover:text-[#F2F3F5]'
                  }`}
                >
                  <div>
                    <div className="font-bold">Revision #{v.version_number}</div>
                    <div className="text-[10px] text-[#A1A6B2]">
                      {new Date(v.created_at).toLocaleString()}
                    </div>
                  </div>
                  <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#20232B]">
                    {v.file_size} B
                  </span>
                </button>
              ))}
            </div>
          </div>

          {/* Optional Alternate Destination */}
          <div className="space-y-1 pt-2 border-t border-[#2C303A]">
            <label className="text-xs text-[#A1A6B2]">
              Alternate Destination (Optional):
            </label>
            <input
              type="text"
              value={customDestination}
              onChange={(e) => setCustomDestination(e.target.value)}
              placeholder="Leave empty for original path"
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2 text-xs text-[#F2F3F5] placeholder-[#A1A6B2] focus:outline-none focus:border-[#6C8CFF]"
            />
          </div>
        </div>

        {/* Step 3: Pre-flight Verification & Restoration Execution */}
        <div className="md:col-span-2 space-y-4">
          <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-4">
            <h4 className="text-xs font-bold text-[#A1A6B2] uppercase tracking-wider flex items-center gap-1.5">
              <ShieldCheck className="w-4 h-4 text-[#42C98A]" />
              2. Pre-flight Safety Inspection
            </h4>

            {preflight ? (
              <div className="space-y-3">
                <div className="grid grid-cols-2 gap-3 text-xs">
                  <div className="p-3 bg-[#20232B] rounded-lg space-y-1">
                    <span className="text-[#A1A6B2]">CAS Content Object:</span>
                    <div className="flex items-center space-x-1.5">
                      {preflight.cas_object_verified ? (
                        <>
                          <CheckCircle2 className="w-4 h-4 text-[#42C98A]" />
                          <span className="font-semibold text-[#42C98A]">
                            Verified & Integrity Valid
                          </span>
                        </>
                      ) : (
                        <>
                          <AlertTriangle className="w-4 h-4 text-[#EA6868]" />
                          <span className="font-semibold text-[#EA6868]">Object Corrupted</span>
                        </>
                      )}
                    </div>
                  </div>

                  <div className="p-3 bg-[#20232B] rounded-lg space-y-1">
                    <span className="text-[#A1A6B2]">Destination Conflict:</span>
                    <div className="flex items-center space-x-1.5">
                      {preflight.destination_exists ? (
                        <>
                          <AlertTriangle className="w-4 h-4 text-[#E9B65B]" />
                          <span className="font-semibold text-[#E9B65B]">
                            File Exists (Will be Backed Up)
                          </span>
                        </>
                      ) : (
                        <>
                          <CheckCircle2 className="w-4 h-4 text-[#42C98A]" />
                          <span className="font-semibold text-[#42C98A]">
                            No Conflict (Clean Write)
                          </span>
                        </>
                      )}
                    </div>
                  </div>
                </div>

                <div className="p-3 bg-[#20232B] rounded-lg space-y-1 text-xs font-mono">
                  <div className="text-[#A1A6B2]">Target Path:</div>
                  <div className="text-[#F2F3F5] break-all">{preflight.target_destination}</div>
                </div>

                {preflight.warning && (
                  <div className="p-3 bg-[#E9B65B]/10 border border-[#E9B65B]/30 rounded-lg flex items-start space-x-2 text-xs text-[#E9B65B]">
                    <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5" />
                    <span>{preflight.warning}</span>
                  </div>
                )}

                {/* Restoration Trigger Button */}
                <div className="pt-2">
                  <button
                    onClick={executeRestore}
                    disabled={!preflight.can_proceed || restoring}
                    className="w-full flex items-center justify-center space-x-2 py-2.5 bg-[#42C98A] hover:bg-[#36B278] disabled:opacity-50 text-white rounded-lg text-sm font-bold shadow-lg shadow-[#42C98A]/20 transition-all"
                  >
                    <RotateCcw className="w-4 h-4" />
                    <span>
                      {restoring ? 'Restoring and Verifying Hash...' : 'Restore File Version Now'}
                    </span>
                  </button>
                </div>
              </div>
            ) : (
              <div className="p-8 text-center text-xs text-[#A1A6B2]">
                Select a file and revision to begin safety inspection.
              </div>
            )}
          </div>

          {/* Recovery Confirmation Card */}
          {recoveryResult && (
            <div className="bg-[#42C98A]/10 border border-[#42C98A]/30 rounded-xl p-5 space-y-2 animate-in fade-in">
              <div className="flex items-center space-x-2 text-[#42C98A] font-bold text-sm">
                <CheckCircle2 className="w-5 h-5" />
                <span>Restoration Completed Successfully</span>
              </div>
              <p className="text-xs text-[#F2F3F5]">{recoveryResult.message}</p>
              <div className="text-[11px] text-[#A1A6B2] font-mono pt-1 space-y-0.5">
                <div>Destination: {recoveryResult.destination_path}</div>
                <div>Verified Restored Hash: {recoveryResult.restored_hash}</div>
                {recoveryResult.backup_version_id && (
                  <div className="text-[#42C98A]">
                    Pre-restore backup saved: Revision ID {recoveryResult.backup_version_id}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
