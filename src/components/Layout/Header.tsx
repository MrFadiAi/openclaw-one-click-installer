import { useState } from 'react';
import { PageType } from '../../App';
import { RefreshCw, ExternalLink, Loader2 } from 'lucide-react';
import { open } from '@tauri-apps/plugin-shell';
import { invoke } from '@tauri-apps/api/core';

interface HeaderProps {
  currentPage: PageType;
}

const pageTitles: Record<PageType, { title: string; description: string }> = {
  dashboard: { title: 'Overview', description: 'Service status, logs and quick actions' },
  mcp: { title: 'MCP Servers', description: 'Manage Model Context Protocol servers' },
  skills: { title: 'Skills', description: 'Manage OpenClaw skills' },
  ai: { title: 'AI Model Configuration', description: 'Configure AI providers and models' },
  channels: { title: 'Message Channels', description: 'Configure Telegram, Discord, Lark, etc.' },
  agents: { title: 'Agent Routing', description: 'Manage agents and binding rules' },

  logs: { title: 'Application Logs', description: 'View Manager application console logs' },
  settings: { title: 'Settings', description: 'Identity configuration and advanced options' },
};

export function Header({ currentPage }: HeaderProps) {
  const { title, description } = pageTitles[currentPage];
  const [opening, setOpening] = useState(false);

  const handleOpenDashboard = async () => {
    setOpening(true);
    try {
      // Get Dashboard URL with token (will auto-generate if no token exists)
      const url = await invoke<string>('get_dashboard_url');
      await open(url);
    } catch (e) {
      console.error('Failed to open Dashboard:', e);
      // Fallback: use window.open (without token)
      window.open('http://localhost:18789', '_blank');
    } finally {
      setOpening(false);
    }
  };

  return (
    <header className="h-14 bg-dark-800/50 border-b border-dark-600 flex items-center justify-between px-6 titlebar-drag backdrop-blur-sm">
      {/* Left side: Page title */}
      <div className="titlebar-no-drag">
        <h2 className="text-lg font-semibold text-white">{title}</h2>
        <p className="text-xs text-gray-500">{description}</p>
      </div>

      {/* Right side: Action buttons */}
      <div className="flex items-center gap-2 titlebar-no-drag">
        <button
          onClick={() => window.location.reload()}
          className="icon-button text-gray-400 hover:text-white"
          title="Refresh"
        >
          <RefreshCw size={16} />
        </button>
        <button
          onClick={handleOpenDashboard}
          disabled={opening}
          className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-dark-600 hover:bg-dark-500 text-sm text-gray-300 hover:text-white transition-colors disabled:opacity-50"
          title="Open Web Dashboard"
        >
          {opening ? <Loader2 size={14} className="animate-spin" /> : <ExternalLink size={14} />}
          <span>Dashboard</span>
        </button>
      </div>
    </header>
  );
}
