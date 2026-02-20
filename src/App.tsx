import React, { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { Sidebar } from './components/Layout/Sidebar';
import { Header } from './components/Layout/Header';
import { Dashboard } from './components/Dashboard';
import { AIConfig } from './components/AIConfig';
import { Channels } from './components/Channels';
import { MCP } from './components/MCP';
import { Skills } from './components/Skills';
import { Settings } from './components/Settings';

import { Logs } from './components/Logs';
import { appLogger } from './lib/logger';
import { isTauri } from './lib/tauri';
import { Download, X, Loader2, CheckCircle, AlertCircle } from 'lucide-react';

import { Agents } from './components/Agents';

export type PageType = 'dashboard' | 'mcp' | 'skills' | 'ai' | 'channels' | 'agents' | 'logs' | 'settings';

export interface EnvironmentStatus {
  node_installed: boolean;
  node_version: string | null;
  node_version_ok: boolean;
  git_installed: boolean;
  git_version: string | null;
  openclaw_installed: boolean;
  openclaw_version: string | null;
  config_dir_exists: boolean;
  ready: boolean;
  os: string;
}

interface ServiceStatus {
  running: boolean;
  pid: number | null;
  port: number;
}

interface UpdateInfo {
  update_available: boolean;
  current_version: string | null;
  latest_version: string | null;
  error: string | null;
}

interface UpdateResult {
  success: boolean;
  message: string;
  error?: string;
}

interface SecureVersionInfo {
  current_version: string;
  is_secure: boolean;
}

class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean, error: Error | null }> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    appLogger.error('ErrorBoundary caught error', { error, errorInfo });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-8 text-center">
          <AlertCircle size={48} className="mx-auto text-red-400 mb-4" />
          <h2 className="text-xl font-bold text-white mb-2">Something went wrong</h2>
          <p className="text-red-200 mb-4">{this.state.error?.message}</p>
          <button
            onClick={() => this.setState({ hasError: false })}
            className="px-4 py-2 bg-dark-700 hover:bg-dark-600 rounded-lg text-white text-sm"
          >
            Try again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

