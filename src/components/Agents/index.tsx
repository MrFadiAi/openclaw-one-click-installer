import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import {
    Users,
    Plus,
    Trash2,
    Loader2,
    Pencil,
    Save,
    X,
    AlertCircle,
    ArrowRight,
    MessageSquare,
    GitMerge,
    Copy,
    Zap,
    CheckCircle2,
    ChevronRight,
    ChevronDown,
    FileText,
    Bot,
    Sparkles
} from 'lucide-react';
import { appLogger } from '../../lib/logger';

// Types corresponding to Rust backend
interface AgentInfo {
    id: string;
    workspace: string | null;
    agent_dir: string | null;
    model: string | null;
    sandbox: boolean | null;
    heartbeat: string | null;
}

interface MatchRule {
    channel: string | null;
    account_id: string | null;
    peer: any | null;
}

interface AgentBinding {
    agent_id: string;
    match_rule: MatchRule;
}

interface AgentsConfigResponse {
    agents: AgentInfo[];
    bindings: AgentBinding[];
}

interface TelegramAccount {
    id: string;
    token?: string;
    groups?: Record<string, any>;
    exclusive_topics?: string[];
    primary?: boolean;
}

interface RoutingTestResult {
    matched: boolean;
    agent_id: string;
    agent_dir?: string;
    model?: string;
    system_prompt_preview?: string;
    message?: string;
}

