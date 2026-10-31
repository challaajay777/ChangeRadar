import React, { useState, useEffect } from 'react';
import {
  GitCompare,
  Columns,
  AlignLeft,
  FileCode,
  Table,
  Image as ImageIcon,
  CheckCircle2,
  AlertCircle,
} from 'lucide-react';
import { ComparisonOutput } from '../types';
import { api } from '../services/api';

interface ComparePageProps {
  initialVersionA?: string;
  initialVersionB?: string;
}

export const ComparePage: React.FC<ComparePageProps> = ({
  initialVersionA,
  initialVersionB,
}) => {
  const [versionA, setVersionA] = useState(initialVersionA || 'v-1');
  const [versionB, setVersionB] = useState(initialVersionB || 'v-2');
  const [diffMode, setDiffMode] = useState<'side-by-side' | 'unified'>('side-by-side');
  const [comparison, setComparison] = useState<ComparisonOutput | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (versionA && versionB) {
      setLoading(true);
      api
        .compareVersions(versionA, versionB)
        .then((res) => setComparison(res))
        .catch((e) => console.warn('Compare error:', e))
        .finally(() => setLoading(false));
    }
  }, [versionA, versionB]);

  return (
    <div className="space-y-4">
      {/* Control Header */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center space-x-3">
          <GitCompare className="w-5 h-5 text-[#6C8CFF]" />
          <div>
            <h3 className="text-sm font-bold text-[#F2F3F5]">Universal File Comparison</h3>
            <p className="text-xs text-[#A1A6B2]">
              {comparison ? comparison.summary : 'Select two historical versions to compute semantic difference.'}
            </p>
          </div>
        </div>

        {/* View Mode Toggle */}
        <div className="flex items-center space-x-2 bg-[#20232B] p-1 rounded-lg border border-[#2C303A]">
          <button
            onClick={() => setDiffMode('side-by-side')}
            className={`flex items-center space-x-1.5 px-2.5 py-1 rounded text-xs font-semibold transition-colors ${
              diffMode === 'side-by-side'
                ? 'bg-[#6C8CFF] text-white shadow-sm'
                : 'text-[#A1A6B2] hover:text-[#F2F3F5]'
            }`}
          >
            <Columns className="w-3.5 h-3.5" />
            <span>Side-by-Side</span>
          </button>
          <button
            onClick={() => setDiffMode('unified')}
            className={`flex items-center space-x-1.5 px-2.5 py-1 rounded text-xs font-semibold transition-colors ${
              diffMode === 'unified'
                ? 'bg-[#6C8CFF] text-white shadow-sm'
                : 'text-[#A1A6B2] hover:text-[#F2F3F5]'
            }`}
          >
            <AlignLeft className="w-3.5 h-3.5" />
            <span>Unified</span>
          </button>
        </div>
      </div>

      {/* Comparison Statistics Bar */}
      {comparison && (
        <div className="grid grid-cols-4 gap-3">
          <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3">
            <span className="text-[11px] text-[#A1A6B2]">Engine</span>
            <div className="text-sm font-bold text-[#6C8CFF]">{comparison.comparator_type}</div>
          </div>
          <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3">
            <span className="text-[11px] text-[#A1A6B2]">Additions</span>
            <div className="text-sm font-bold text-[#42C98A]">+{comparison.added_count}</div>
          </div>
          <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3">
            <span className="text-[11px] text-[#A1A6B2]">Deletions</span>
            <div className="text-sm font-bold text-[#EA6868]">-{comparison.removed_count}</div>
          </div>
          <div className="bg-[#181A20] border border-[#2C303A] rounded-lg p-3">
            <span className="text-[11px] text-[#A1A6B2]">Modifications</span>
            <div className="text-sm font-bold text-[#E9B65B]">{comparison.modified_count}</div>
          </div>
        </div>
      )}

      {/* Diff View Area */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl overflow-hidden min-h-[400px]">
        {loading ? (
          <div className="p-16 text-center text-xs text-[#A1A6B2]">
            Computing universal file diff...
          </div>
        ) : !comparison ? (
          <div className="p-16 text-center text-xs text-[#A1A6B2]">
            Select two versions from Version History to compare.
          </div>
        ) : comparison.text_diff ? (
          diffMode === 'side-by-side' ? (
            /* Side-by-side Text Diff */
            <div className="overflow-x-auto font-mono text-xs">
              <div className="grid grid-cols-2 bg-[#20232B] border-b border-[#2C303A] p-2 text-[11px] font-bold text-[#A1A6B2]">
                <div>Original Revision (Old)</div>
                <div>Modified Revision (New)</div>
              </div>
              <div className="divide-y divide-[#2C303A]/40">
                {comparison.text_diff.side_by_side.map((row, idx) => (
                  <div key={idx} className="grid grid-cols-2 hover:bg-[#20232B]/30">
                    <div
                      className={`flex p-1 border-r border-[#2C303A] ${
                        row.change_type === 'DELETE' ? 'diff-del' : ''
                      }`}
                    >
                      <span className="w-8 text-[#A1A6B2] text-right pr-2 select-none">
                        {row.line_num_a || ''}
                      </span>
                      <span className="flex-1 whitespace-pre-wrap">{row.line_text_a || ''}</span>
                    </div>

                    <div
                      className={`flex p-1 ${
                        row.change_type === 'INSERT' ? 'diff-ins' : ''
                      }`}
                    >
                      <span className="w-8 text-[#A1A6B2] text-right pr-2 select-none">
                        {row.line_num_b || ''}
                      </span>
                      <span className="flex-1 whitespace-pre-wrap">{row.line_text_b || ''}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ) : (
            /* Unified Text Diff */
            <div className="p-4 font-mono text-xs overflow-x-auto whitespace-pre">
              {comparison.text_diff.unified_diff}
            </div>
          )
        ) : comparison.json_diff ? (
          /* Structural JSON Diff */
          <div className="p-4 space-y-2">
            <h4 className="text-xs font-bold text-[#A1A6B2] uppercase">JSON Structural Shifts</h4>
            <div className="space-y-1 font-mono text-xs">
              {comparison.json_diff.map((item, idx) => (
                <div
                  key={idx}
                  className={`p-2 rounded border ${
                    item.kind === 'ADDED'
                      ? 'bg-[#42C98A]/10 border-[#42C98A]/30 text-[#42C98A]'
                      : item.kind === 'REMOVED'
                      ? 'bg-[#EA6868]/10 border-[#EA6868]/30 text-[#EA6868]'
                      : 'bg-[#E9B65B]/10 border-[#E9B65B]/30 text-[#E9B65B]'
                  }`}
                >
                  <span className="font-bold">[{item.kind}]</span> {item.path}
                  {item.old_val && <div className="text-[11px] text-[#EA6868]">OLD: {item.old_val}</div>}
                  {item.new_val && <div className="text-[11px] text-[#42C98A]">NEW: {item.new_val}</div>}
                </div>
              ))}
            </div>
          </div>
        ) : comparison.csv_diff ? (
          /* CSV Table Diff */
          <div className="p-4 space-y-3">
            <h4 className="text-xs font-bold text-[#A1A6B2] uppercase">CSV Record Diff</h4>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs">
                <thead className="bg-[#20232B] text-[#A1A6B2]">
                  <tr>
                    <th className="p-2">Row</th>
                    <th className="p-2">Status</th>
                    {comparison.csv_diff.headers.map((h, i) => (
                      <th key={i} className="p-2">{h}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {comparison.csv_diff.sample_changes.map((r, i) => (
                    <tr key={i} className={r.change_kind === 'ADDED' ? 'diff-ins' : 'diff-del'}>
                      <td className="p-2 font-mono">#{r.row_index}</td>
                      <td className="p-2 font-bold">{r.change_kind}</td>
                      {r.row_data.map((c, j) => (
                        <td key={j} className="p-2 font-mono">{c}</td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ) : comparison.image_diff ? (
          /* Image Diff */
          <div className="p-6 text-center space-y-4">
            <ImageIcon className="w-12 h-12 text-[#6C8CFF] mx-auto" />
            <h4 className="text-base font-bold text-[#F2F3F5]">Image Comparison Analysis</h4>
            <div className="max-w-md mx-auto bg-[#20232B] p-4 rounded-xl space-y-2 text-xs">
              <div className="flex justify-between">
                <span className="text-[#A1A6B2]">Dimensions Old:</span>
                <span className="font-mono">{comparison.image_diff.old_width}x{comparison.image_diff.old_height}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#A1A6B2]">Dimensions New:</span>
                <span className="font-mono">{comparison.image_diff.new_width}x{comparison.image_diff.new_height}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#A1A6B2]">Perceptual Similarity:</span>
                <span className="font-mono font-bold text-[#42C98A]">{comparison.image_diff.perceptual_similarity_pct}%</span>
              </div>
            </div>
          </div>
        ) : (
          <div className="p-8 text-center text-xs text-[#A1A6B2]">
            Binary comparison: file content hashes differ.
          </div>
        )}
      </div>
    </div>
  );
};
