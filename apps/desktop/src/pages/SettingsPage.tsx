import React, { useState } from 'react';
import {
  Settings,
  ShieldCheck,
  Moon,
  Sun,
  Server,
  Lock,
  HardDrive,
  CheckCircle2,
} from 'lucide-react';

export const SettingsPage: React.FC = () => {
  const [theme, setTheme] = useState('dark');
  const [apiPort, setApiPort] = useState(41820);
  const [scanInterval, setScanInterval] = useState(300);
  const [saved, setSaved] = useState(false);

  const saveSettings = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 2500);
  };

  return (
    <div className="space-y-6 max-w-4xl">
      {/* Header */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 flex items-center justify-between">
        <div>
          <h3 className="text-base font-bold text-[#F2F3F5]">Application Settings & Privacy</h3>
          <p className="text-xs text-[#A1A6B2]">
            Local-first configuration. All settings and data stay strictly on your local computer.
          </p>
        </div>

        {saved && (
          <div className="flex items-center space-x-1.5 px-3 py-1.5 bg-[#42C98A]/20 border border-[#42C98A]/30 text-[#42C98A] text-xs font-semibold rounded-lg animate-in fade-in">
            <CheckCircle2 className="w-4 h-4" />
            <span>Preferences Saved</span>
          </div>
        )}
      </div>

      {/* Appearance Section */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-4">
        <h4 className="text-sm font-bold text-[#F2F3F5] flex items-center gap-2">
          <Moon className="w-4 h-4 text-[#6C8CFF]" />
          <span>Appearance & Workstation Theme</span>
        </h4>

        <div className="grid grid-cols-3 gap-3">
          <button
            onClick={() => setTheme('dark')}
            className={`p-3 rounded-lg border text-left text-xs font-semibold transition-colors flex items-center space-x-2.5 ${
              theme === 'dark'
                ? 'bg-[#20232B] border-[#6C8CFF] text-[#F2F3F5]'
                : 'bg-[#181A20] border-[#2C303A] text-[#A1A6B2]'
            }`}
          >
            <Moon className="w-4 h-4 text-[#6C8CFF]" />
            <span>Dark Charcoal (Default)</span>
          </button>

          <button
            onClick={() => setTheme('light')}
            className={`p-3 rounded-lg border text-left text-xs font-semibold transition-colors flex items-center space-x-2.5 ${
              theme === 'light'
                ? 'bg-[#20232B] border-[#6C8CFF] text-[#F2F3F5]'
                : 'bg-[#181A20] border-[#2C303A] text-[#A1A6B2]'
            }`}
          >
            <Sun className="w-4 h-4 text-[#E9B65B]" />
            <span>Light Slate</span>
          </button>

          <button
            onClick={() => setTheme('system')}
            className={`p-3 rounded-lg border text-left text-xs font-semibold transition-colors flex items-center space-x-2.5 ${
              theme === 'system'
                ? 'bg-[#20232B] border-[#6C8CFF] text-[#F2F3F5]'
                : 'bg-[#181A20] border-[#2C303A] text-[#A1A6B2]'
            }`}
          >
            <Settings className="w-4 h-4 text-[#42C98A]" />
            <span>Sync with OS</span>
          </button>
        </div>
      </div>

      {/* Monitoring & Developer API */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-4">
        <h4 className="text-sm font-bold text-[#F2F3F5] flex items-center gap-2">
          <Server className="w-4 h-4 text-[#6C8CFF]" />
          <span>Local Developer API & Reconciliation</span>
        </h4>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-1">
            <label className="text-xs font-semibold text-[#A1A6B2]">Developer API Port:</label>
            <input
              type="number"
              value={apiPort}
              onChange={(e) => setApiPort(Number(e.target.value))}
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2 text-xs text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            />
            <span className="text-[10px] text-[#A1A6B2]">
              Bound strictly to 127.0.0.1 for local daemon & CLI access.
            </span>
          </div>

          <div className="space-y-1">
            <label className="text-xs font-semibold text-[#A1A6B2]">
              Reconciliation Interval (Seconds):
            </label>
            <input
              type="number"
              value={scanInterval}
              onChange={(e) => setScanInterval(Number(e.target.value))}
              className="w-full bg-[#20232B] border border-[#2C303A] rounded-lg p-2 text-xs text-[#F2F3F5] focus:outline-none focus:border-[#6C8CFF]"
            />
            <span className="text-[10px] text-[#A1A6B2]">
              Frequency of periodic disk vs SQLite state checks.
            </span>
          </div>
        </div>

        <button
          onClick={saveSettings}
          className="px-4 py-2 bg-[#6C8CFF] hover:bg-[#5876E8] text-white text-xs font-semibold rounded-lg transition-colors"
        >
          Save Configuration
        </button>
      </div>

      {/* Privacy Guarantee Card */}
      <div className="bg-[#181A20] border border-[#42C98A]/30 rounded-xl p-5 space-y-3">
        <div className="flex items-center space-x-2 text-[#42C98A]">
          <ShieldCheck className="w-5 h-5" />
          <h4 className="text-sm font-bold">Privacy & Security Guarantee</h4>
        </div>

        <p className="text-xs text-[#A1A6B2] leading-relaxed">
          ChangeRadar is built upon a 100% offline-first architecture. There is no cloud account, no mandatory subscription, and no telemetry by default. File content never leaves your computer. All cryptographic hashes and compressed versions are stored in your local user data directory.
        </p>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-2 pt-2 text-[11px] text-[#A1A6B2]">
          <div className="flex items-center gap-1.5 text-[#42C98A]">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>Zero Telemetry</span>
          </div>
          <div className="flex items-center gap-1.5 text-[#42C98A]">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>No Cloud Dependency</span>
          </div>
          <div className="flex items-center gap-1.5 text-[#42C98A]">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>Local SQLite WAL</span>
          </div>
          <div className="flex items-center gap-1.5 text-[#42C98A]">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>Open Source (MIT)</span>
          </div>
        </div>
      </div>
    </div>
  );
};
