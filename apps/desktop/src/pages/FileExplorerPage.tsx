import React, { useState } from 'react';
import { Files, Search, Filter, History, ExternalLink } from 'lucide-react';
import { FileRecord, MonitoredFolder } from '../types';

interface FileExplorerPageProps {
  files: FileRecord[];
  folders: MonitoredFolder[];
  onSelectFileForVersions: (fileId: string) => void;
}

export const FileExplorerPage: React.FC<FileExplorerPageProps> = ({
  files,
  folders,
  onSelectFileForVersions,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedFolder, setSelectedFolder] = useState<string>('ALL');

  const folderMap = new Map(folders.map((f) => [f.id, f.display_name]));

  const filtered = files.filter((file) => {
    if (selectedFolder !== 'ALL' && file.folder_id !== selectedFolder) {
      return false;
    }
    if (searchTerm.trim() !== '') {
      const q = searchTerm.toLowerCase();
      return (
        file.file_name.toLowerCase().includes(q) ||
        file.relative_path.toLowerCase().includes(q) ||
        file.extension.toLowerCase().includes(q) ||
        file.current_hash.toLowerCase().includes(q)
      );
    }
    return true;
  });

  return (
    <div className="space-y-4">
      {/* Filter and Search Bar */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center space-x-3 flex-1 min-w-[280px]">
          <div className="relative flex-1">
            <Search className="w-4 h-4 text-[#A1A6B2] absolute left-3 top-2.5" />
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search files by name, extension, relative path, or hash..."
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
          </div>
        </div>

        <div className="text-xs text-[#A1A6B2]">
          Showing <span className="font-semibold text-[#F2F3F5]">{filtered.length}</span> tracked files
        </div>
      </div>

      {/* Files Table */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-[#20232B] text-[#A1A6B2] uppercase text-[10px] font-semibold tracking-wider border-b border-[#2C303A]">
              <tr>
                <th className="p-3">File Name</th>
                <th className="p-3">Folder</th>
                <th className="p-3">Relative Path</th>
                <th className="p-3">Size</th>
                <th className="p-3">Current Hash</th>
                <th className="p-3">Updated</th>
                <th className="p-3 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#2C303A]">
              {filtered.length === 0 ? (
                <tr>
                  <td colSpan={7} className="p-8 text-center text-xs text-[#A1A6B2]">
                    No files found matching the search criteria.
                  </td>
                </tr>
              ) : (
                filtered.map((file) => (
                  <tr key={file.id} className="hover:bg-[#20232B]/40 transition-colors">
                    <td className="p-3 font-medium text-[#F2F3F5] flex items-center space-x-2">
                      <span className="px-1.5 py-0.5 rounded bg-[#20232B] border border-[#2C303A] text-[10px] uppercase font-mono text-[#6C8CFF]">
                        {file.extension || 'file'}
                      </span>
                      <span>{file.file_name}</span>
                      {file.is_deleted && (
                        <span className="px-1.5 py-0.2 bg-[#EA6868]/20 text-[#EA6868] text-[9px] font-bold rounded">
                          DELETED
                        </span>
                      )}
                    </td>
                    <td className="p-3 text-[#A1A6B2]">
                      {folderMap.get(file.folder_id) || 'Unknown'}
                    </td>
                    <td className="p-3 font-mono text-[#A1A6B2] max-w-xs truncate">
                      {file.relative_path}
                    </td>
                    <td className="p-3 font-mono text-[#F2F3F5]">
                      {file.file_size > 1024 * 1024
                        ? `${(file.file_size / (1024 * 1024)).toFixed(1)} MB`
                        : file.file_size > 1024
                        ? `${(file.file_size / 1024).toFixed(1)} KB`
                        : `${file.file_size} B`}
                    </td>
                    <td className="p-3 font-mono text-[11px] text-[#A1A6B2]">
                      {file.current_hash.slice(0, 12)}...
                    </td>
                    <td className="p-3 text-[#A1A6B2] font-mono whitespace-nowrap">
                      {new Date(file.updated_at).toLocaleDateString()}
                    </td>
                    <td className="p-3 text-right">
                      <button
                        onClick={() => onSelectFileForVersions(file.id)}
                        className="flex items-center space-x-1 ml-auto px-2.5 py-1 bg-[#20232B] hover:bg-[#6C8CFF] text-[#F2F3F5] hover:text-white rounded border border-[#2C303A] text-xs font-semibold transition-colors"
                      >
                        <History className="w-3.5 h-3.5" />
                        <span>History</span>
                      </button>
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
