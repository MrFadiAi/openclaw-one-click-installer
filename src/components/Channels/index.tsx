import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import {
  MessageCircle,
  Hash,
  Slack,
  MessagesSquare,
  MessageSquare,
  Check,
  X,
  Loader2,
  ChevronRight,
  Apple,
  Bell,
  Eye,
  EyeOff,
  Play,
  QrCode,
  CheckCircle,
  XCircle,
  Download,
  Package,
  AlertTriangle,
  Trash2,
  Plus,
} from 'lucide-react';
import clsx from 'clsx';

interface FeishuPluginStatus {
  installed: boolean;
  version: string | null;
  plugin_name: string | null;
}

interface ChannelConfig {
  id: string;
  channel_type: string;
  enabled: boolean;
  config: Record<string, unknown>;
}

// Channel configuration field definition
interface ChannelField {
  key: string;
  label: string;
  type: 'text' | 'password' | 'select';
  placeholder?: string;
  options?: { value: string; label: string }[];
  required?: boolean;
}

const channelInfo: Record<
  string,
  {
    name: string;
    icon: React.ReactNode;
    color: string;
    fields: ChannelField[];
    helpText?: string;
  }
> = {
  telegram: {
    name: 'Telegram',
    icon: <MessageCircle size={20} />,
    color: 'text-blue-400',
    fields: [
      { key: 'botToken', label: 'Bot Token', type: 'password', placeholder: 'Get from @BotFather', required: true },
      { key: 'userId', label: 'User ID', type: 'text', placeholder: 'Your Telegram User ID', required: true },
      {
        key: 'dmPolicy', label: 'DM Policy', type: 'select', options: [
          { value: 'pairing', label: 'Pairing Mode' },
          { value: 'open', label: 'Open Mode' },
          { value: 'disabled', label: 'Disabled' },
        ]
      },
      {
        key: 'groupPolicy', label: 'Group Policy', type: 'select', options: [
          { value: 'open', label: 'Enabled (Respond to all)' },
          { value: 'allowlist', label: 'Allowlist (Explicitly allowed only)' },
          { value: 'disabled', label: 'Disabled (Ignore all)' },
        ]
      },
      {
        key: 'streamMode', label: 'Stream Mode', type: 'select', options: [
          { value: 'partial', label: 'Partial (Default)' },
          { value: 'block', label: 'Block' },
          { value: 'off', label: 'Off' },
        ]
      },
    ],
    helpText: '1. Search @BotFather and send /newbot to get Token  2. Search @userinfobot to get User ID',
  },
  discord: {
    name: 'Discord',
    icon: <Hash size={20} />,
    color: 'text-indigo-400',
    fields: [
      { key: 'botToken', label: 'Bot Token', type: 'password', placeholder: 'Discord Bot Token', required: true },
      { key: 'testChannelId', label: 'Test Channel ID', type: 'text', placeholder: 'Channel ID for sending test messages (optional)' },
      {
        key: 'dmPolicy', label: 'DM Policy', type: 'select', options: [
          { value: 'pairing', label: 'Pairing Mode' },
          { value: 'open', label: 'Open Mode' },
          { value: 'disabled', label: 'Disabled' },
        ]
      },
    ],
    helpText: 'Get from Discord Developer Portal, enable Developer Mode to copy Channel ID',
  },
  slack: {
    name: 'Slack',
    icon: <Slack size={20} />,
    color: 'text-purple-400',
    fields: [
      { key: 'botToken', label: 'Bot Token', type: 'password', placeholder: 'xoxb-...', required: true },
      { key: 'appToken', label: 'App Token', type: 'password', placeholder: 'xapp-...' },
      { key: 'testChannelId', label: 'Test Channel ID', type: 'text', placeholder: 'Channel ID for sending test messages (optional)' },
    ],
    helpText: 'Get from Slack API dashboard, Channel ID can be copied from channel details',
  },
  feishu: {
    name: 'Feishu',
    icon: <MessagesSquare size={20} />,
    color: 'text-blue-500',
    fields: [
      { key: 'appId', label: 'App ID', type: 'text', placeholder: 'Feishu App ID', required: true },
      { key: 'appSecret', label: 'App Secret', type: 'password', placeholder: 'Feishu App Secret', required: true },
      { key: 'testChatId', label: 'Test Chat ID', type: 'text', placeholder: 'Group/User ID for sending test messages (optional)' },
      {
        key: 'connectionMode', label: 'Connection Mode', type: 'select', options: [
          { value: 'websocket', label: 'WebSocket (Recommended)' },
          { value: 'webhook', label: 'Webhook' },
        ]
      },
      {
        key: 'domain', label: 'Deployment Region', type: 'select', options: [
          { value: 'feishu', label: 'China (feishu.cn)' },
          { value: 'lark', label: 'International (larksuite.com)' },
        ]
      },
      {
        key: 'requireMention', label: 'Require @Mention', type: 'select', options: [
          { value: 'true', label: 'Yes' },
          { value: 'false', label: 'No' },
        ]
      },
    ],
    helpText: 'Get credentials from Feishu Open Platform, Chat ID can be found in group settings',
  },
  imessage: {
    name: 'iMessage',
    icon: <Apple size={20} />,
    color: 'text-green-400',
    fields: [
      {
        key: 'dmPolicy', label: 'DM Policy', type: 'select', options: [
          { value: 'pairing', label: 'Pairing Mode' },
          { value: 'open', label: 'Open Mode' },
          { value: 'disabled', label: 'Disabled' },
        ]
      },
      {
        key: 'groupPolicy', label: 'Group Policy', type: 'select', options: [
          { value: 'open', label: 'Enabled (Respond to all)' },
          { value: 'allowlist', label: 'Allowlist (Explicitly allowed only)' },
          { value: 'disabled', label: 'Disabled (Ignore all)' },
        ]
      },
    ],
    helpText: 'macOS only, requires Messages access permission',
  },
  whatsapp: {
    name: 'WhatsApp',
    icon: <MessageCircle size={20} />,
    color: 'text-green-500',
    fields: [
      {
        key: 'dmPolicy', label: 'DM Policy', type: 'select', options: [
          { value: 'pairing', label: 'Pairing Mode' },
          { value: 'open', label: 'Open Mode' },
          { value: 'disabled', label: 'Disabled' },
        ]
      },
      {
        key: 'groupPolicy', label: 'Group Policy', type: 'select', options: [
          { value: 'open', label: 'Enabled (Respond to all)' },
          { value: 'allowlist', label: 'Allowlist (Explicitly allowed only)' },
          { value: 'disabled', label: 'Disabled (Ignore all)' },
        ]
      },
    ],
    helpText: 'Requires QR code scan to login, run: openclaw channels login --channel whatsapp',
  },
  wechat: {
    name: 'WeChat',
    icon: <MessageSquare size={20} />,
    color: 'text-green-600',
    fields: [
      { key: 'appId', label: 'App ID', type: 'text', placeholder: 'WeChat Open Platform App ID' },
      { key: 'appSecret', label: 'App Secret', type: 'password', placeholder: 'WeChat Open Platform App Secret' },
    ],
    helpText: 'WeChat Official Account/Enterprise WeChat configuration',
  },
  dingtalk: {
    name: 'DingTalk',
    icon: <Bell size={20} />,
    color: 'text-blue-600',
    fields: [
      { key: 'appKey', label: 'App Key', type: 'text', placeholder: 'DingTalk App Key' },
      { key: 'appSecret', label: 'App Secret', type: 'password', placeholder: 'DingTalk App Secret' },
    ],
    helpText: 'Get from DingTalk Open Platform',
  },
};