export function Agents() {
    const [loading, setLoading] = useState(true);
    const [agents, setAgents] = useState<AgentInfo[]>([]);
    const [bindings, setBindings] = useState<AgentBinding[]>([]);
    const [error, setError] = useState<string | null>(null);
    const [openclawHomeDir, setOpenclawHomeDir] = useState<string>('');
    const [telegramAccounts, setTelegramAccounts] = useState<TelegramAccount[]>([]);

    // Dialog states
    const [showAgentDialog, setShowAgentDialog] = useState(false);
    const [editingAgent, setEditingAgent] = useState<AgentInfo | null>(null);
    const [showBindingDialog, setShowBindingDialog] = useState(false);
    const [showWizardDialog, setShowWizardDialog] = useState(false);
    const [saving, setSaving] = useState(false);
    const [showRoutingFlow, setShowRoutingFlow] = useState(false);

    // System prompt state
    const [systemPrompt, setSystemPrompt] = useState('');
    const [loadingPrompt, setLoadingPrompt] = useState(false);

    // Routing test state
    const [testResult, setTestResult] = useState<RoutingTestResult | null>(null);
    const [testingAccount, setTestingAccount] = useState<string | null>(null);

    // Form states
    const [agentForm, setAgentForm] = useState<AgentInfo>({
        id: '',
        workspace: null,
        agent_dir: null,
        model: null,
        sandbox: null,
        heartbeat: null
    });

    const [bindingForm, setBindingForm] = useState<AgentBinding>({
        agent_id: '',
        match_rule: {
            channel: null,
            account_id: null,
            peer: null
        }
    });

    // Wizard form state
    const [wizardStep, setWizardStep] = useState(0);
    const [wizardForm, setWizardForm] = useState({
        botAccountId: '',
        agentId: '',
        systemPrompt: '',
        model: '',
    });

    const fetchData = async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await invoke<AgentsConfigResponse>('get_agents_config');
            setAgents(data.agents);
            setBindings(data.bindings);
        } catch (e) {
            setError(String(e));
            appLogger.error('Failed to fetch agents config', e);
        } finally {
            setLoading(false);
        }
    };

    const fetchAccounts = async () => {
        try {
            const accts = await invoke<TelegramAccount[]>('get_telegram_accounts');
            setTelegramAccounts(accts);
        } catch { /* ignore */ }
    };

    useEffect(() => {
        fetchData();
        invoke<string>('get_openclaw_home_dir').then(dir => setOpenclawHomeDir(dir)).catch(() => { });
        fetchAccounts();
    }, []);

    // Load system prompt when editing an agent
    const loadSystemPrompt = async (agentId: string, workspace: string | null) => {
        setLoadingPrompt(true);
        try {
            const prompt = await invoke<string>('get_agent_system_prompt', {
                agentId,
                workspace: workspace || null
            });
            setSystemPrompt(prompt);
        } catch {
            setSystemPrompt('');
        } finally {
            setLoadingPrompt(false);
        }
    };

    const handleSaveAgent = async () => {
        if (!agentForm.id) return;
        setSaving(true);
        try {
            await invoke('save_agent', { agent: agentForm });

            // Save system prompt if provided
            if (systemPrompt.trim()) {
                await invoke('save_agent_system_prompt', {
                    agentId: agentForm.id,
                    workspace: agentForm.workspace || null,
                    content: systemPrompt
                });
            }

            setShowAgentDialog(false);
            fetchData();
        } catch (e) {
            setError(String(e));
        } finally {
            setSaving(false);
        }
    };

    const handleDeleteAgent = async (id: string) => {
        if (!confirm(`Delete agent ${id}?`)) return;
        try {
            await invoke('delete_agent', { agentId: id });
            fetchData();
        } catch (e) {
            setError(String(e));
        }
    };

    const handleSaveBinding = async () => {
        if (!bindingForm.agent_id) return;
        setSaving(true);
        try {
            await invoke('save_agent_binding', { binding: bindingForm });
            setShowBindingDialog(false);
            fetchData();
        } catch (e) {
            setError(String(e));
        } finally {
            setSaving(false);
        }
    };

    const handleDeleteBinding = async (index: number) => {
        if (!confirm('Delete this binding rule?')) return;
        try {
            await invoke('delete_agent_binding', { index });
            fetchData();
        } catch (e) {
            setError(String(e));
        }
    };

    // Clone agent handler
    const handleCloneAgent = (agent: AgentInfo) => {
        setEditingAgent(null);
        setAgentForm({
            id: `${agent.id}_copy`,
            workspace: agent.workspace,
            agent_dir: agent.agent_dir ? `${agent.agent_dir}_copy` : null,
            model: agent.model,
            sandbox: agent.sandbox,
            heartbeat: agent.heartbeat
        });
        setSystemPrompt('');
        // Load original system prompt for cloning
        loadSystemPrompt(agent.id, agent.workspace);
        setShowAgentDialog(true);
    };

    // Test routing handler
    const handleTestRouting = async (accountId: string) => {
        setTestingAccount(accountId);
        try {
            const result = await invoke<RoutingTestResult>('test_agent_routing', { accountId });
            setTestResult(result);
        } catch (e) {
            setError(String(e));
        } finally {
            setTestingAccount(null);
        }
    };

    // Wizard submit handler
    const handleWizardSubmit = async () => {
        setSaving(true);
        try {
            // 1. Save agent
            // Auto-configure isolated workspace
            const agentWorkspace = openclawHomeDir
                ? `${openclawHomeDir}/agents/${wizardForm.agentId}`.replace(/\\/g, '/')
                : null;

            const agent: AgentInfo = {
                id: wizardForm.agentId,
                workspace: agentWorkspace,
                agent_dir: `agents/${wizardForm.agentId}`, // Reverted to explicit directory
                model: wizardForm.model || null,
                sandbox: null,
                heartbeat: null
            };
            await invoke('save_agent', { agent });

            // 2. Save system prompt if provided
            if (wizardForm.systemPrompt.trim()) {
                await invoke('save_agent_system_prompt', {
                    agentId: wizardForm.agentId,
                    workspace: openclawHomeDir || null,
                    content: wizardForm.systemPrompt
                });
            }

            // Note: save_agent auto-creates binding if matching account exists

            setShowWizardDialog(false);
            setWizardStep(0);
            setWizardStep(0);
            setWizardForm({ botAccountId: '', agentId: '', systemPrompt: '', model: '' });
            fetchData();
        } catch (e) {
            setError(String(e));
        } finally {
            setSaving(false);
        }
    };

    if (loading && !agents.length) {
        return (
            <div className="flex items-center justify-center h-full">
                <Loader2 className="animate-spin text-claw-400" size={32} />
            </div>
        );
    }

    return (
        <div className="h-full overflow-y-auto scroll-container pr-2 space-y-8">

            {/* Visual Routing Diagram */}
            {bindings.length > 0 && (
                <section>
                    <button
                        onClick={() => setShowRoutingFlow(!showRoutingFlow)}
                        className="w-full flex items-center justify-between mb-4 group cursor-pointer"
                    >
                        <h2 className="text-xl font-semibold text-white flex items-center gap-2">
                            <Sparkles className="text-amber-400" size={24} />
                            Routing Flow
                        </h2>
                        <div className="flex items-center gap-2 text-gray-500 group-hover:text-gray-300 transition-colors">
                            <span className="text-xs">{showRoutingFlow ? 'Hide' : 'Show'}</span>
                            <ChevronDown size={16} className={`transition-transform ${showRoutingFlow ? 'rotate-180' : ''}`} />
                        </div>
                    </button>
                    <AnimatePresence>
                        {showRoutingFlow && (
                            <motion.div
                                initial={{ height: 0, opacity: 0 }}
                                animate={{ height: 'auto', opacity: 1 }}
                                exit={{ height: 0, opacity: 0 }}
                                className="overflow-hidden"
                            >
                                <div className="bg-dark-700 rounded-xl border border-dark-600 p-5 overflow-x-auto">
                                    <div className="flex flex-col gap-3">
                                        {bindings.map((binding, idx) => {
                                            const agent = agents.find(a => a.id === binding.agent_id);
                                            return (
                                                <div key={idx} className="flex items-center gap-0 text-sm">
                                                    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-blue-500/10 border border-blue-500/30 text-blue-300 min-w-[120px]">
                                                        <Bot size={14} />
                                                        <span className="font-medium">{binding.match_rule?.account_id || 'Any'}</span>
                                                    </div>
                                                    <ChevronRight size={16} className="text-gray-600 mx-1 flex-shrink-0" />
                                                    <div className="px-3 py-2 rounded-lg bg-dark-600 border border-dark-500 text-gray-400 text-xs">
                                                        {binding.match_rule?.channel || 'any'}
                                                    </div>
                                                    <ChevronRight size={16} className="text-gray-600 mx-1 flex-shrink-0" />
                                                    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-claw-500/10 border border-claw-500/30 text-claw-300 min-w-[120px]">
                                                        <Users size={14} />
                                                        <span className="font-medium">{binding.agent_id}</span>
                                                    </div>
                                                    <ChevronRight size={16} className="text-gray-600 mx-1 flex-shrink-0" />
                                                    <div className="px-3 py-2 rounded-lg bg-purple-500/10 border border-purple-500/30 text-purple-300 text-xs max-w-[200px] truncate">
                                                        {agent?.model || 'Default Model'}
                                                    </div>
                                                </div>
                                            );
                                        })}
                                    </div>
                                </div>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </section>
            )}

            {/* Agents Section */}
            <section>
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h2 className="text-xl font-semibold text-white flex items-center gap-2">
                            <Users className="text-claw-400" size={24} />
                            Agents
                        </h2>
                        <p className="text-sm text-gray-500">Manage agent definitions and overrides</p>
                    </div>
                    <div className="flex gap-2">
                        <button
                            onClick={() => {
                                setWizardStep(0);
                                setWizardForm({
                                    botAccountId: telegramAccounts[0]?.id || '',
                                    agentId: '',
                                    systemPrompt: '',
                                    model: '',
                                });
                                setShowWizardDialog(true);
                            }}
                            className="btn-secondary flex items-center gap-2"
                        >
                            <Zap size={16} />
                            Quick Setup
                        </button>
                        <button
                            onClick={() => {
                                setEditingAgent(null);
                                setAgentForm({ id: '', workspace: openclawHomeDir || null, agent_dir: null, model: null, sandbox: null, heartbeat: null });
                                setSystemPrompt('');
                                setShowAgentDialog(true);
                            }}
                            className="btn-primary flex items-center gap-2"
                        >
                            <Plus size={16} />
                            Add Agent
                        </button>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {agents.length === 0 ? (
                        <div className="col-span-full p-8 text-center text-gray-500 bg-dark-700/50 rounded-xl border border-dashed border-dark-600">
                            No agents configured. Create one to get started.
                        </div>
                    ) : (
                        agents.map(agent => (
                            <div key={agent.id} className="bg-dark-700 rounded-xl p-4 border border-dark-600 hover:border-claw-500/30 transition-colors group">
                                <div className="flex justify-between items-start mb-3">
                                    <div className="flex items-center gap-2">
                                        <div className="w-8 h-8 rounded-lg bg-claw-500/20 flex items-center justify-center text-claw-400 font-bold">
                                            {agent.id.charAt(0).toUpperCase()}
                                        </div>
                                        <div>
                                            <h3 className="font-medium text-white">{agent.id}</h3>
                                            {agent.sandbox && <span className="text-xs text-amber-400 bg-amber-500/10 px-1.5 rounded">Sandbox</span>}
                                        </div>
                                    </div>
                                    <div className="opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                                        <button
                                            onClick={() => handleCloneAgent(agent)}
                                            className="p-1.5 hover:bg-dark-600 rounded text-gray-400 hover:text-blue-400"
                                            title="Clone Agent"
                                        >
                                            <Copy size={14} />
                                        </button>
                                        <button
                                            onClick={() => {
                                                setEditingAgent(agent);
                                                setAgentForm(agent);
                                                setSystemPrompt('');
                                                loadSystemPrompt(agent.id, agent.workspace);
                                                setShowAgentDialog(true);
                                            }}
                                            className="p-1.5 hover:bg-dark-600 rounded text-gray-400 hover:text-white"
                                            title="Edit Agent"
                                        >
                                            <Pencil size={14} />
                                        </button>
                                        <button
                                            onClick={() => handleDeleteAgent(agent.id)}
                                            className="p-1.5 hover:bg-dark-600 rounded text-gray-400 hover:text-red-400"
                                            title="Delete Agent"
                                        >
                                            <Trash2 size={14} />
                                        </button>
                                    </div>
                                </div>

                                <div className="space-y-2 text-sm text-gray-400">
                                    {agent.agent_dir && (
                                        <div className="flex items-center gap-2" title="Agent Directory">
                                            <div className="text-xs px-1.5 py-0.5 bg-dark-600 rounded border border-dark-500 font-mono text-gray-400">
                                                ./{agent.agent_dir}
                                            </div>
                                        </div>
                                    )}
                                    {agent.model && (
                                        <div className="flex items-center gap-2" title="Model Override">
                                            <MessageSquare size={14} />
                                            <span className="truncate">{agent.model}</span>
                                        </div>
                                    )}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </section>

            {/* Bindings Section */}
            <section>
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h2 className="text-xl font-semibold text-white flex items-center gap-2">
                            <GitMerge className="text-purple-400" size={24} />
                            Routing Rules
                        </h2>
                        <p className="text-sm text-gray-500">Route incoming messages to specific agents</p>
                    </div>
                    <button
                        onClick={() => {
                            setBindingForm({
                                agent_id: agents[0]?.id || '',
                                match_rule: { channel: 'telegram', account_id: telegramAccounts[0]?.id || null, peer: null }
                            });
                            setShowBindingDialog(true);
                        }}
                        disabled={agents.length === 0}
                        className="btn-secondary flex items-center gap-2"
                    >
                        <Plus size={16} />
                        Add Rule
                    </button>
                </div>

                <div className="bg-dark-700 rounded-xl border border-dark-600 overflow-hidden">
                    <table className="w-full text-left text-sm">
                        <thead className="bg-dark-800 text-gray-400">
                            <tr>
                                <th className="px-4 py-3 font-medium">If Matches...</th>
                                <th className="px-4 py-3 font-medium">Route To Agent</th>
                                <th className="px-4 py-3 text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-dark-600">
                            {bindings.length === 0 ? (
                                <tr>
                                    <td colSpan={3} className="px-4 py-8 text-center text-gray-500">
                                        No routing rules configured. Messages will use the default agent.
                                    </td>
                                </tr>
                            ) : (
                                bindings.map((binding, idx) => (
                                    <tr key={idx} className="hover:bg-dark-600/50 transition-colors">
                                        <td className="px-4 py-3">
                                            <div className="flex flex-wrap gap-2">
                                                {binding.match_rule?.channel && (
                                                    <span className="px-2 py-1 rounded bg-blue-500/20 text-blue-300 text-xs border border-blue-500/30">
                                                        Channel: {binding.match_rule.channel}
                                                    </span>
                                                )}
                                                {binding.match_rule?.account_id && (
                                                    <span className="px-2 py-1 rounded bg-green-500/20 text-green-300 text-xs border border-green-500/30">
                                                        Account: {binding.match_rule.account_id}
                                                    </span>
                                                )}
                                                {!binding.match_rule?.channel && !binding.match_rule?.account_id && !binding.match_rule?.peer && (
                                                    <span className="text-gray-500 italic">Catch-all</span>
                                                )}
                                            </div>
                                        </td>
                                        <td className="px-4 py-3">
                                            <div className="flex items-center gap-2 text-white font-medium">
                                                <ArrowRight size={14} className="text-gray-500" />
                                                {binding.agent_id}
                                            </div>
                                        </td>
                                        <td className="px-4 py-3 text-right">
                                            <div className="flex items-center justify-end gap-1">
                                                <button
                                                    onClick={() => handleTestRouting(binding.match_rule?.account_id || binding.agent_id)}
                                                    disabled={testingAccount === (binding.match_rule?.account_id || binding.agent_id)}
                                                    className="p-1.5 hover:bg-dark-500 rounded text-gray-400 hover:text-green-400 transition-colors"
                                                    title="Test Routing"
                                                >
                                                    {testingAccount === (binding.match_rule?.account_id || binding.agent_id)
                                                        ? <Loader2 className="animate-spin" size={14} />
                                                        : <Zap size={14} />
                                                    }
                                                </button>
                                                <button
                                                    onClick={() => handleDeleteBinding(idx)}
                                                    className="p-1.5 hover:bg-dark-500 rounded text-gray-400 hover:text-red-400 transition-colors"
                                                    title="Delete Rule"
                                                >
                                                    <Trash2 size={14} />
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            </section>

            {/* Test Routing Result */}
            <AnimatePresence>
                {testResult && (
                    <motion.div
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -10 }}
                        className="fixed bottom-4 right-4 z-40 max-w-md"
                    >
                        <div className={`rounded-xl border shadow-2xl p-4 ${testResult.matched ? 'bg-dark-800 border-green-500/30' : 'bg-dark-800 border-amber-500/30'}`}>
                            <div className="flex items-center justify-between mb-3">
                                <div className="flex items-center gap-2">
                                    {testResult.matched
                                        ? <CheckCircle2 size={18} className="text-green-400" />
                                        : <AlertCircle size={18} className="text-amber-400" />
                                    }
                                    <span className="text-sm font-semibold text-white">Routing Test Result</span>
                                </div>
                                <button onClick={() => setTestResult(null)} className="text-gray-500 hover:text-white"><X size={14} /></button>
                            </div>
                            <div className="space-y-2 text-sm">
                                <div className="flex items-center gap-2">
                                    <span className="text-gray-400">Agent:</span>
                                    <span className="text-white font-medium">{testResult.agent_id}</span>
                                </div>
                                {testResult.model && (
                                    <div className="flex items-center gap-2">
                                        <span className="text-gray-400">Model:</span>
                                        <span className="text-purple-300">{testResult.model}</span>
                                    </div>
                                )}
                                {testResult.system_prompt_preview && (
                                    <div>
                                        <span className="text-gray-400 text-xs">Personality (SOUL.md):</span>
                                        <div className="mt-1 p-2 bg-dark-700 rounded text-xs text-gray-300 font-mono max-h-24 overflow-auto">
                                            {testResult.system_prompt_preview}
                                        </div>
                                    </div>
                                )}
                                {testResult.message && (
                                    <p className="text-amber-300 text-xs">{testResult.message}</p>
                                )}
                            </div>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Agent Dialog */}
            <AnimatePresence>
                {showAgentDialog && (
                    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" onClick={() => setShowAgentDialog(false)}>
                        <motion.div
                            initial={{ scale: 0.95, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            exit={{ scale: 0.95, opacity: 0 }}
                            className="bg-dark-800 rounded-xl border border-dark-600 w-full max-w-lg overflow-hidden max-h-[90vh] flex flex-col"
                            onClick={e => e.stopPropagation()}
                        >
                            <div className="px-6 py-4 border-b border-dark-600 flex justify-between items-center flex-shrink-0">
                                <h3 className="text-lg font-semibold text-white">
                                    {editingAgent ? 'Edit Agent' : 'Add New Agent'}
                                </h3>
                                <button onClick={() => setShowAgentDialog(false)} className="text-gray-500 hover:text-white"><X size={20} /></button>
                            </div>

                            <div className="p-6 space-y-4 overflow-y-auto">
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Agent ID *</label>
                                    <input
                                        type="text"
                                        value={agentForm.id}
                                        onChange={e => setAgentForm({ ...agentForm, id: e.target.value })}
                                        disabled={!!editingAgent}
                                        className="input-base"
                                        placeholder="e.g. coder"
                                    />
                                </div>
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Workspace Path (Optional)</label>
                                    <input
                                        type="text"
                                        value={agentForm.workspace || ''}
                                        onChange={e => setAgentForm({ ...agentForm, workspace: e.target.value || null })}
                                        className="input-base"
                                        placeholder={openclawHomeDir || '/path/to/workspace'}
                                    />
                                    <p className="text-xs text-gray-500 mt-1">Default: <code className="text-gray-400">{openclawHomeDir || '~/.openclaw'}</code></p>
                                </div>
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Agent Directory (Optional)</label>
                                    <input
                                        type="text"
                                        value={agentForm.agent_dir || ''}
                                        onChange={e => setAgentForm({ ...agentForm, agent_dir: e.target.value || null })}
                                        className="input-base"
                                        placeholder="e.g. agents/coder"
                                    />
                                    <p className="text-xs text-gray-500 mt-1">Subdirectory for agent-specific files. Relative to workspace.</p>
                                </div>
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Model Override (Optional)</label>
                                    <input
                                        type="text"
                                        value={agentForm.model || ''}
                                        onChange={e => setAgentForm({ ...agentForm, model: e.target.value || null })}
                                        className="input-base"
                                        placeholder="e.g. glm/glm-5"
                                    />
                                </div>

                                {/* System Prompt Editor (actually SOUL.md in OpenClaw) */}
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1 flex items-center gap-2">
                                        <FileText size={14} />
                                        Personality (SOUL.md)
                                    </label>
                                    {loadingPrompt ? (
                                        <div className="flex items-center gap-2 text-gray-500 text-sm py-4">
                                            <Loader2 className="animate-spin" size={14} /> Loading...
                                        </div>
                                    ) : (
                                        <textarea
                                            value={systemPrompt}
                                            onChange={e => setSystemPrompt(e.target.value)}
                                            className="input-base font-mono text-sm"
                                            rows={6}
                                            placeholder="You are a coding expert specializing in..."
                                        />
                                    )}
                                    <p className="text-xs text-gray-500 mt-1">Defines the agent's personality. Saved to <code className="text-gray-400">agents/{agentForm.id || '...'}/SOUL.md</code></p>
                                </div>

                                <div className="flex items-center gap-2 pt-2">
                                    <input
                                        type="checkbox"
                                        id="sandbox"
                                        checked={agentForm.sandbox || false}
                                        onChange={e => setAgentForm({ ...agentForm, sandbox: e.target.checked })}
                                        className="w-4 h-4 rounded bg-dark-600 border-dark-500 text-claw-500 focus:ring-claw-500/50"
                                    />
                                    <label htmlFor="sandbox" className="text-sm text-gray-300 select-none">Enable Sandbox</label>
                                </div>
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Heartbeat Interval (Optional)</label>
                                    <input
                                        type="text"
                                        value={agentForm.heartbeat || ''}
                                        onChange={e => setAgentForm({ ...agentForm, heartbeat: e.target.value || null })}
                                        className="input-base"
                                        placeholder='e.g. "1h" or "0m" to disable'
                                    />
                                </div>
                            </div>

                            <div className="px-6 py-4 border-t border-dark-600 flex justify-end gap-3 flex-shrink-0">
                                <button onClick={() => setShowAgentDialog(false)} className="btn-secondary">Cancel</button>
                                <button
                                    onClick={handleSaveAgent}
                                    disabled={saving || !agentForm.id}
                                    className="btn-primary flex items-center gap-2"
                                >
                                    {saving ? <Loader2 className="animate-spin" size={16} /> : <Save size={16} />}
                                    Save Agent
                                </button>
                            </div>
                        </motion.div>
                    </div>
                )}
            </AnimatePresence>

            {/* Binding Dialog */}
            <AnimatePresence>
                {showBindingDialog && (
                    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" onClick={() => setShowBindingDialog(false)}>
                        <motion.div
                            initial={{ scale: 0.95, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            exit={{ scale: 0.95, opacity: 0 }}
                            className="bg-dark-800 rounded-xl border border-dark-600 w-full max-w-md overflow-hidden"
                            onClick={e => e.stopPropagation()}
                        >
                            <div className="px-6 py-4 border-b border-dark-600 flex justify-between items-center">
                                <h3 className="text-lg font-semibold text-white">Add Routing Rule</h3>
                                <button onClick={() => setShowBindingDialog(false)} className="text-gray-500 hover:text-white"><X size={20} /></button>
                            </div>

                            <div className="p-6 space-y-4">
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Route To Agent *</label>
                                    <select
                                        value={bindingForm.agent_id}
                                        onChange={e => setBindingForm({ ...bindingForm, agent_id: e.target.value })}
                                        className="input-base"
                                    >
                                        {agents.map(a => <option key={a.id} value={a.id}>{a.id}</option>)}
                                    </select>
                                </div>

                                <div className="pt-2 border-t border-dark-600">
                                    <p className="text-xs text-gray-500 mb-3 uppercase font-semibold">Match Criteria</p>
                                    <div className="space-y-3">
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1">Channel</label>
                                            <input
                                                type="text"
                                                value={bindingForm.match_rule.channel || 'telegram'}
                                                readOnly
                                                className="input-base bg-dark-700 text-gray-400 cursor-not-allowed"
                                            />
                                            <p className="text-xs text-gray-500 mt-1">Currently only Telegram supports multi-agent routing.</p>
                                        </div>
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1">Bot Account *</label>
                                            <select
                                                value={bindingForm.match_rule.account_id || ''}
                                                onChange={e => setBindingForm({
                                                    ...bindingForm,
                                                    match_rule: { ...bindingForm.match_rule, account_id: e.target.value || null }
                                                })}
                                                className="input-base"
                                            >
                                                <option value="">Select a bot account...</option>
                                                {telegramAccounts.map(acct => (
                                                    <option key={acct.id} value={acct.id}>{acct.id}</option>
                                                ))}
                                            </select>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div className="px-6 py-4 border-t border-dark-600 flex justify-end gap-3">
                                <button onClick={() => setShowBindingDialog(false)} className="btn-secondary">Cancel</button>
                                <button
                                    onClick={handleSaveBinding}
                                    disabled={saving || !bindingForm.agent_id}
                                    className="btn-primary flex items-center gap-2"
                                >
                                    {saving ? <Loader2 className="animate-spin" size={16} /> : <Plus size={16} />}
                                    Add Rule
                                </button>
                            </div>
                        </motion.div>
                    </div>
                )}
            </AnimatePresence>

            {/* Quick Setup Wizard Dialog */}
            <AnimatePresence>
                {showWizardDialog && (
                    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" onClick={() => setShowWizardDialog(false)}>
                        <motion.div
                            initial={{ scale: 0.95, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            exit={{ scale: 0.95, opacity: 0 }}
                            className="bg-dark-800 rounded-xl border border-dark-600 w-full max-w-lg overflow-hidden"
                            onClick={e => e.stopPropagation()}
                        >
                            <div className="px-6 py-4 border-b border-dark-600 flex justify-between items-center">
                                <div className="flex items-center gap-3">
                                    <Zap className="text-amber-400" size={20} />
                                    <h3 className="text-lg font-semibold text-white">Quick Agent Setup</h3>
                                </div>
                                <button onClick={() => setShowWizardDialog(false)} className="text-gray-500 hover:text-white"><X size={20} /></button>
                            </div>

                            {/* Step indicators */}
                            <div className="px-6 pt-4 flex gap-2">
                                {['Bot Account', 'Agent Config', 'Personality'].map((label, i) => (
                                    <div key={i} className="flex-1">
                                        <div className={`h-1 rounded-full transition-colors ${i <= wizardStep ? 'bg-claw-500' : 'bg-dark-600'}`} />
                                        <p className={`text-xs mt-1 ${i <= wizardStep ? 'text-claw-400' : 'text-gray-600'}`}>{label}</p>
                                    </div>
                                ))}
                            </div>

                            <div className="p-6 space-y-4 min-h-[220px]">
                                {wizardStep === 0 && (
                                    <div className="space-y-4">
                                        <p className="text-sm text-gray-300">Select the Telegram bot account to link with a new agent.</p>
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1">Bot Account</label>
                                            <select
                                                value={wizardForm.botAccountId}
                                                onChange={e => {
                                                    const val = e.target.value;
                                                    setWizardForm({ ...wizardForm, botAccountId: val, agentId: val });
                                                }}
                                                className="input-base"
                                            >
                                                <option value="">Select a bot...</option>
                                                {telegramAccounts.map(acct => (
                                                    <option key={acct.id} value={acct.id}>{acct.id}</option>
                                                ))}
                                            </select>
                                        </div>
                                    </div>
                                )}

                                {wizardStep === 1 && (
                                    <div className="space-y-4">
                                        <p className="text-sm text-gray-300">Configure the agent that will handle messages for <span className="text-claw-400 font-medium">{wizardForm.botAccountId}</span>.</p>
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1">Agent ID</label>
                                            <input
                                                type="text"
                                                value={wizardForm.agentId}
                                                onChange={e => setWizardForm({ ...wizardForm, agentId: e.target.value })}
                                                className="input-base"
                                                placeholder="e.g. coder"
                                            />
                                            <p className="text-xs text-gray-500 mt-1">Will be saved to <code className="text-gray-400">agents/{wizardForm.agentId || '...'}/</code></p>
                                        </div>
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1">Model Override (Optional)</label>
                                            <input
                                                type="text"
                                                value={wizardForm.model}
                                                onChange={e => setWizardForm({ ...wizardForm, model: e.target.value })}
                                                className="input-base"
                                                placeholder="Leave empty for default model"
                                            />
                                        </div>
                                    </div>
                                )}

                                {wizardStep === 2 && (
                                    <div className="space-y-4">
                                        <p className="text-sm text-gray-300">Define the personality for <span className="text-claw-400 font-medium">{wizardForm.agentId}</span>.</p>
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1 flex items-center gap-2">
                                                <FileText size={14} />
                                                Personality (SOUL.md)
                                            </label>
                                            <textarea
                                                value={wizardForm.systemPrompt}
                                                onChange={e => setWizardForm({ ...wizardForm, systemPrompt: e.target.value })}
                                                className="input-base font-mono text-sm"
                                                rows={6}
                                                placeholder="You are a coding expert. You help users write clean, efficient code..."
                                            />
                                        </div>
                                    </div>
                                )}
                            </div>

                            <div className="px-6 py-4 border-t border-dark-600 flex justify-between">
                                <button
                                    onClick={() => wizardStep === 0 ? setShowWizardDialog(false) : setWizardStep(wizardStep - 1)}
                                    className="btn-secondary"
                                >
                                    {wizardStep === 0 ? 'Cancel' : 'Back'}
                                </button>
                                {wizardStep < 2 ? (
                                    <button
                                        onClick={() => setWizardStep(wizardStep + 1)}
                                        disabled={wizardStep === 0 ? !wizardForm.botAccountId : !wizardForm.agentId}
                                        className="btn-primary flex items-center gap-2"
                                    >
                                        Next
                                        <ChevronRight size={16} />
                                    </button>
                                ) : (
                                    <button
                                        onClick={handleWizardSubmit}
                                        disabled={saving || !wizardForm.agentId}
                                        className="btn-primary flex items-center gap-2"
                                    >
                                        {saving ? <Loader2 className="animate-spin" size={16} /> : <Zap size={16} />}
                                        Create Agent & Binding
                                    </button>
                                )}
                            </div>
                        </motion.div>
                    </div>
                )}
            </AnimatePresence>

            {error && (
                <div className="fixed bottom-4 right-4 bg-red-500 text-white px-4 py-2 rounded-lg shadow-lg flex items-center gap-2 animate-in slide-in-from-bottom-2 z-50">
                    <AlertCircle size={18} />
                    {error}
                    <button onClick={() => setError(null)} className="ml-2 hover:bg-white/20 p-1 rounded"><X size={14} /></button>
                </div>
            )}
        </div>
    );
}
