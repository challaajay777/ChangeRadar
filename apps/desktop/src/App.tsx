import React, { useState, useEffect } from 'react';
import { Sidebar, NavSection } from './components/Sidebar';
import { Header } from './components/Header';
import { CommandPalette } from './components/CommandPalette';
import { AddFolderModal } from './components/AddFolderModal';
import { CreateSnapshotModal } from './components/CreateSnapshotModal';

import { OverviewPage } from './pages/OverviewPage';
import { ActivityPage } from './pages/ActivityPage';
import { FoldersPage } from './pages/FoldersPage';
import { FileExplorerPage } from './pages/FileExplorerPage';
import { VersionHistoryPage } from './pages/VersionHistoryPage';
import { ComparePage } from './pages/ComparePage';
import { SnapshotsPage } from './pages/SnapshotsPage';
import { RecoveryCenterPage } from './pages/RecoveryCenterPage';
import { ChangeIntelligencePage } from './pages/ChangeIntelligencePage';
import { NotificationsPage } from './pages/NotificationsPage';
import { StoragePage } from './pages/StoragePage';
import { PluginsPage } from './pages/PluginsPage';
import { SettingsPage } from './pages/SettingsPage';
import { DiagnosticsPage } from './pages/DiagnosticsPage';

import {
  MonitoredFolder,
  FileRecord,
  FileEvent,
  SystemOverview,
  Snapshot,
  NotificationItem,
} from './types';
import { api } from './services/api';

