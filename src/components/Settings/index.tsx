import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import {
  Save,
  Loader2,
  Trash2,
  AlertTriangle,
  X,
  Globe,
  Download,
  Upload,
  RefreshCw,
  CheckCircle,
  ArrowUpCircle,
  Database,
  Clock,
  Server,
  FileJson,
  GitMerge
} from 'lucide-react';
import { appLogger } from '../../lib/logger';
import { isTauri } from '../../lib/tauri';

interface InstallResult {
  success: boolean;
  message: string;
  error?: string;
}

interface SettingsProps {
  onEnvironmentChange?: () => void;
}

interface BrowserConfig {
  enabled: boolean;
  color: string | null;
}

interface WebConfig {
  brave_api_key: string | null;
}

interface CompactionConfig {
  enabled: boolean;
  threshold: number | null;
  context_pruning: boolean;
  max_context_messages: number | null;
}

interface WorkspaceConfig {
  workspace: string | null;
  timezone: string | null;
  time_format: string | null;
  skip_bootstrap: boolean;
  bootstrap_max_chars: number | null;
}

interface GatewayConfig {
  port: number;
  log_level: string;
}

interface SubagentDefaults {
  max_spawn_depth: number | null;
  max_children_per_agent: number | null;
  max_concurrent: number | null;
}