interface TestResult {
  success: boolean;
  message: string;
  error: string | null;
}

export function Channels() {
  const [channels, setChannels] = useState<ChannelConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedChannel, setSelectedChannel] = useState<string | null>(null);
  const [configForm, setConfigForm] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [loginLoading, setLoginLoading] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [showClearConfirm, setShowClearConfirm] = useState(false);

  // OpenClaw channel access control state
  const [allowedGroups, setAllowedGroups] = useState<Record<string, boolean>>({}); // { groupId: requireMention }
  const [allowFromUsers, setAllowFromUsers] = useState<string[]>([]);     // allowFrom (DM user IDs)
  const [groupAllowFromUsers, setGroupAllowFromUsers] = useState<string[]>([]); // groupAllowFrom (group sender IDs)
  const [newGroupInput, setNewGroupInput] = useState('');
  const [newAllowFromInput, setNewAllowFromInput] = useState('');
  const [newGroupAllowFromInput, setNewGroupAllowFromInput] = useState('');

  // Feishu plugin status
  const [feishuPluginStatus, setFeishuPluginStatus] = useState<FeishuPluginStatus | null>(null);
  const [feishuPluginLoading, setFeishuPluginLoading] = useState(false);
  const [feishuPluginInstalling, setFeishuPluginInstalling] = useState(false);

  // Track which password fields are visible
  const [visiblePasswords, setVisiblePasswords] = useState<Set<string>>(new Set());

  const togglePasswordVisibility = (fieldKey: string) => {
    setVisiblePasswords((prev) => {
      const next = new Set(prev);
      if (next.has(fieldKey)) {
        next.delete(fieldKey);
      } else {
        next.add(fieldKey);
      }
      return next;
    });
  };

  // Check Feishu plugin status
  const checkFeishuPlugin = async () => {
    setFeishuPluginLoading(true);
    try {
      const status = await invoke<FeishuPluginStatus>('check_feishu_plugin');
      setFeishuPluginStatus(status);
    } catch (e) {
      console.error('Failed to check Feishu plugin:', e);
      setFeishuPluginStatus({ installed: false, version: null, plugin_name: null });
    } finally {
      setFeishuPluginLoading(false);
    }
  };

  // Install Feishu plugin
  const handleInstallFeishuPlugin = async () => {
    setFeishuPluginInstalling(true);
    try {
      const result = await invoke<string>('install_feishu_plugin');
      alert(result);
      // Refresh plugin status
      await checkFeishuPlugin();
    } catch (e) {
      alert('Installation failed: ' + e);
    } finally {
      setFeishuPluginInstalling(false);
    }
  };

  // Show clear confirmation
  const handleShowClearConfirm = () => {
    if (!selectedChannel) return;
    setShowClearConfirm(true);
  };

  // Execute clear channel config
  const handleClearConfig = async () => {
    if (!selectedChannel) return;

    const channel = channels.find((c) => c.id === selectedChannel);
    const channelName = channel ? channelInfo[channel.channel_type]?.name || channel.channel_type : selectedChannel;

    setShowClearConfirm(false);
    setClearing(true);
    try {
      await invoke('clear_channel_config', { channelId: selectedChannel });
      // Clear form
      setConfigForm({});
      // Refresh list
      await fetchChannels();
      setTestResult({
        success: true,
        message: `${channelName} configuration cleared`,
        error: null,
      });
    } catch (e) {
      setTestResult({
        success: false,
        message: 'Clear failed',
        error: String(e),
      });
    } finally {
      setClearing(false);
    }
  };

  // Quick test
  const handleQuickTest = async () => {
    if (!selectedChannel) return;

    setTesting(true);
    setTestResult(null);

    try {
      const result = await invoke<{
        success: boolean;
        channel: string;
        message: string;
        error: string | null;
      }>('test_channel', { channelType: selectedChannel });

      setTestResult({
        success: result.success,
        message: result.message,
        error: result.error,
      });
    } catch (e) {
      setTestResult({
        success: false,
        message: 'Test failed',
        error: String(e),
      });
    } finally {
      setTesting(false);
    }
  };

  // WhatsApp QR code login
  const handleWhatsAppLogin = async () => {
    setLoginLoading(true);
    try {
      // Call backend command to start WhatsApp login
      await invoke('start_channel_login', { channelType: 'whatsapp' });

      // Start polling to check login status
      const pollInterval = setInterval(async () => {
        try {
          const result = await invoke<{
            success: boolean;
            message: string;
          }>('test_channel', { channelType: 'whatsapp' });

          if (result.success) {
            clearInterval(pollInterval);
            setLoginLoading(false);
            // Refresh channel list
            await fetchChannels();
            setTestResult({
              success: true,
              message: 'WhatsApp login successful!',
              error: null,
            });
          }
        } catch {
          // Continue polling
        }
      }, 3000); // Check every 3 seconds

      // Stop polling after 60 seconds
      setTimeout(() => {
        clearInterval(pollInterval);
        setLoginLoading(false);
      }, 60000);

      alert('Please scan the QR code in the popup terminal window to complete login\n\nThe interface will update automatically after successful login');
    } catch (e) {
      alert('Failed to start login: ' + e);
      setLoginLoading(false);
    }
  };

  const fetchChannels = async () => {
    try {
      const result = await invoke<ChannelConfig[]>('get_channels_config');
      setChannels(result);
      return result;
    } catch (e) {
      console.error('Failed to get channel config:', e);
      return [];
    }
  };

  useEffect(() => {
    const init = async () => {
      try {
        const result = await fetchChannels();

        // Auto-select the first configured channel
        const configured = result.find((c) => c.enabled);
        if (configured) {
          handleChannelSelect(configured.id, result);
        }
      } finally {
        setLoading(false);
      }
    };
    init();
  }, []);

  const handleChannelSelect = (channelId: string, channelList?: ChannelConfig[]) => {
    setSelectedChannel(channelId);
    setTestResult(null); // Clear test result

    const list = channelList || channels;
    const channel = list.find((c) => c.id === channelId);

    if (channel) {
      const form: Record<string, string> = {};
      Object.entries(channel.config).forEach(([key, value]) => {
        // Handle boolean values
        if (typeof value === 'boolean') {
          form[key] = value ? 'true' : 'false';
        } else if (typeof value === 'object' && value !== null) {
          // Skip objects (handled separately: groups, allowFrom, groupAllowFrom)
        } else {
          form[key] = String(value ?? '');
        }
      });
      setConfigForm(form);

      // Load groups (object: { groupId: { requireMention: bool } })
      const groupsObj = (channel.config.groups as Record<string, { requireMention?: boolean }>) || {};
      const groupMap: Record<string, boolean> = {};
      for (const [gid, settings] of Object.entries(groupsObj)) {
        groupMap[gid] = settings?.requireMention !== false; // default true
      }
      setAllowedGroups(groupMap);

      // Load allowFrom (array of user IDs for DM)
      const allowFrom = channel.config.allowFrom as (number | string)[] || [];
      setAllowFromUsers(Array.isArray(allowFrom) ? allowFrom.map(String) : []);

      // Load groupAllowFrom (array of user IDs for groups)
      const groupAllowFrom = channel.config.groupAllowFrom as (number | string)[] || [];
      setGroupAllowFromUsers(Array.isArray(groupAllowFrom) ? groupAllowFrom.map(String) : []);

      // If Feishu channel is selected, check plugin status
      if (channel.channel_type === 'feishu') {
        checkFeishuPlugin();
      }
    } else {
      setConfigForm({});
    }
  };

  const handleSave = async () => {
    if (!selectedChannel) return;

    setSaving(true);
    try {
      const channel = channels.find((c) => c.id === selectedChannel);
      if (!channel) return;

      // Convert form values
      const config: Record<string, unknown> = {};
      Object.entries(configForm).forEach(([key, value]) => {
        if (value === 'true') {
          config[key] = true;
        } else if (value === 'false') {
          config[key] = false;
        } else if (value) {
          config[key] = value;
        }
      });

      // Save groups as object: { "groupId": { requireMention: bool } }
      if (Object.keys(allowedGroups).length > 0) {
        const groupsObj: Record<string, { requireMention: boolean }> = {};
        for (const [gid, reqMention] of Object.entries(allowedGroups)) {
          groupsObj[gid] = { requireMention: reqMention };
        }
        config['groups'] = groupsObj;
      }

      // Save allowFrom (DM user IDs) as array of numbers
      if (allowFromUsers.length > 0) {
        config['allowFrom'] = allowFromUsers.map(id => /^-?\d+$/.test(id) ? Number(id) : id);
      }

      // Save groupAllowFrom (group sender user IDs) as array of numbers
      if (groupAllowFromUsers.length > 0) {
        config['groupAllowFrom'] = groupAllowFromUsers.map(id => /^-?\d+$/.test(id) ? Number(id) : id);
      }

      await invoke('save_channel_config', {
        channel: {
          ...channel,
          config,
        },
      });

      // Refresh list
      await fetchChannels();

      alert('Channel configuration saved!');
    } catch (e) {
      console.error('Save failed:', e);
      alert('Save failed: ' + e);
    } finally {
      setSaving(false);
    }
  };

  const currentChannel = channels.find((c) => c.id === selectedChannel);
  const currentInfo = currentChannel ? channelInfo[currentChannel.channel_type] : null;

  // Check if channel has valid configuration
  const hasValidConfig = (channel: ChannelConfig) => {
    const info = channelInfo[channel.channel_type];
    if (!info) return channel.enabled;

    // Check if required fields are filled
    const requiredFields = info.fields.filter((f) => f.required);
    if (requiredFields.length === 0) return channel.enabled;

    return requiredFields.some((field) => {
      const value = channel.config[field.key];
      return value !== undefined && value !== null && value !== '';
    });
  };

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-claw-500" />
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2">
      <div className="max-w-4xl">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Channel list */}
          <div className="md:col-span-1 space-y-2">
            <h3 className="text-sm font-medium text-gray-400 mb-3 px-1">
              Message Channels
            </h3>
            {channels.map((channel) => {
              const info = channelInfo[channel.channel_type] || {
                name: channel.channel_type,
                icon: <MessageSquare size={20} />,
                color: 'text-gray-400',
                fields: [],
              };
              const isSelected = selectedChannel === channel.id;
              const isConfigured = hasValidConfig(channel);

              return (
                <button
                  key={channel.id}
                  onClick={() => handleChannelSelect(channel.id)}
                  className={clsx(
                    'w-full flex items-center gap-3 p-4 rounded-xl border transition-all',
                    isSelected
                      ? 'bg-dark-600 border-claw-500'
                      : 'bg-dark-700 border-dark-500 hover:border-dark-400'
                  )}
                >
                  <div
                    className={clsx(
                      'w-10 h-10 rounded-lg flex items-center justify-center',
                      isConfigured ? 'bg-dark-500' : 'bg-dark-600'
                    )}
                  >
                    <span className={info.color}>{info.icon}</span>
                  </div>
                  <div className="flex-1 text-left">
                    <p
                      className={clsx(
                        'text-sm font-medium',
                        isSelected ? 'text-white' : 'text-gray-300'
                      )}
                    >
                      {info.name}
                    </p>
                    <div className="flex items-center gap-2 mt-1">
                      {isConfigured ? (
                        <>
                          <Check size={12} className="text-green-400" />
                          <span className="text-xs text-green-400">Configured</span>
                        </>
                      ) : (
                        <>
                          <X size={12} className="text-gray-500" />
                          <span className="text-xs text-gray-500">Not Configured</span>
                        </>
                      )}
                    </div>
                  </div>
                  <ChevronRight
                    size={16}
                    className={isSelected ? 'text-claw-400' : 'text-gray-600'}
                  />
                </button>
              );
            })}
          </div>

          {/* Configuration panel */}
          <div className="md:col-span-2">
            {currentChannel && currentInfo ? (
              <motion.div
                key={selectedChannel}
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                className="bg-dark-700 rounded-2xl p-6 border border-dark-500"
              >
                <div className="flex items-center gap-3 mb-4">
                  <div className={clsx('w-10 h-10 rounded-lg flex items-center justify-center bg-dark-500', currentInfo.color)}>
                    {currentInfo.icon}
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-white">
                      Configure {currentInfo.name}
                    </h3>
                    {currentInfo.helpText && (
                      <p className="text-xs text-gray-500">{currentInfo.helpText}</p>
                    )}
                  </div>
                </div>

                {/* Feishu plugin status hint */}
                {currentChannel.channel_type === 'feishu' && (
                  <div className="mb-4">
                    {feishuPluginLoading ? (
                      <div className="p-4 bg-dark-600 rounded-xl border border-dark-500 flex items-center gap-3">
                        <Loader2 size={20} className="animate-spin text-gray-400" />
                        <span className="text-gray-400">Checking Feishu plugin status...</span>
                      </div>
                    ) : feishuPluginStatus?.installed ? (
                      <div className="p-4 bg-green-500/10 rounded-xl border border-green-500/30 flex items-center gap-3">
                        <Package size={20} className="text-green-400" />
                        <div className="flex-1">
                          <p className="text-green-400 font-medium">Feishu plugin installed</p>
                          <p className="text-xs text-gray-400 mt-0.5">
                            {feishuPluginStatus.plugin_name || '@m1heng-clawd/feishu'}
                            {feishuPluginStatus.version && ` v${feishuPluginStatus.version}`}
                          </p>
                        </div>
                        <CheckCircle size={16} className="text-green-400" />
                      </div>
                    ) : (
                      <div className="p-4 bg-amber-500/10 rounded-xl border border-amber-500/30">
                        <div className="flex items-start gap-3">
                          <AlertTriangle size={20} className="text-amber-400 mt-0.5" />
                          <div className="flex-1">
                            <p className="text-amber-400 font-medium">Feishu plugin installation required</p>
                            <p className="text-xs text-gray-400 mt-1">
                              Feishu channel requires the @m1heng-clawd/feishu plugin to be installed first.
                            </p>
                            <div className="mt-3 flex flex-wrap gap-2">
                              <button
                                onClick={handleInstallFeishuPlugin}
                                disabled={feishuPluginInstalling}
                                className="btn-primary flex items-center gap-2 text-sm py-2"
                              >
                                {feishuPluginInstalling ? (
                                  <Loader2 size={14} className="animate-spin" />
                                ) : (
                                  <Download size={14} />
                                )}
                                {feishuPluginInstalling ? 'Installing...' : 'One-click Install Plugin'}
                              </button>
                              <button
                                onClick={checkFeishuPlugin}
                                disabled={feishuPluginLoading}
                                className="btn-secondary flex items-center gap-2 text-sm py-2"
                              >
                                Refresh Status
                              </button>
                            </div>
                            <p className="text-xs text-gray-500 mt-2">
                              Or run manually: <code className="px-1.5 py-0.5 bg-dark-600 rounded text-gray-400">openclaw plugins install @m1heng-clawd/feishu</code>
                            </p>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                )}

                <div className="space-y-4">
                  {currentInfo.fields.map((field) => (
                    <div key={field.key}>
                      <label className="block text-sm text-gray-400 mb-2">
                        {field.label}
                        {field.required && <span className="text-red-400 ml-1">*</span>}
                        {configForm[field.key] && (
                          <span className="ml-2 text-green-500 text-xs">✓</span>
                        )}
                      </label>

                      {field.type === 'select' ? (
                        <select
                          value={configForm[field.key] || ''}
                          onChange={(e) =>
                            setConfigForm({ ...configForm, [field.key]: e.target.value })
                          }
                          className="input-base"
                        >
                          <option value="">Please select...</option>
                          {field.options?.map((opt) => (
                            <option key={opt.value} value={opt.value}>
                              {opt.label}
                            </option>
                          ))}
                        </select>
                      ) : field.type === 'password' ? (
                        <div className="relative">
                          <input
                            type={visiblePasswords.has(field.key) ? 'text' : 'password'}
                            value={configForm[field.key] || ''}
                            onChange={(e) =>
                              setConfigForm({ ...configForm, [field.key]: e.target.value })
                            }
                            placeholder={field.placeholder}
                            className="input-base pr-10"
                          />
                          <button
                            type="button"
                            onClick={() => togglePasswordVisibility(field.key)}
                            className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white transition-colors"
                            title={visiblePasswords.has(field.key) ? 'Hide' : 'Show'}
                          >
                            {visiblePasswords.has(field.key) ? (
                              <EyeOff size={18} />
                            ) : (
                              <Eye size={18} />
                            )}
                          </button>
                        </div>
                      ) : (
                        <input
                          type={field.type}
                          value={configForm[field.key] || ''}
                          onChange={(e) =>
                            setConfigForm({ ...configForm, [field.key]: e.target.value })
                          }
                          placeholder={field.placeholder}
                          className="input-base"
                        />
                      )}

                      {/* Groups UI: shown when groupPolicy is 'allowlist' */}
                      {field.key === 'groupPolicy' && configForm[field.key] === 'allowlist' && (
                        <div className="mt-3 space-y-3">
                          {/* Allowed Groups */}
                          <div className="p-4 bg-dark-600 rounded-xl border border-dark-500">
                            <label className="block text-sm text-gray-400 mb-2">Allowed Groups (Chat ID)</label>
                            <div className="flex gap-2 mb-2">
                              <input
                                type="text"
                                value={newGroupInput}
                                onChange={(e) => setNewGroupInput(e.target.value)}
                                placeholder="e.g. -1001234567890"
                                className="input-base text-sm"
                                onKeyDown={(e) => {
                                  if (e.key === 'Enter') {
                                    e.preventDefault();
                                    if (newGroupInput && !(newGroupInput in allowedGroups)) {
                                      setAllowedGroups({ ...allowedGroups, [newGroupInput]: false });
                                      setNewGroupInput('');
                                    }
                                  }
                                }}
                              />
                              <button
                                onClick={() => {
                                  if (newGroupInput && !(newGroupInput in allowedGroups)) {
                                    setAllowedGroups({ ...allowedGroups, [newGroupInput]: false });
                                    setNewGroupInput('');
                                  }
                                }}
                                className="btn-secondary p-2"
                              >
                                <Plus size={16} />
                              </button>
                            </div>
                            <div className="space-y-1 max-h-40 overflow-y-auto">
                              {Object.entries(allowedGroups).map(([id, requireMention]) => (
                                <div key={id} className="flex items-center justify-between text-sm bg-dark-500 px-3 py-1.5 rounded-lg border border-dark-400">
                                  <span className="font-mono text-gray-300">{id}</span>
                                  <div className="flex items-center gap-2">
                                    <button
                                      onClick={() => setAllowedGroups({ ...allowedGroups, [id]: !requireMention })}
                                      className={`text-xs px-2 py-0.5 rounded-full border transition-colors ${requireMention
                                          ? 'border-yellow-500/50 bg-yellow-500/10 text-yellow-400'
                                          : 'border-green-500/50 bg-green-500/10 text-green-400'
                                        }`}
                                      title={requireMention ? 'Bot only responds when @mentioned' : 'Bot responds to all messages'}
                                    >
                                      {requireMention ? '@mention required' : 'responds to all'}
                                    </button>
                                    <button
                                      onClick={() => {
                                        const next = { ...allowedGroups };
                                        delete next[id];
                                        setAllowedGroups(next);
                                      }}
                                      className="text-gray-500 hover:text-red-400"
                                    >
                                      <Trash2 size={14} />
                                    </button>
                                  </div>
                                </div>
                              ))}
                              {Object.keys(allowedGroups).length === 0 && (
                                <div className="text-xs text-gray-500 text-center py-2 italic">
                                  No groups added. Bot will ignore all groups.
                                </div>
                              )}
                            </div>
                            <p className="text-xs text-gray-500 mt-2">
                              New groups default to <code className="px-1 py-0.5 bg-dark-500 rounded">responds to all</code>. Click the badge to toggle <code className="px-1 py-0.5 bg-dark-500 rounded">requireMention</code>.
                            </p>
                          </div>

                          {/* Group Allowed Senders (groupAllowFrom) */}
                          <div className="p-4 bg-dark-600 rounded-xl border border-dark-500">
                            <label className="block text-sm text-gray-400 mb-2">Allowed Senders in Groups (User ID)</label>
                            <div className="flex gap-2 mb-2">
                              <input
                                type="text"
                                value={newGroupAllowFromInput}
                                onChange={(e) => setNewGroupAllowFromInput(e.target.value)}
                                placeholder="e.g. 123456789"
                                className="input-base text-sm"
                                onKeyDown={(e) => {
                                  if (e.key === 'Enter') {
                                    e.preventDefault();
                                    if (newGroupAllowFromInput && !groupAllowFromUsers.includes(newGroupAllowFromInput)) {
                                      setGroupAllowFromUsers([...groupAllowFromUsers, newGroupAllowFromInput]);
                                      setNewGroupAllowFromInput('');
                                    }
                                  }
                                }}
                              />
                              <button
                                onClick={() => {
                                  if (newGroupAllowFromInput && !groupAllowFromUsers.includes(newGroupAllowFromInput)) {
                                    setGroupAllowFromUsers([...groupAllowFromUsers, newGroupAllowFromInput]);
                                    setNewGroupAllowFromInput('');
                                  }
                                }}
                                className="btn-secondary p-2"
                              >
                                <Plus size={16} />
                              </button>
                            </div>
                            <div className="space-y-1 max-h-40 overflow-y-auto">
                              {groupAllowFromUsers.map(id => (
                                <div key={id} className="flex items-center justify-between text-sm bg-dark-500 px-3 py-1.5 rounded-lg border border-dark-400">
                                  <span className="font-mono text-gray-300">{id}</span>
                                  <button
                                    onClick={() => setGroupAllowFromUsers(groupAllowFromUsers.filter(u => u !== id))}
                                    className="text-gray-500 hover:text-red-400"
                                  >
                                    <Trash2 size={14} />
                                  </button>
                                </div>
                              ))}
                              {groupAllowFromUsers.length === 0 && (
                                <div className="text-xs text-gray-500 text-center py-2 italic">
                                  No senders restricted. All group members can interact.
                                </div>
                              )}
                            </div>
                            <p className="text-xs text-gray-500 mt-2">
                              Saved as <code className="px-1 py-0.5 bg-dark-500 rounded">channels.telegram.groupAllowFrom</code>. Numeric Telegram user IDs.
                            </p>
                          </div>
                        </div>
                      )}

                      {/* DM allowFrom: shown when dmPolicy is 'pairing' or 'allowlist' */}
                      {field.key === 'dmPolicy' && (configForm[field.key] === 'pairing' || configForm[field.key] === 'allowlist') && (
                        <div className="mt-3 p-4 bg-dark-600 rounded-xl border border-dark-500">
                          <label className="block text-sm text-gray-400 mb-2">Allowed DM Users (User ID)</label>
                          <div className="flex gap-2 mb-2">
                            <input
                              type="text"
                              value={newAllowFromInput}
                              onChange={(e) => setNewAllowFromInput(e.target.value)}
                              placeholder="e.g. 123456789"
                              className="input-base text-sm"
                              onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                  e.preventDefault();
                                  if (newAllowFromInput && !allowFromUsers.includes(newAllowFromInput)) {
                                    setAllowFromUsers([...allowFromUsers, newAllowFromInput]);
                                    setNewAllowFromInput('');
                                  }
                                }
                              }}
                            />
                            <button
                              onClick={() => {
                                if (newAllowFromInput && !allowFromUsers.includes(newAllowFromInput)) {
                                  setAllowFromUsers([...allowFromUsers, newAllowFromInput]);
                                  setNewAllowFromInput('');
                                }
                              }}
                              className="btn-secondary p-2"
                            >
                              <Plus size={16} />
                            </button>
                          </div>
                          <div className="space-y-1 max-h-40 overflow-y-auto">
                            {allowFromUsers.map(id => (
                              <div key={id} className="flex items-center justify-between text-sm bg-dark-500 px-3 py-1.5 rounded-lg border border-dark-400">
                                <span className="font-mono text-gray-300">{id}</span>
                                <button
                                  onClick={() => setAllowFromUsers(allowFromUsers.filter(u => u !== id))}
                                  className="text-gray-500 hover:text-red-400"
                                >
                                  <Trash2 size={14} />
                                </button>
                              </div>
                            ))}
                            {allowFromUsers.length === 0 && (
                              <div className="text-xs text-gray-500 text-center py-2 italic">
                                {configForm[field.key] === 'pairing'
                                  ? 'Users will be added automatically via pairing flow.'
                                  : 'No users allowed. Add user IDs above.'}
                              </div>
                            )}
                          </div>
                          <p className="text-xs text-gray-500 mt-2">
                            Saved as <code className="px-1 py-0.5 bg-dark-500 rounded">channels.telegram.allowFrom</code>. Numeric Telegram user IDs.
                          </p>
                        </div>
                      )}
                    </div>
                  ))}

                  {/* WhatsApp special handling: QR code login button */}
                  {currentChannel.channel_type === 'whatsapp' && (
                    <div className="p-4 bg-green-500/10 rounded-xl border border-green-500/30">
                      <div className="flex items-center gap-3 mb-3">
                        <QrCode size={24} className="text-green-400" />
                        <div>
                          <p className="text-white font-medium">QR Code Login</p>
                          <p className="text-xs text-gray-400">WhatsApp requires QR code scan to login</p>
                        </div>
                      </div>
                      <div className="flex gap-2">
                        <button
                          onClick={handleWhatsAppLogin}
                          disabled={loginLoading}
                          className="flex-1 btn-secondary flex items-center justify-center gap-2"
                        >
                          {loginLoading ? (
                            <Loader2 size={16} className="animate-spin" />
                          ) : (
                            <QrCode size={16} />
                          )}
                          {loginLoading ? 'Waiting for login...' : 'Start QR Code Login'}
                        </button>
                        <button
                          onClick={async () => {
                            await fetchChannels();
                            handleQuickTest();
                          }}
                          disabled={testing}
                          className="btn-secondary flex items-center justify-center gap-2 px-4"
                          title="Refresh Status"
                        >
                          {testing ? (
                            <Loader2 size={16} className="animate-spin" />
                          ) : (
                            <Check size={16} />
                          )}
                        </button>
                      </div>
                      <p className="text-xs text-gray-500 mt-2 text-center">
                        After successful login, click the button on the right to refresh status, or run: openclaw channels login --channel whatsapp
                      </p>
                    </div>
                  )}

                  {/* Action buttons */}
                  <div className="pt-4 border-t border-dark-500 flex flex-wrap items-center gap-3">
                    <button
                      onClick={handleSave}
                      disabled={saving}
                      className="btn-primary flex items-center gap-2"
                    >
                      {saving ? (
                        <Loader2 size={16} className="animate-spin" />
                      ) : (
                        <Check size={16} />
                      )}
                      Save Configuration
                    </button>

                    {/* Quick test button */}
                    <button
                      onClick={handleQuickTest}
                      disabled={testing}
                      className="btn-secondary flex items-center gap-2"
                    >
                      {testing ? (
                        <Loader2 size={16} className="animate-spin" />
                      ) : (
                        <Play size={16} />
                      )}
                      Quick Test
                    </button>

                    {/* Clear config button */}
                    {!showClearConfirm ? (
                      <button
                        onClick={handleShowClearConfirm}
                        disabled={clearing}
                        className="btn-secondary flex items-center gap-2 text-red-400 hover:text-red-300 hover:border-red-500/50"
                      >
                        {clearing ? (
                          <Loader2 size={16} className="animate-spin" />
                        ) : (
                          <Trash2 size={16} />
                        )}
                        Clear Configuration
                      </button>
                    ) : (
                      <div className="flex items-center gap-2 px-3 py-1.5 bg-red-500/20 rounded-lg border border-red-500/50">
                        <span className="text-sm text-red-300">Confirm clear?</span>
                        <button
                          onClick={handleClearConfig}
                          className="px-2 py-1 text-xs bg-red-500 text-white rounded hover:bg-red-600 transition-colors"
                        >
                          Confirm
                        </button>
                        <button
                          onClick={() => setShowClearConfirm(false)}
                          className="px-2 py-1 text-xs bg-dark-600 text-gray-300 rounded hover:bg-dark-500 transition-colors"
                        >
                          Cancel
                        </button>
                      </div>
                    )}
                  </div>

                  {/* Test result display */}
                  {testResult && (
                    <motion.div
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      className={clsx(
                        'mt-4 p-4 rounded-xl flex items-start gap-3',
                        testResult.success ? 'bg-green-500/10' : 'bg-red-500/10'
                      )}
                    >
                      {testResult.success ? (
                        <CheckCircle size={20} className="text-green-400 mt-0.5" />
                      ) : (
                        <XCircle size={20} className="text-red-400 mt-0.5" />
                      )}
                      <div className="flex-1">
                        <p className={clsx(
                          'font-medium',
                          testResult.success ? 'text-green-400' : 'text-red-400'
                        )}>
                          {testResult.success ? 'Test successful' : 'Test failed'}
                        </p>
                        <p className="text-sm text-gray-400 mt-1">{testResult.message}</p>
                        {testResult.error && (
                          <p className="text-xs text-red-300 mt-2 whitespace-pre-wrap">
                            {testResult.error}
                          </p>
                        )}
                      </div>
                    </motion.div>
                  )}
                </div>
              </motion.div>
            ) : (
              <div className="h-full flex items-center justify-center text-gray-500">
                <p>Select a channel on the left to configure</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