function App() {
  const [currentPage, setCurrentPage] = useState<PageType>('dashboard');
  const [isReady, setIsReady] = useState<boolean | null>(null);
  const [envStatus, setEnvStatus] = useState<EnvironmentStatus | null>(null);
  const [serviceStatus, setServiceStatus] = useState<ServiceStatus | null>(null);

  // Update related state
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateBanner, setShowUpdateBanner] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [updateResult, setUpdateResult] = useState<UpdateResult | null>(null);

  // Security check state
  const [secureVersionInfo, setSecureVersionInfo] = useState<SecureVersionInfo | null>(null);
  const [showSecurityBanner, setShowSecurityBanner] = useState(false);

  // Check environment
  const checkEnvironment = useCallback(async () => {
    if (!isTauri()) {
      appLogger.warn('Not in Tauri environment, skipping environment check');
      setIsReady(true);
      return;
    }

    appLogger.info('Starting system environment check...');
    try {
      const status = await invoke<EnvironmentStatus>('check_environment');
      appLogger.info('Environment check completed', status);
      setEnvStatus(status);
      setIsReady(true); // Always show main interface
    } catch (e) {
      appLogger.error('Environment check failed', e);
      setIsReady(true);
    }
  }, []);

  // Check for updates
  const checkUpdate = useCallback(async () => {
    if (!isTauri()) return;

    appLogger.info('Checking for OpenClaw updates...');
    try {
      const info = await invoke<UpdateInfo>('check_openclaw_update');
      appLogger.info('Update check result', info);
      setUpdateInfo(info);
      if (info.update_available) {
        setShowUpdateBanner(true);
      }
    } catch (e) {
      appLogger.error('Update check failed', e);
    }
  }, []);

  // Check security version
  const checkSecurity = useCallback(async () => {
    if (!isTauri()) return;

    appLogger.info('Checking OpenClaw version security...');
    try {
      const info = await invoke<SecureVersionInfo>('check_secure_version');
      appLogger.info('Security check result', info);
      setSecureVersionInfo(info);
      if (!info.is_secure) {
        setShowSecurityBanner(true);
      }
    } catch (e) {
      appLogger.error('Security check failed', e);
    }
  }, []);

  // Perform update
  const handleUpdate = async () => {
    setUpdating(true);
    setUpdateResult(null);
    try {
      const result = await invoke<UpdateResult>('update_openclaw');
      setUpdateResult(result);
      if (result.success) {
        // Re-check environment after successful update
        await checkEnvironment();
        // Close notification after 3 seconds
        setTimeout(() => {
          setShowUpdateBanner(false);
          setUpdateResult(null);
        }, 3000);
      }
    } catch (e) {
      setUpdateResult({
        success: false,
        message: 'Error occurred during update',
        error: String(e),
      });
    } finally {
      setUpdating(false);
    }
  };

  useEffect(() => {
    appLogger.info('🦞 App component mounted');
    checkEnvironment();
  }, [checkEnvironment]);

  // Delay update check after startup (avoid blocking startup)
  useEffect(() => {
    if (!isTauri()) return;
    const timer = setTimeout(() => {
      checkUpdate();
    }, 2000);
    return () => clearTimeout(timer);
  }, [checkUpdate]);

  // Check security after startup
  useEffect(() => {
    if (!isTauri()) return;
    const timer = setTimeout(() => {
      checkSecurity();
    }, 1000); // Check shortly after startup
    return () => clearTimeout(timer);
  }, [checkSecurity]);

  // Periodically get service status
  useEffect(() => {
    // Don't poll if not in Tauri environment
    if (!isTauri()) return;

    const fetchServiceStatus = async () => {
      try {
        const status = await invoke<ServiceStatus>('get_service_status');
        setServiceStatus(status);
      } catch {
        // Silently handle polling errors
      }
    };
    fetchServiceStatus();
    const interval = setInterval(fetchServiceStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  const handleSetupComplete = useCallback(() => {
    appLogger.info('Setup wizard completed');
    checkEnvironment(); // Re-check environment
  }, [checkEnvironment]);

  // Page navigation handler
  const handleNavigate = (page: PageType) => {
    appLogger.action('Page navigation', { from: currentPage, to: page });
    setCurrentPage(page);
  };

  const renderPage = () => {
    const pageVariants = {
      initial: { opacity: 0, x: 20 },
      animate: { opacity: 1, x: 0 },
      exit: { opacity: 0, x: -20 },
    };

    const pages: Record<PageType, JSX.Element> = {
      dashboard: <Dashboard envStatus={envStatus} onSetupComplete={handleSetupComplete} />,
      mcp: <MCP />,
      skills: <Skills />,
      ai: <AIConfig />,
      channels: <Channels />,
      agents: <Agents />,

      logs: <Logs />,
      settings: <Settings onEnvironmentChange={checkEnvironment} />,
    };

    return (
      <AnimatePresence mode="wait">
        <motion.div
          key={currentPage}
          variants={pageVariants}
          initial="initial"
          animate="animate"
          exit="exit"
          transition={{ duration: 0.2 }}
          className="h-full"
        >
          {pages[currentPage]}
        </motion.div>
      </AnimatePresence>
    );
  };

  // Checking environment
  if (isReady === null) {
    return (
      <div className="flex h-screen bg-dark-900 items-center justify-center">
        <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />
        <div className="relative z-10 text-center">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-xl bg-gradient-to-br from-brand-500 to-purple-600 mb-4 animate-pulse">
            <span className="text-3xl">🦞</span>
          </div>
          <p className="text-dark-400">Starting...</p>
        </div>
      </div>
    );
  }

  // Main interface
  return (
    <div className="flex h-screen bg-dark-900 overflow-hidden">
      {/* Background decoration */}
      <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />

      {/* Security Banner (High Priority) */}
      <AnimatePresence>
        {showSecurityBanner && secureVersionInfo && !secureVersionInfo.is_secure && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-[60] bg-gradient-to-r from-red-600 to-orange-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <AlertCircle size={20} className="text-white" />
                <div>
                  <p className="text-sm font-bold text-white">
                    Security Warning: Your OpenClaw version ({secureVersionInfo.current_version}) is insecure.
                  </p>
                  <p className="text-xs text-white/90">
                    A version &ge; 2026.1.29 is required. Please update immediately.
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowSecurityBanner(false)}
                className="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/90 hover:text-white"
              >
                <X size={16} />
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Update banner */}
      <AnimatePresence>
        {showUpdateBanner && updateInfo?.update_available && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-50 bg-gradient-to-r from-claw-600 to-purple-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3">
                {updateResult?.success ? (
                  <CheckCircle size={20} className="text-green-300" />
                ) : updateResult && !updateResult.success ? (
                  <AlertCircle size={20} className="text-red-300" />
                ) : (
                  <Download size={20} className="text-white" />
                )}
                <div>
                  {updateResult ? (
                    <p className={`text-sm font-medium ${updateResult.success ? 'text-green-100' : 'text-red-100'}`}>
                      {updateResult.message}
                    </p>
                  ) : (
                    <>
                      <p className="text-sm font-medium text-white">
                        New version available: OpenClaw {updateInfo.latest_version}
                      </p>
                      <p className="text-xs text-white/70">
                        Current version: {updateInfo.current_version}
                      </p>
                    </>
                  )}
                </div>
              </div>

              <div className="flex items-center gap-2">
                {!updateResult && (
                  <button
                    onClick={handleUpdate}
                    disabled={updating}
                    className="px-4 py-1.5 bg-white/20 hover:bg-white/30 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2 disabled:opacity-50"
                  >
                    {updating ? (
                      <>
                        <Loader2 size={14} className="animate-spin" />
                        Updating...
                      </>
                    ) : (
                      <>
                        <Download size={14} />
                        Update Now
                      </>
                    )}
                  </button>
                )}
                <button
                  onClick={() => {
                    setShowUpdateBanner(false);
                    setUpdateResult(null);
                  }}
                  className="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/70 hover:text-white"
                >
                  <X size={16} />
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Sidebar */}
      <Sidebar currentPage={currentPage} onNavigate={handleNavigate} serviceStatus={serviceStatus} />

      {/* Main content area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header (macOS drag area) */}
        <Header currentPage={currentPage} />

        {/* Page content */}
        <main className="flex-1 overflow-hidden p-6">
          <ErrorBoundary>
            {renderPage()}
          </ErrorBoundary>
        </main>
      </div>
    </div>
  );
}

export default App;