export function Settings({ onEnvironmentChange }: SettingsProps) {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  const [showUninstallConfirm, setShowUninstallConfirm] = useState(false);
  const [uninstalling, setUninstalling] = useState(false);
  const [uninstallResult, setUninstallResult] = useState<InstallResult | null>(null);

  // Manager self-update states
  const [managerChecking, setManagerChecking] = useState(false);
  const [managerUpdateAvailable, setManagerUpdateAvailable] = useState(false);
  const [managerUpdateVersion, setManagerUpdateVersion] = useState<string | null>(null);
  const [managerUpdateBody, setManagerUpdateBody] = useState<string | null>(null);
  const [managerDownloading, setManagerDownloading] = useState(false);
  const [managerDownloadProgress, setManagerDownloadProgress] = useState(0);
  const [managerUpdateDone, setManagerUpdateDone] = useState(false);
  const [managerUpdateError, setManagerUpdateError] = useState<string | null>(null);
  const [managerUpdateObj, setManagerUpdateObj] = useState<any>(null);
  const [managerCheckDone, setManagerCheckDone] = useState(false);

  // Config States
  const [browser, setBrowser] = useState<BrowserConfig>({ enabled: true, color: null });
  const [webConfig, setWebConfig] = useState<WebConfig>({ brave_api_key: null });
  const [compaction, setCompaction] = useState<CompactionConfig>({ enabled: false, threshold: null, context_pruning: false, max_context_messages: null });
  const [workspace, setWorkspace] = useState<WorkspaceConfig>({ workspace: null, timezone: null, time_format: null, skip_bootstrap: false, bootstrap_max_chars: null });
  const [gateway, setGateway] = useState<GatewayConfig>({ port: 3000, log_level: 'info' });
  const [subagentDefaults, setSubagentDefaults] = useState<SubagentDefaults>({ max_spawn_depth: null, max_children_per_agent: null, max_concurrent: null });
  const [appVersion, setAppVersion] = useState<string>('...');

  // Load initial data
  useEffect(() => {
    const loadConfig = async () => {
      setLoading(true);
      try {
        const [br, web, comp, ws, gw, sub] = await Promise.all([
          invoke<BrowserConfig>('get_browser_config'),
          invoke<WebConfig>('get_web_config'),
          invoke<CompactionConfig>('get_compaction_config'),
          invoke<WorkspaceConfig>('get_workspace_config'),
          invoke<GatewayConfig>('get_gateway_config'),
          invoke<SubagentDefaults>('get_subagent_defaults'),
        ]);
        setBrowser(br);
        setWebConfig(web);
        setCompaction(comp);
        setWorkspace(ws);
        setGateway(gw);
        setSubagentDefaults(sub);

        if (isTauri()) {
          const { getVersion } = await import('@tauri-apps/api/app');
          setAppVersion(await getVersion());
        }
      } catch (e) {
        appLogger.error('Failed to load settings', e);
      } finally {
        setLoading(false);
      }
    };
    loadConfig();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setSaveSuccess(false);
    try {
      await Promise.all([
        invoke('save_browser_config', { enabled: browser.enabled, color: browser.color }),
        invoke('save_web_config', { braveApiKey: webConfig.brave_api_key }),
        invoke('save_compaction_config', {
          enabled: compaction.enabled,
          threshold: compaction.threshold,
          contextPruning: compaction.context_pruning,
          maxContextMessages: compaction.max_context_messages
        }),
        invoke('save_workspace_config', {
          workspace: workspace.workspace,
          timezone: workspace.timezone,
          timeFormat: workspace.time_format,
          skipBootstrap: workspace.skip_bootstrap,
          bootstrapMaxChars: workspace.bootstrap_max_chars
        }),
        invoke('save_gateway_config', { port: gateway.port, logLevel: gateway.log_level }),
        invoke('save_subagent_defaults', { defaults: subagentDefaults }),
      ]);

      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } catch (e) {
      console.error('Failed to save:', e);
      alert('Failed to save settings: ' + String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleExport = async () => {
    try {
      const path = await save({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }],
        defaultPath: 'openclaw-config.json'
      });

      if (path) {
        await invoke('export_config', { path });
        alert('Configuration exported successfully!');
      }
    } catch (e) {
      console.error('Export failed:', e);
      alert('Failed to export configuration: ' + String(e));
    }
  };

  const handleImport = async () => {
    if (!confirm('Importing configuration will overwrite your current settings. Continue?')) return;

    try {
      const path = await open({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (path) {
        await invoke('import_config', { path });
        alert('Configuration imported successfully! Please restart the manager to apply all changes.');
        // Reload settings to reflect changes
        window.location.reload();
      }
    } catch (e) {
      console.error('Import failed:', e);
      alert('Failed to import configuration: ' + String(e));
    }
  };

  const handleUninstall = async () => {
    setUninstalling(true);
    setUninstallResult(null);
    try {
      const result = await invoke<InstallResult>('uninstall_openclaw');
      setUninstallResult(result);
      if (result.success) {
        onEnvironmentChange?.();
        setTimeout(() => setShowUninstallConfirm(false), 2000);
      }
    } catch (e) {
      setUninstallResult({
        success: false,
        message: 'An error occurred during uninstallation',
        error: String(e),
      });
    } finally {
      setUninstalling(false);
    }
  };

  // Manager self-update: check for updates
  const checkManagerUpdate = async () => {
    if (!isTauri()) return;
    setManagerChecking(true);
    setManagerUpdateError(null);
    setManagerUpdateAvailable(false);
    setManagerCheckDone(false);
    setManagerUpdateDone(false);
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        setManagerUpdateAvailable(true);
        setManagerUpdateVersion(update.version);
        setManagerUpdateBody(update.body || null);
        setManagerUpdateObj(update);
      } else {
        setManagerCheckDone(true);
      }
    } catch (e: any) {
      appLogger.error('Manager update check failed', e);
      setManagerUpdateError(e?.message || String(e));
    } finally {
      setManagerChecking(false);
    }
  };

  // Manager self-update: download & install
  const downloadManagerUpdate = async () => {
    if (!managerUpdateObj) return;
    setManagerDownloading(true);
    setManagerDownloadProgress(0);
    setManagerUpdateError(null);
    try {
      let downloaded = 0;
      let contentLength = 1;
      await managerUpdateObj.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength || 1;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            setManagerDownloadProgress(Math.min(100, Math.round((downloaded / contentLength) * 100)));
            break;
          case 'Finished':
            setManagerDownloadProgress(100);
            break;
        }
      });
      setManagerUpdateDone(true);
    } catch (e: any) {
      appLogger.error('Manager update download failed', e);
      setManagerUpdateError(e?.message || String(e));
    } finally {
      setManagerDownloading(false);
    }
  };

  // Manager self-update: restart app
  const restartApp = async () => {
    try {
      const { relaunch } = await import('@tauri-apps/plugin-process');
      await relaunch();
    } catch (e: any) {
      appLogger.error('Relaunch failed', e);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="animate-spin text-claw-400" size={32} />
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2 pb-20">
      <div className="max-w-3xl space-y-6 mx-auto">

        {/* Compaction & Memory */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
              <Database size={20} className="text-purple-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Compaction & Memory</h3>
              <p className="text-xs text-gray-500">Manage agent memory optimization</p>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">Enable Compaction</p>
                <p className="text-xs text-gray-500">Compress conversation history when it gets too long</p>
              </div>
              <input
                type="checkbox"
                checked={compaction.enabled}
                onChange={e => setCompaction({ ...compaction, enabled: e.target.checked })}
                className="w-5 h-5 rounded bg-dark-500 border-dark-400 text-claw-500 focus:ring-claw-500/50"
              />
            </div>

            {compaction.enabled && (
              <div className="pl-4 border-l-2 border-dark-600">
                <label className="block text-sm text-gray-400 mb-2">Token Threshold</label>
                <input
                  type="number"
                  value={compaction.threshold || ''}
                  onChange={e => setCompaction({ ...compaction, threshold: parseInt(e.target.value) || null })}
                  placeholder="e.g. 8000"
                  className="input-base"
                />
                <p className="text-xs text-gray-500 mt-1">Number of tokens before compaction triggers.</p>
              </div>
            )}

            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">Context Pruning</p>
                <p className="text-xs text-gray-500">Limit the number of recent messages kept in context</p>
              </div>
              <input
                type="checkbox"
                checked={compaction.context_pruning}
                onChange={e => setCompaction({ ...compaction, context_pruning: e.target.checked })}
                className="w-5 h-5 rounded bg-dark-500 border-dark-400 text-claw-500 focus:ring-claw-500/50"
              />
            </div>

            {compaction.context_pruning && (
              <div className="pl-4 border-l-2 border-dark-600">
                <label className="block text-sm text-gray-400 mb-2">Max Messages</label>
                <input
                  type="number"
                  value={compaction.max_context_messages || ''}
                  onChange={e => setCompaction({ ...compaction, max_context_messages: parseInt(e.target.value) || null })}
                  placeholder="e.g. 50"
                  className="input-base"
                />
                <p className="text-xs text-gray-500 mt-1">Maximum number of recent messages to keep.</p>
              </div>
            )}
          </div>
        </div>

        {/* Workspace */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
              <Clock size={20} className="text-orange-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Workspace</h3>
              <p className="text-xs text-gray-500">Time and localization settings</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Timezone</label>
              <select
                value={workspace.timezone || 'Asia/Shanghai'}
                onChange={e => setWorkspace({ ...workspace, timezone: e.target.value })}
                className="input-base"
              >
                <option value="Asia/Shanghai">Asia/Shanghai</option>
                <option value="Asia/Hong_Kong">Asia/Hong_Kong</option>
                <option value="Asia/Tokyo">Asia/Tokyo</option>
                <option value="America/New_York">America/New_York</option>
                <option value="America/Los_Angeles">America/Los_Angeles</option>
                <option value="Europe/London">Europe/London</option>
                <option value="UTC">UTC</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Time Format</label>
              <select
                value={workspace.time_format || ''}
                onChange={e => setWorkspace({ ...workspace, time_format: e.target.value || null })}
                className="input-base"
              >
                <option value="">Default (24h)</option>
                <option value="12h">12h (AM/PM)</option>
                <option value="24h">24h</option>
              </select>
            </div>
          </div>
        </div>

        {/* Gateway Settings */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center">
              <Server size={20} className="text-cyan-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Gateway Settings</h3>
              <p className="text-xs text-gray-500">Network and logging configuration</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Gateway Port</label>
              <input
                type="number"
                value={gateway.port}
                onChange={e => setGateway({ ...gateway, port: parseInt(e.target.value) || 3000 })}
                className="input-base"
              />
              <p className="text-xs text-yellow-500/80 mt-1 flex items-center gap-1">
                <AlertTriangle size={12} /> Requires restart
              </p>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Log Level</label>
              <select
                value={gateway.log_level}
                onChange={e => setGateway({ ...gateway, log_level: e.target.value })}
                className="input-base"
              >
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warn</option>
                <option value="error">Error</option>
              </select>
            </div>
          </div>
        </div>

        {/* Subagent Defaults */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-indigo-500/20 flex items-center justify-center">
              <GitMerge size={20} className="text-indigo-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Subagent Defaults</h3>
              <p className="text-xs text-gray-500">Global limits for nested subagent spawning</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Max Spawn Depth</label>
              <input
                type="number"
                min={0}
                max={10}
                value={subagentDefaults.max_spawn_depth ?? ''}
                onChange={e => setSubagentDefaults({ ...subagentDefaults, max_spawn_depth: e.target.value ? parseInt(e.target.value) : null })}
                className="input-base"
                placeholder="2"
              />
              <p className="text-xs text-gray-600 mt-1">Nesting levels</p>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Max Children / Agent</label>
              <input
                type="number"
                min={0}
                max={50}
                value={subagentDefaults.max_children_per_agent ?? ''}
                onChange={e => setSubagentDefaults({ ...subagentDefaults, max_children_per_agent: e.target.value ? parseInt(e.target.value) : null })}
                className="input-base"
                placeholder="5"
              />
              <p className="text-xs text-gray-600 mt-1">Per parent</p>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Max Concurrent</label>
              <input
                type="number"
                min={0}
                max={100}
                value={subagentDefaults.max_concurrent ?? ''}
                onChange={e => setSubagentDefaults({ ...subagentDefaults, max_concurrent: e.target.value ? parseInt(e.target.value) : null })}
                className="input-base"
                placeholder="8"
              />
              <p className="text-xs text-gray-600 mt-1">System-wide</p>
            </div>
          </div>
        </div>

        {/* Configuration Management */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-slate-500/20 flex items-center justify-center">
              <FileJson size={20} className="text-slate-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Configuration Management</h3>
              <p className="text-xs text-gray-500">Backup and restore settings</p>
            </div>
          </div>

          <div className="flex gap-4">
            <button
              onClick={handleExport}
              className="flex-1 flex items-center justify-center gap-2 p-3 bg-dark-600 hover:bg-dark-500 rounded-lg transition-colors text-sm text-white border border-dark-500 hover:border-dark-400"
            >
              <Download size={16} />
              Export Config
            </button>
            <button
              onClick={handleImport}
              className="flex-1 flex items-center justify-center gap-2 p-3 bg-dark-600 hover:bg-dark-500 rounded-lg transition-colors text-sm text-white border border-dark-500 hover:border-dark-400"
            >
              <Upload size={16} />
              Import Config
            </button>
          </div>
        </div>

        {/* Browser Control */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
              <Globe size={20} className="text-blue-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Browser Control</h3>
              <p className="text-xs text-gray-500">Configure built-in browser capabilities</p>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg mb-4">
            <div>
              <p className="text-sm text-white">Enable Browser Tool</p>
              <p className="text-xs text-gray-500">Allow agents to browse the web</p>
            </div>
            <input
              type="checkbox"
              checked={browser.enabled}
              onChange={e => setBrowser({ ...browser, enabled: e.target.checked })}
              className="w-5 h-5 rounded bg-dark-500 border-dark-400 text-claw-500 focus:ring-claw-500/50"
            />
          </div>

          {browser.enabled && (
            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">Browser Chrome Color</p>
                <p className="text-xs text-gray-500">Custom color for the browser window</p>
              </div>
              <div className="flex items-center gap-3">
                <input
                  type="color"
                  value={browser.color || '#000000'}
                  onChange={e => setBrowser({ ...browser, color: e.target.value })}
                  className="w-8 h-8 rounded overflow-hidden cursor-pointer border-0 p-0"
                />
                <span className="text-sm font-mono text-gray-400">{browser.color || 'Default'}</span>
              </div>
            </div>
          )}
        </div>

        {/* Web Search Config */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
              <Globe size={20} className="text-orange-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Web Search</h3>
              <p className="text-xs text-gray-500">Configure search engine APIs</p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Brave Search API Key</label>
              <input
                type="password"
                value={webConfig.brave_api_key || ''}
                onChange={e => setWebConfig({ ...webConfig, brave_api_key: e.target.value || null })}
                placeholder="BSA-..."
                className="input-base"
              />
              <p className="text-xs text-gray-500 mt-1">Required for agents to perform web searches.</p>
            </div>
          </div>
        </div>

        {/* Manager Update */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
              <ArrowUpCircle size={20} className="text-emerald-400" />
            </div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-white">Manager Update</h3>
              <p className="text-xs text-gray-500">Keep OpenClaw Manager up to date</p>
            </div>
            <span className="text-xs font-mono text-gray-500 bg-dark-600 px-2 py-1 rounded">v{appVersion}</span>
          </div>

          <div className="space-y-4">
            {/* Check for updates button */}
            {!managerUpdateAvailable && !managerUpdateDone && (
              <button
                onClick={checkManagerUpdate}
                disabled={managerChecking}
                className="w-full flex items-center gap-3 p-4 bg-dark-600 rounded-lg hover:bg-dark-500 transition-colors text-left disabled:opacity-50"
              >
                {managerChecking ? (
                  <Loader2 size={18} className="text-emerald-400 animate-spin" />
                ) : (
                  <RefreshCw size={18} className="text-emerald-400" />
                )}
                <div className="flex-1">
                  <p className="text-sm text-white">{managerChecking ? 'Checking...' : 'Check for Updates'}</p>
                  <p className="text-xs text-gray-500">Check GitHub for the latest Manager version</p>
                </div>
              </button>
            )}

            {/* Up to date message */}
            {managerCheckDone && !managerUpdateAvailable && (
              <div className="flex items-center gap-3 p-4 bg-emerald-900/20 rounded-lg border border-emerald-800/30">
                <CheckCircle size={18} className="text-emerald-400" />
                <p className="text-sm text-emerald-300">You're on the latest version!</p>
              </div>
            )}

            {/* Update available */}
            {managerUpdateAvailable && !managerUpdateDone && (
              <div className="p-4 bg-dark-600 rounded-lg space-y-3">
                <div className="flex items-center gap-2">
                  <Download size={16} className="text-emerald-400" />
                  <span className="text-sm font-medium text-white">Update available: v{managerUpdateVersion}</span>
                </div>
                {managerUpdateBody && (
                  <p className="text-xs text-gray-400 whitespace-pre-line max-h-32 overflow-y-auto">{managerUpdateBody}</p>
                )}

                {/* Download progress */}
                {managerDownloading && (
                  <div className="space-y-1">
                    <div className="w-full bg-dark-500 rounded-full h-2">
                      <div
                        className="bg-gradient-to-r from-emerald-500 to-claw-500 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${managerDownloadProgress}%` }}
                      />
                    </div>
                    <p className="text-xs text-gray-500 text-right">{managerDownloadProgress}%</p>
                  </div>
                )}

                {!managerDownloading && (
                  <button
                    onClick={downloadManagerUpdate}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-gradient-to-r from-emerald-600 to-claw-600 hover:from-emerald-500 hover:to-claw-500 text-white text-sm font-medium rounded-lg transition-all"
                  >
                    <Download size={16} />
                    Download & Install
                  </button>
                )}
              </div>
            )}

            {/* Update installed - restart */}
            {managerUpdateDone && (
              <div className="p-4 bg-emerald-900/20 rounded-lg border border-emerald-800/30 space-y-3">
                <div className="flex items-center gap-2">
                  <CheckCircle size={16} className="text-emerald-400" />
                  <span className="text-sm font-medium text-emerald-300">Update installed successfully!</span>
                </div>
                <button
                  onClick={restartApp}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-lg transition-colors"
                >
                  <RefreshCw size={16} />
                  Restart Now
                </button>
              </div>
            )}

            {/* Error message */}
            {managerUpdateError && (
              <div className="flex items-start gap-2 p-3 bg-red-900/20 rounded-lg border border-red-800/30">
                <AlertTriangle size={16} className="text-red-400 mt-0.5 shrink-0" />
                <p className="text-xs text-red-300">{managerUpdateError}</p>
              </div>
            )}
          </div>
        </div>

        {/* Danger Zone */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-red-900/30 opacity-80 hover:opacity-100 transition-opacity">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center">
              <AlertTriangle size={20} className="text-red-400" />
            </div>
            <h3 className="text-lg font-semibold text-white">Danger Zone</h3>
          </div>
          <button
            onClick={() => setShowUninstallConfirm(true)}
            className="w-full flex items-center gap-3 p-3 bg-red-950/30 rounded-lg hover:bg-red-900/40 transition-colors text-left border border-red-900/30"
          >
            <Trash2 size={18} className="text-red-400" />
            <div className="flex-1">
              <p className="text-sm text-red-300">Uninstall OpenClaw</p>
            </div>
          </button>
        </div>

        {/* Global Save Button (Floating) */}
        <div className="fixed bottom-6 right-6 z-40">
          <button
            onClick={handleSave}
            disabled={saving || saveSuccess}
            className={`shadow-xl flex items-center gap-2 px-6 py-3 rounded-full text-base font-medium transition-all ${saveSuccess
              ? 'bg-green-600 text-white hover:bg-green-700'
              : 'btn-primary'
              }`}
          >
            {saving ? (
              <Loader2 size={20} className="animate-spin" />
            ) : saveSuccess ? (
              <CheckCircle size={20} />
            ) : (
              <Save size={20} />
            )}
            {saveSuccess ? 'Saved!' : 'Save Settings'}
          </button>
        </div>
      </div>

      {/* Uninstall Modal */}
      {showUninstallConfirm && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500 max-w-md w-full shadow-2xl">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold text-white">Uninstall OpenClaw</h3>
              <button onClick={() => setShowUninstallConfirm(false)}><X size={20} className="text-gray-400 hover:text-white" /></button>
            </div>

            {!uninstallResult ? (
              <>
                <p className="text-gray-300 mb-4">Are you sure? This will <span className="text-red-400 font-semibold">permanently delete</span> the entire <code className="bg-dark-600 px-1.5 py-0.5 rounded text-red-300 text-xs">~/.openclaw</code> folder (all configs, agents, and data) and uninstall the OpenClaw CLI.</p>
                <p className="text-yellow-400/80 text-xs mb-6 flex items-center gap-2"><AlertTriangle size={14} /> This action cannot be undone.</p>
                <div className="flex gap-3">
                  <button onClick={() => setShowUninstallConfirm(false)} className="flex-1 btn-secondary">Cancel</button>
                  <button onClick={handleUninstall} disabled={uninstalling} className="flex-1 btn-primary bg-red-600 hover:bg-red-500 flex justify-center gap-2">
                    {uninstalling ? <Loader2 className="animate-spin" size={16} /> : <Trash2 size={16} />}
                    Uninstall
                  </button>
                </div>
              </>
            ) : (
              <div className={`p-4 rounded-lg bg-${uninstallResult.success ? 'green' : 'red'}-900/30 border border-${uninstallResult.success ? 'green' : 'red'}-800`}>
                <p className={`text-${uninstallResult.success ? 'green' : 'red'}-300 text-sm`}>{uninstallResult.message}</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
