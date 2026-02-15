import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  User,
  Shield,
  Save,
  Loader2,
  FolderOpen,
  FileCode,
  Trash2,
  AlertTriangle,
  X,
  Activity,
  Globe,
  FileText
} from 'lucide-react';
import clsx from 'clsx';
import { appLogger } from '../../lib/logger';

interface InstallResult {
  success: boolean;
  message: string;
  error?: string;
}

interface SettingsProps {
  onEnvironmentChange?: () => void;
}

// Config Types
interface HeartbeatConfig {
  every: string | null;
  target: string | null;
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

interface BrowserConfig {
  enabled: boolean;
  color: string | null;
}

interface WebConfig {
  brave_api_key: string | null;
}

export function Settings({ onEnvironmentChange }: SettingsProps) {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  // Existing states
  const [identity, setIdentity] = useState({
    botName: 'Clawd',
    userName: 'Master',
    timezone: 'Asia/Shanghai',
  });
  const [showUninstallConfirm, setShowUninstallConfirm] = useState(false);
  const [uninstalling, setUninstalling] = useState(false);
  const [uninstallResult, setUninstallResult] = useState<InstallResult | null>(null);

  // New Feature States
  const [heartbeat, setHeartbeat] = useState<HeartbeatConfig>({ every: null, target: null });
  const [compaction, setCompaction] = useState<CompactionConfig>({ enabled: false, threshold: null, context_pruning: false, max_context_messages: null });
  const [workspace, setWorkspace] = useState<WorkspaceConfig>({ workspace: null, timezone: null, time_format: null, skip_bootstrap: false, bootstrap_max_chars: null });
  const [browser, setBrowser] = useState<BrowserConfig>({ enabled: true, color: null });
  const [webConfig, setWebConfig] = useState<WebConfig>({ brave_api_key: null });

  // Personality Editor State
  const [selectedFile, setSelectedFile] = useState<'AGENTS.md' | 'SOUL.md' | 'TOOLS.md'>('AGENTS.md');
  const [fileContent, setFileContent] = useState('');
  const [fileLoading, setFileLoading] = useState(false);

  // Load initial data
  useEffect(() => {
    const loadConfig = async () => {
      setLoading(true);
      try {
        const [hb, cmp, ws, br, web] = await Promise.all([
          invoke<HeartbeatConfig>('get_heartbeat_config'),
          invoke<CompactionConfig>('get_compaction_config'),
          invoke<WorkspaceConfig>('get_workspace_config'),
          invoke<BrowserConfig>('get_browser_config'),
          invoke<WebConfig>('get_web_config'),
        ]);
        setHeartbeat(hb);
        setCompaction(cmp);
        setWorkspace(ws);
        setBrowser(br);
        setWebConfig(web);
      } catch (e) {
        appLogger.error('Failed to load settings', e);
      } finally {
        setLoading(false);
      }
    };
    loadConfig();
  }, []);

  // Load personality file content when tab changes
  useEffect(() => {
    const loadFile = async () => {
      setFileLoading(true);
      try {
        const content = await invoke<string>('get_personality_file', { filename: selectedFile });
        setFileContent(content);
      } catch (e) {
        appLogger.error(`Failed to load ${selectedFile}`, e);
        setFileContent('');
      } finally {
        setFileLoading(false);
      }
    };
    loadFile();
  }, [selectedFile]);

  const handleSave = async () => {
    setSaving(true);
    try {
      // Save all configurations in parallel
      await Promise.all([
        invoke('save_heartbeat_config', { every: heartbeat.every, target: heartbeat.target }),
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
        invoke('save_browser_config', { enabled: browser.enabled, color: browser.color }),
        invoke('save_web_config', { braveApiKey: webConfig.brave_api_key }),
        invoke('save_personality_file', { filename: selectedFile, content: fileContent })
      ]);

      // Show success feedback
      const btn = document.getElementById('save-btn');
      if (btn) {
        // const originalText = btn.innerText; // Unused
        btn.innerHTML = '<span class="flex items-center gap-2"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg> Saved!</span>';
        setTimeout(() => {
          btn.innerHTML = '<span class="flex items-center gap-2"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path><polyline points="17 21 17 13 7 13 7 21"></polyline><polyline points="7 3 7 8 15 8"></polyline></svg> Save Settings</span>';
        }, 2000);
      }
    } catch (e) {
      console.error('Failed to save:', e);
      alert('Failed to save settings: ' + String(e));
    } finally {
      setSaving(false);
    }
  };

  const openConfigDir = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      const home = await invoke<{ config_dir: string }>('get_system_info');
      await open(home.config_dir);
    } catch (e) {
      console.error('Failed to open directory:', e);
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

        {/* Identity Configuration */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-claw-500/20 flex items-center justify-center">
              <User size={20} className="text-claw-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Identity Configuration</h3>
              <p className="text-xs text-gray-500">Set the AI assistant's name and how it addresses you</p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">
                AI Assistant Name
              </label>
              <input
                type="text"
                value={identity.botName}
                onChange={(e) =>
                  setIdentity({ ...identity, botName: e.target.value })
                }
                placeholder="Clawd"
                className="input-base"
              />
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-2">
                Your Name
              </label>
              <input
                type="text"
                value={identity.userName}
                onChange={(e) =>
                  setIdentity({ ...identity, userName: e.target.value })
                }
                placeholder="Master"
                className="input-base"
              />
            </div>
          </div>
        </div>

        {/* Security Settings */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center">
              <Shield size={20} className="text-amber-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Security Settings</h3>
              <p className="text-xs text-gray-500">Permissions and access control</p>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">Enable Whitelist</p>
                <p className="text-xs text-gray-500">Only allow whitelisted users to access</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" />
                <div className="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
              </label>
            </div>

            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">File Access Permission</p>
                <p className="text-xs text-gray-500">Allow AI to read and write local files</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" />
                <div className="w-11 h-6 bg-dark-500 peer-focus:ring-2 peer-focus:ring-claw-500/50 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-claw-500"></div>
              </label>
            </div>
          </div>
        </div>

        {/* Heartbeat & Compaction */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center">
              <Activity size={20} className="text-red-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Heartbeat & Compaction</h3>
              <p className="text-xs text-gray-500">Manage agent lifecycle and memory optimization</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <h4 className="text-sm font-medium text-gray-300 border-b border-dark-600 pb-2">Heartbeat</h4>
              <div>
                <label className="block text-sm text-gray-400 mb-2">Interval (Every)</label>
                <input
                  type="text"
                  value={heartbeat.every || ''}
                  onChange={e => setHeartbeat({ ...heartbeat, every: e.target.value || null })}
                  placeholder="e.g. 30m, 1h"
                  className="input-base"
                />
                <p className="text-xs text-gray-500 mt-1">Leave empty to disable heartbeat.</p>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-2">Target Channel</label>
                <select
                  value={heartbeat.target || ''}
                  onChange={e => setHeartbeat({ ...heartbeat, target: e.target.value || null })}
                  className="input-base"
                >
                  <option value="">None / Last Active</option>
                  <option value="whatsapp">WhatsApp</option>
                  <option value="telegram">Telegram</option>
                  <option value="discord">Discord</option>
                  <option value="slack">Slack</option>
                </select>
              </div>
            </div>

            <div className="space-y-4">
              <h4 className="text-sm font-medium text-gray-300 border-b border-dark-600 pb-2">Memory</h4>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-400">Enable Compaction</span>
                  <input
                    type="checkbox"
                    checked={compaction.enabled}
                    onChange={e => setCompaction({ ...compaction, enabled: e.target.checked })}
                    className="w-4 h-4 rounded bg-dark-600 border-dark-500 text-claw-500 focus:ring-claw-500/50"
                  />
                </div>
                {compaction.enabled && (
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">Threshold (Tokens)</label>
                    <input
                      type="number"
                      value={compaction.threshold || ''}
                      onChange={e => setCompaction({ ...compaction, threshold: parseInt(e.target.value) || null })}
                      placeholder="e.g. 8000"
                      className="input-base text-sm py-1"
                    />
                  </div>
                )}

                <div className="flex items-center justify-between pt-2 border-t border-dark-600">
                  <span className="text-sm text-gray-400">Context Pruning</span>
                  <input
                    type="checkbox"
                    checked={compaction.context_pruning}
                    onChange={e => setCompaction({ ...compaction, context_pruning: e.target.checked })}
                    className="w-4 h-4 rounded bg-dark-600 border-dark-500 text-claw-500 focus:ring-claw-500/50"
                  />
                </div>
                {compaction.context_pruning && (
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">Max Messages</label>
                    <input
                      type="number"
                      value={compaction.max_context_messages || ''}
                      onChange={e => setCompaction({ ...compaction, max_context_messages: parseInt(e.target.value) || null })}
                      placeholder="e.g. 50"
                      className="input-base text-sm py-1"
                    />
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Workspace & Personality */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
              <User size={20} className="text-purple-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Workspace & Personality</h3>
              <p className="text-xs text-gray-500">Define agent identity and workspace settings</p>
            </div>
          </div>

          <div className="space-y-4 mb-6">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Workspace Path</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={workspace.workspace || ''}
                  onChange={e => setWorkspace({ ...workspace, workspace: e.target.value || null })}
                  placeholder="/path/to/custom/workspace"
                  className="input-base flex-1"
                />
                <button title="Browse" className="btn-secondary px-3"><FolderOpen size={16} /></button>
              </div>
              <p className="text-xs text-gray-500 mt-1">Leave empty to use default (~/.openclaw)</p>
            </div>

            <div className="grid grid-cols-2 gap-4">
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

            <div className="flex items-center gap-4 pt-2">
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="skipBootstrap"
                  checked={workspace.skip_bootstrap}
                  onChange={e => setWorkspace({ ...workspace, skip_bootstrap: e.target.checked })}
                />
                <label htmlFor="skipBootstrap" className="text-sm text-gray-400">Skip Bootstrap</label>
              </div>
              <div>
                <input
                  type="number"
                  value={workspace.bootstrap_max_chars || ''}
                  onChange={e => setWorkspace({ ...workspace, bootstrap_max_chars: parseInt(e.target.value) || null })}
                  placeholder="Max Chars"
                  className="input-base py-1 px-2 w-24 text-sm inline-block mr-2"
                />
                <span className="text-xs text-gray-500">Bootstrap Max Chars</span>
              </div>
            </div>
          </div>

          {/* Configuration File Editor */}
          <div className="border-t border-dark-600 pt-4">
            <div className="flex gap-1 mb-2 bg-dark-800 p-1 rounded-lg w-fit">
              {(['AGENTS.md', 'SOUL.md', 'TOOLS.md'] as const).map(file => (
                <button
                  key={file}
                  onClick={() => setSelectedFile(file)}
                  className={clsx(
                    'px-3 py-1.5 rounded-md text-xs font-medium transition-colors flex items-center gap-2',
                    selectedFile === file ? 'bg-dark-600 text-white shadow-sm' : 'text-gray-400 hover:text-white'
                  )}
                >
                  <FileText size={14} />
                  {file}
                </button>
              ))}
            </div>

            <div className="relative">
              {fileLoading && (
                <div className="absolute inset-0 bg-dark-800/80 backdrop-blur-sm flex items-center justify-center z-10 rounded-lg">
                  <Loader2 className="animate-spin text-claw-400" size={24} />
                </div>
              )}
              <textarea
                value={fileContent}
                onChange={e => setFileContent(e.target.value)}
                className="w-full h-64 bg-dark-800 border border-dark-600 rounded-lg p-4 font-mono text-sm text-gray-300 focus:ring-2 focus:ring-claw-500/50 focus:border-claw-500 outline-none resize-none"
                placeholder={`Content for ${selectedFile}...`}
                spellCheck={false}
              />
            </div>
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

        {/* Advanced Settings */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
              <FileCode size={20} className="text-purple-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Advanced Settings</h3>
              <p className="text-xs text-gray-500">Configuration files and directories</p>
            </div>
          </div>

          <div className="space-y-3">
            <button
              onClick={openConfigDir}
              className="w-full flex items-center gap-3 p-4 bg-dark-600 rounded-lg hover:bg-dark-500 transition-colors text-left"
            >
              <FolderOpen size={18} className="text-gray-400" />
              <div className="flex-1">
                <p className="text-sm text-white">Open Configuration Directory</p>
                <p className="text-xs text-gray-500">~/.openclaw</p>
              </div>
            </button>
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
            id="save-btn"
            onClick={handleSave}
            disabled={saving}
            className="btn-primary shadow-xl flex items-center gap-2 px-6 py-3 rounded-full text-base font-medium"
          >
            {saving ? (
              <Loader2 size={20} className="animate-spin" />
            ) : (
              <Save size={20} />
            )}
            Save Settings
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
                <p className="text-gray-300 mb-6">Are you sure? This will remove the CLI tool but keep your config.</p>
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
