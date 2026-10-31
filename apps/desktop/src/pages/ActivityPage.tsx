import React, { useState } from 'react';
import {
  Activity,
  Search,
  Filter,
  Download,
  Clock,
  ArrowRight,
  ExternalLink,
} from 'lucide-react';
import { FileEvent, MonitoredFolder, ChangeKind } from '../types';

interface ActivityPageProps {
  events: FileEvent[];
  folders: MonitoredFolder[];
  onOpenCompare?: (fileId?: string) => void;
}

export const ActivityPage: React.FC<ActivityPageProps> = ({
  events,
  folders,
  onOpenCompare,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedFolder, setSelectedFolder] = useState<string>('ALL');
  const [selectedKind, setSelectedKind] = useState<string>('ALL');

  const filteredEvents = events.filter((ev) => {
    if (selectedFolder !== 'ALL' && ev.folder_id !== selectedFolder) {
      return false;
    }
    if (selectedKind !== 'ALL' && ev.event_type !== selectedKind) {
      return false;
    }
    if (searchTerm.trim() !== '') {
      const q = searchTerm.toLowerCase();
      return (
        ev.new_path.toLowerCase().includes(q) ||
        ev.summary.toLowerCase().includes(q) ||
        ev.event_type.toLowerCase().includes(q)
      );
    }
    return true;
  });

  const exportActivity = () => {
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(filteredEvents, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', `changeradar-activity-${new Date().toISOString().slice(0, 10)}.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  };

  return (
    <div className="space-y-4">
      {/* Filter and Control Bar */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center space-x-3 flex-1 min-w-[280px]">
          <div className="relative flex-1">
            <Search className="w-4 h-4 text-[#A1A6B2] absolute left-3 top-2.5" />
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search activity by filename, summary, or path..."
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg pl-9 pr-3 py-1.5 text-xs text-[#F2F3F5] placeholder-[#A1A6B2] focus:outline-none focus:border-[#6C8CFF]"
            />
          </div>

          <div className="flex items-center space-x-2">
            <Filter className="w-4 h-4 text-[#A1A6B2]" />
            <select
              value={selectedFolder}
              onChange={(e) => setSelectedFolder(e.target.value)}
              className="bg-[#20232B] border border-[#2C303A] rounded-lg px-2.5 py-1.5 text-xs text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            >
              <option value="ALL">All Folders</option>
              {folders.map((f) => (
                <option key={f.id} value={f.id}>
                  {f.display_name}
                </option>
              ))}
            </select>

            <select
              value={selectedKind}
              onChange={(e) => setSelectedKind(e.target.value)}
              className="bg-[#20232B] border border-[#2C303A] rounded-lg px-2.5 py-1.5 text-xs text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            >
              <option value="ALL">All Event Types</option>
              <option value="CREATED">CREATED</option>
              <option value="MODIFIED">MODIFIED</option>
              <option value="DELETED">DELETED</option>
              <option value="RENAMED">RENAMED</option>
              <option value="RESTORED">RESTORED</option>
            </select>
          </div>
        </div>

        <button
          onClick={exportActivity}
          className="flex items-center space-x-1.5 px-3 py-1.5 bg-[#20232B] hover:bg-[#252933] border border-[#2C303A] text-xs font-semibold text-[#F2F3F5] rounded-lg transition-colors"
        >
          <Download className="w-3.5 h-3.5 text-[#6C8CFF]" />
          <span>Export Activity ({filteredEvents.length})</span>
        </button>
      </div>

      {/* Events Table */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-[#20232B] text-[#A1A6B2] uppercase text-[10px] font-semibold tracking-wider border-b border-[#2C303A]">
              <tr>
                <th className="p-3">Timestamp</th>
                <th className="p-3">Event Type</th>
                <th className="p-3">File Path</th>
                <th className="p-3">Change Summary</th>
                <th className="p-3">Delta</th>
                <th className="p-3 text-right">Confidence</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#2C303A]">
              {filteredEvents.length === 0 ? (
                <tr>
                  <td colSpan={6} className="p-8 text-center text-xs text-[#A1A6B2]">
                    No change events matching current filters.
                  </td>
                </tr>
              ) : (
                filteredEvents.map((ev) => (
                  <tr key={ev.id} className="hover:bg-[#20232B]/40 transition-colors">
                    <td className="p-3 text-[#A1A6B2] font-mono whitespace-nowrap">
                      {new Date(ev.created_at).toLocaleString([], {
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                        second: '2-digit',
                      })}
                    </td>
                    <td className="p-3 whitespace-nowrap">
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
                    </td>
                    <td className="p-3 font-mono font-medium text-[#F2F3F5] max-w-xs truncate">
                      {ev.new_path}
                    </td>
                    <td className="p-3 text-[#A1A6B2] max-w-sm truncate">{ev.summary}</td>
                    <td className="p-3 font-mono whitespace-nowrap">
                      {ev.file_size_diff > 0 ? (
                        <span className="text-[#42C98A]">+{ev.file_size_diff} B</span>
                      ) : ev.file_size_diff < 0 ? (
                        <span className="text-[#EA6868]">{ev.file_size_diff} B</span>
                      ) : (
                        <span className="text-[#A1A6B2]">0 B</span>
                      )}
                    </td>
                    <td className="p-3 text-right font-mono text-[10px] text-[#A1A6B2]">
                      {ev.confidence}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
