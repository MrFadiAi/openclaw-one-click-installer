import { motion } from 'framer-motion';
import {
  LayoutDashboard,
  Bot,
  MessageSquare,

  ScrollText,
  Settings,
  Blocks,
  Book,
  Users,
} from 'lucide-react';
import { PageType } from '../../App';
import clsx from 'clsx';

interface ServiceStatus {
  running: boolean;
  pid: number | null;
  port: number;
}

interface SidebarProps {
  currentPage: PageType;
  onNavigate: (page: PageType) => void;
  serviceStatus: ServiceStatus | null;
}

const menuItems: { id: PageType; label: string; icon: React.ElementType }[] = [
  { id: 'dashboard', label: 'Overview', icon: LayoutDashboard },
  { id: 'mcp', label: 'MCPs', icon: Blocks },
  { id: 'skills', label: 'Skills', icon: Book },
  { id: 'agents', label: 'Agents', icon: Users },
  { id: 'ai', label: 'AI Config', icon: Bot },
  { id: 'channels', label: 'Channels', icon: MessageSquare },

  { id: 'logs', label: 'Logs', icon: ScrollText },
  { id: 'settings', label: 'Settings', icon: Settings },
];

export function Sidebar({ currentPage, onNavigate, serviceStatus }: SidebarProps) {
  const isRunning = serviceStatus?.running ?? false;
  return (
    <aside className="w-64 bg-dark-800 border-r border-dark-600 flex flex-col">
      {/* Logo area (macOS titlebar drag) */}
      <div className="h-14 flex items-center px-6 titlebar-drag border-b border-dark-600">
        <div className="flex items-center gap-3 titlebar-no-drag">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-claw-400 to-claw-600 flex items-center justify-center">
            <span className="text-lg">🦞</span>
          </div>
          <div>
            <h1 className="text-sm font-semibold text-white">OpenClaw</h1>
            <p className="text-xs text-gray-500">Manager</p>
          </div>
        </div>
      </div>

      {/* Navigation menu */}
      <nav className="flex-1 py-4 px-3">
        <ul className="space-y-1">
          {menuItems.map((item) => {
            const isActive = currentPage === item.id;
            const Icon = item.icon;

            return (
              <li key={item.id}>
                <button
                  onClick={() => onNavigate(item.id)}
                  className={clsx(
                    'w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-all relative',
                    isActive
                      ? 'text-white bg-dark-600'
                      : 'text-gray-400 hover:text-white hover:bg-dark-700'
                  )}
                >
                  {isActive && (
                    <motion.div
                      layoutId="activeIndicator"
                      className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-claw-500 rounded-r-full"
                      transition={{ type: 'spring', stiffness: 300, damping: 30 }}
                    />
                  )}
                  <Icon size={18} className={isActive ? 'text-claw-400' : ''} />
                  <span>{item.label}</span>
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Footer info */}
      <div className="p-4 border-t border-dark-600">
        <div className="px-4 py-3 bg-dark-700 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <div className={clsx('status-dot', isRunning ? 'running' : 'stopped')} />
            <span className="text-xs text-gray-400">
              {isRunning ? 'Service Running' : 'Service Stopped'}
            </span>
          </div>
          <p className="text-xs text-gray-500">Port: {serviceStatus?.port ?? 18789}</p>
        </div>
      </div>
    </aside>
  );
}
