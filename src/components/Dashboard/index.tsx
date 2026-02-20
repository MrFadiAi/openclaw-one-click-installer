import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { StatusCard } from './StatusCard';
import { QuickActions } from './QuickActions';
import { SystemInfo } from './SystemInfo';
import { Setup } from '../Setup';
import { api, ServiceStatus, isTauri } from '../../lib/tauri';
import { EnvironmentStatus } from '../../App';

interface DashboardProps {
  envStatus: EnvironmentStatus | null;
  onSetupComplete: () => void;
}

export function Dashboard({ envStatus, onSetupComplete }: DashboardProps) {
  const [status, setStatus] = useState<ServiceStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);

  const fetchStatus = async () => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    try {
      const result = await api.getServiceStatus();
      setStatus(result);
    } catch {
      // Handle silently
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    if (!isTauri()) return;

    const statusInterval = setInterval(fetchStatus, 3000);

    return () => {
      clearInterval(statusInterval);
    };
  }, []);

  const handleStart = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await api.startService();
      await fetchStatus();
    } catch (e) {
      console.error('Start failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleStop = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await api.stopService();
      await fetchStatus();
    } catch (e) {
      console.error('Stop failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleRestart = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await api.restartService();
      await fetchStatus();
    } catch (e) {
      console.error('Restart failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleKillAll = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await invoke<string>('kill_all_port_processes');
      await fetchStatus();
    } catch (e) {
      console.error('Kill All failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const containerVariants = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    show: { opacity: 1, y: 0 },
  };

  // Check if environment is ready
  const needsSetup = envStatus && !envStatus.ready;

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2">
      <motion.div
        variants={containerVariants}
        initial="hidden"
        animate="show"
        className="space-y-6"
      >
        {/* Environment setup wizard (only shown when needed) */}
        {needsSetup && (
          <motion.div variants={itemVariants}>
            <Setup onComplete={onSetupComplete} embedded />
          </motion.div>
        )}

        {/* Service status card */}
        <motion.div variants={itemVariants}>
          <StatusCard status={status} loading={loading} />
        </motion.div>

        {/* Quick actions */}
        <motion.div variants={itemVariants}>
          <QuickActions
            status={status}
            loading={actionLoading}
            onStart={handleStart}
            onStop={handleStop}
            onRestart={handleRestart}
            onKillAll={handleKillAll}
          />
        </motion.div>

        {/* System info */}
        <motion.div variants={itemVariants}>
          <SystemInfo />
        </motion.div>
      </motion.div>
    </div>
  );
}