export const App: React.FC = () => {
  const [currentSection, setCurrentSection] = useState<NavSection>('overview');
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isAddFolderOpen, setIsAddFolderOpen] = useState(false);
  const [isCreateSnapshotOpen, setIsCreateSnapshotOpen] = useState(false);

  // Cross-page navigation context
  const [selectedFileForVersions, setSelectedFileForVersions] = useState<string | null>(null);
  const [compareVersionsPair, setCompareVersionsPair] = useState<[string, string] | null>(null);
  const [restoreTargetVersionId, setRestoreTargetVersionId] = useState<string | undefined>(undefined);

  // Global state
  const [overview, setOverview] = useState<SystemOverview>({
    is_monitoring: true,
    active_folders: 0,
    total_tracked_files: 0,
    changes_today: 0,
    versions_stored: 0,
    total_cas_bytes: 0,
    uncompressed_bytes: 0,
    deduplication_savings_pct: 0,
    radar_score: 38,
  });
  const [folders, setFolders] = useState<MonitoredFolder[]>([]);
  const [events, setEvents] = useState<FileEvent[]>([]);
  const [files, setFiles] = useState<FileRecord[]>([]);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [notifications, setNotifications] = useState<NotificationItem[]>([]);

  const refreshAllData = async () => {
    try {
      const [ov, flds, evts, fls, snaps, notifs] = await Promise.all([
        api.getOverview(),
        api.listFolders(),
        api.listEvents(),
        api.listFiles(),
        api.listSnapshots(),
        api.listNotifications(),
      ]);
      setOverview(ov);
      setFolders(flds);
      setEvents(evts);
      setFiles(fls);
      setSnapshots(snaps);
      setNotifications(notifs);
    } catch (e) {
      console.warn('Data sync notice:', e);
    }
  };

  useEffect(() => {
    refreshAllData();
    const interval = setInterval(refreshAllData, 4000);
    return () => clearInterval(interval);
  }, []);

  const handleSelectFileForVersions = (fileId: string) => {
    setSelectedFileForVersions(fileId);
    setCurrentSection('versions');
  };

  const handleCompareVersions = (verA: string, verB: string) => {
    setCompareVersionsPair([verA, verB]);
    setCurrentSection('compare');
  };

  const handleRestoreVersion = (versionId: string) => {
    setRestoreTargetVersionId(versionId);
    setCurrentSection('recovery');
  };

  const unreadCount = notifications.filter((n) => !n.is_read).length;

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#101114]">
      {/* Sidebar */}
      <Sidebar
        currentSection={currentSection}
        onSelectSection={(s) => setCurrentSection(s)}
        unreadNotificationsCount={unreadCount}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col h-full min-w-0 overflow-hidden">
        <Header
          currentSection={currentSection}
          onOpenCommandPalette={() => setIsCommandPaletteOpen(true)}
          onQuickAddFolder={() => setIsAddFolderOpen(true)}
          onOpenNotifications={() => setCurrentSection('notifications')}
          onOpenDiagnostics={() => setCurrentSection('diagnostics')}
          radarScore={overview.radar_score}
        />

        <main className="flex-1 overflow-y-auto p-6">
          {currentSection === 'overview' && (
            <OverviewPage
              overview={overview}
              recentEvents={events}
              onNavigate={(s) => setCurrentSection(s)}
              onAddFolder={() => setIsAddFolderOpen(true)}
              onCreateSnapshot={() => setIsCreateSnapshotOpen(true)}
            />
          )}

          {currentSection === 'activity' && (
            <ActivityPage
              events={events}
              folders={folders}
              onOpenCompare={(fileId) => {
                if (fileId) setSelectedFileForVersions(fileId);
                setCurrentSection('compare');
              }}
            />
          )}

          {currentSection === 'folders' && (
            <FoldersPage
              folders={folders}
              onRefresh={refreshAllData}
              onOpenAddModal={() => setIsAddFolderOpen(true)}
            />
          )}

          {currentSection === 'files' && (
            <FileExplorerPage
              files={files}
              folders={folders}
              onSelectFileForVersions={handleSelectFileForVersions}
            />
          )}

          {currentSection === 'versions' && (
            <VersionHistoryPage
              files={files}
              selectedFileId={selectedFileForVersions}
              onSelectFile={(id) => setSelectedFileForVersions(id)}
              onRestoreVersion={handleRestoreVersion}
              onCompareVersions={handleCompareVersions}
            />
          )}

          {currentSection === 'compare' && (
            <ComparePage
              initialVersionA={compareVersionsPair?.[0]}
              initialVersionB={compareVersionsPair?.[1]}
            />
          )}

          {currentSection === 'snapshots' && (
            <SnapshotsPage
              snapshots={snapshots}
              folders={folders}
              onRefresh={refreshAllData}
              onCreateSnapshot={() => setIsCreateSnapshotOpen(true)}
            />
          )}

          {currentSection === 'recovery' && (
            <RecoveryCenterPage
              files={files}
              initialVersionId={restoreTargetVersionId}
            />
          )}

          {currentSection === 'intelligence' && (
            <ChangeIntelligencePage radarScore={overview.radar_score} />
          )}

          {currentSection === 'notifications' && (
            <NotificationsPage
              notifications={notifications}
              onRefresh={refreshAllData}
            />
          )}

          {currentSection === 'storage' && <StoragePage />}

          {currentSection === 'plugins' && <PluginsPage />}

          {currentSection === 'settings' && <SettingsPage />}

          {currentSection === 'diagnostics' && <DiagnosticsPage />}
        </main>
      </div>

      {/* Global Command Palette */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        onSelectSection={(s) => setCurrentSection(s)}
        onAddFolder={() => setIsAddFolderOpen(true)}
        onCreateSnapshot={() => setIsCreateSnapshotOpen(true)}
      />

      {/* Add Folder Modal */}
      <AddFolderModal
        isOpen={isAddFolderOpen}
        onClose={() => setIsAddFolderOpen(false)}
        onSuccess={refreshAllData}
      />

      {/* Create Snapshot Modal */}
      <CreateSnapshotModal
        isOpen={isCreateSnapshotOpen}
        folders={folders}
        onClose={() => setIsCreateSnapshotOpen(false)}
        onSuccess={refreshAllData}
      />
    </div>
  );
};
