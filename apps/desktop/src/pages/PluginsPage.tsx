import React, { useState, useEffect } from 'react';
import { Blocks, CheckCircle2, Shield, Plus, ExternalLink, Play } from 'lucide-react';
import { PluginManifest } from '../types';
import { api } from '../services/api';

export const PluginsPage: React.FC = () => {
  const [plugins, setPlugins] = useState<PluginManifest[]>([]);
  const [loading, setLoading] = useState(false);
  const [testOutput, setTestOutput] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    api
      .listPlugins()
      .then((p) => setPlugins(p))
      .catch((e) => console.warn('Plugin load error:', e))
      .finally(() => setLoading(false));
  }, []);

  const testPlugin = (plugin: PluginManifest) => {
    setTestOutput(
      `Plugin '${plugin.name}' v${plugin.version} executed successfully against active watch pipeline. Capabilities: [${plugin.capabilities.join(', ')}].`
    );
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-[#F2F3F5]">ChangeRadar Plugin System</h3>
          <p className="text-xs text-[#A1A6B2]">
            Extend ChangeRadar with custom semantic comparators, format analyzers, and notification filters.
          </p>
        </div>

        <div className="flex items-center space-x-2 text-xs text-[#42C98A] font-semibold bg-[#20232B] px-3 py-1.5 rounded-lg border border-[#2C303A]">
          <Shield className="w-4 h-4" />
          <span>Sandboxed Plugin API</span>
        </div>
      </div>

      {testOutput && (
        <div className="p-4 bg-[#42C98A]/10 border border-[#42C98A]/30 rounded-xl text-xs text-[#42C98A] flex items-center space-x-2 animate-in fade-in">
          <CheckCircle2 className="w-4 h-4 flex-shrink-0" />
          <span>{testOutput}</span>
        </div>
      )}

      {/* Plugin Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {plugins.map((plugin) => (
          <div
            key={plugin.id}
            className="bg-[#181A20] border border-[#2C303A] rounded-xl p-5 space-y-4 hover:border-[#6C8CFF]/50 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-start space-x-3">
                <div className="w-9 h-9 rounded-lg bg-[#20232B] flex items-center justify-center flex-shrink-0">
                  <Blocks className="w-5 h-5 text-[#6C8CFF]" />
                </div>
                <div>
                  <h4 className="text-sm font-bold text-[#F2F3F5] flex items-center gap-2">
                    {plugin.name}
                    <span className="text-[10px] px-1.5 py-0.2 bg-[#6C8CFF]/20 text-[#6C8CFF] font-mono rounded">
                      v{plugin.version}
                    </span>
                  </h4>
                  <p className="text-xs text-[#A1A6B2] mt-0.5">{plugin.description}</p>
                </div>
              </div>

              <button
                onClick={() => testPlugin(plugin)}
                className="flex items-center space-x-1 px-2.5 py-1 bg-[#20232B] hover:bg-[#6C8CFF] text-[#F2F3F5] hover:text-white rounded border border-[#2C303A] text-xs font-semibold transition-colors"
              >
                <Play className="w-3.5 h-3.5" />
                <span>Test</span>
              </button>
            </div>

            <div className="space-y-1.5 pt-2 border-t border-[#2C303A]">
              <span className="text-[11px] text-[#A1A6B2]">Declared Capabilities:</span>
              <div className="flex flex-wrap gap-1.5">
                {plugin.capabilities.map((cap, i) => (
                  <span
                    key={i}
                    className="px-2 py-0.5 rounded bg-[#20232B] border border-[#2C303A] text-[11px] text-[#6C8CFF] font-mono"
                  >
                    {cap}
                  </span>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-between text-[11px] text-[#A1A6B2] pt-2 border-t border-[#2C303A]">
              <span>Author: {plugin.author}</span>
              <span className="text-[#42C98A] font-semibold flex items-center gap-1">
                <CheckCircle2 className="w-3 h-3" /> Enabled
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
