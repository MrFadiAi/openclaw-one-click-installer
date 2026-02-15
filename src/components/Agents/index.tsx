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
    Hash,
    GitMerge
} from 'lucide-react';
import { appLogger } from '../../lib/logger';

// Types corresponding to Rust backend
interface AgentInfo {
    id: string;
    workspace: string | null;
    agent_dir: string | null;
    model: string | null;
    sandbox: boolean | null;
}

interface MatchRule {
    channel: string | null;
    account_id: string | null;
    peer: any | null; // Supports string (legacy) or object { kind: 'group', id: '...' }
}

interface AgentBinding {
    agent_id: string;
    match_rule: MatchRule;
}

interface AgentsConfigResponse {
    agents: AgentInfo[];
    bindings: AgentBinding[];
}

export function Agents() {
    const [loading, setLoading] = useState(true);
    const [agents, setAgents] = useState<AgentInfo[]>([]);
    const [bindings, setBindings] = useState<AgentBinding[]>([]);
    const [error, setError] = useState<string | null>(null);

    // Dialog states
    const [showAgentDialog, setShowAgentDialog] = useState(false);
    const [editingAgent, setEditingAgent] = useState<AgentInfo | null>(null);
    const [showBindingDialog, setShowBindingDialog] = useState(false);
    const [saving, setSaving] = useState(false);

    // Form states
    const [agentForm, setAgentForm] = useState<AgentInfo>({
        id: '',
        workspace: null,
        agent_dir: null,
        model: null,
        sandbox: null
    });

    const [bindingForm, setBindingForm] = useState<AgentBinding>({
        agent_id: '',
        match_rule: {
            channel: null,
            account_id: null,
            peer: null
        }
    });

    // Peer Type State for Binding Form
    const [peerType, setPeerType] = useState<'any' | 'user' | 'group'>('any');
    const [peerId, setPeerId] = useState('');

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

    useEffect(() => {
        fetchData();
    }, []);

    const handleSaveAgent = async () => {
        if (!agentForm.id) return;
        setSaving(true);
        try {
            await invoke('save_agent', { agent: agentForm });
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

    if (loading && !agents.length) {
        return (
            <div className="flex items-center justify-center h-full">
                <Loader2 className="animate-spin text-claw-400" size={32} />
            </div>
        );
    }

    return (
        <div className="h-full overflow-y-auto scroll-container pr-2 space-y-8">

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
                    <button
                        onClick={() => {
                            setEditingAgent(null);
                            setAgentForm({ id: '', workspace: null, agent_dir: null, model: null, sandbox: null });
                            setShowAgentDialog(true);
                        }}
                        className="btn-primary flex items-center gap-2"
                    >
                        <Plus size={16} />
                        Add Agent
                    </button>
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
                                    <div className="opacity-0 group-hover:opacity-100 transition-opacity flex gap-2">
                                        <button
                                            onClick={() => {
                                                setEditingAgent(agent);
                                                setAgentForm(agent);
                                                setShowAgentDialog(true);
                                            }}
                                            className="p-1.5 hover:bg-dark-600 rounded text-gray-400 hover:text-white"
                                        >
                                            <Pencil size={14} />
                                        </button>
                                        <button
                                            onClick={() => handleDeleteAgent(agent.id)}
                                            className="p-1.5 hover:bg-dark-600 rounded text-gray-400 hover:text-red-400"
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
                                    {agent.workspace && (
                                        <div className="flex items-center gap-2" title="Workspace Override">
                                            <Hash size={14} />
                                            <span className="truncate">{agent.workspace}</span>
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
                                match_rule: { channel: null, account_id: null, peer: null }
                            });
                            setPeerType('any');
                            setPeerId('');
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
                                                {binding.match_rule?.peer && (
                                                    <span className="px-2 py-1 rounded bg-amber-500/20 text-amber-300 text-xs border border-amber-500/30">
                                                        {typeof binding.match_rule.peer === 'object' && binding.match_rule.peer.kind === 'group'
                                                            ? `Group: ${binding.match_rule.peer.id}`
                                                            : `Peer: ${binding.match_rule.peer}`}
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
                                            <button
                                                onClick={() => handleDeleteBinding(idx)}
                                                className="p-1.5 hover:bg-dark-500 rounded text-gray-400 hover:text-red-400 transition-colors"
                                            >
                                                <Trash2 size={14} />
                                            </button>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            </section>

            {/* Agent Dialog */}
            <AnimatePresence>
                {showAgentDialog && (
                    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" onClick={() => setShowAgentDialog(false)}>
                        <motion.div
                            initial={{ scale: 0.95, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            exit={{ scale: 0.95, opacity: 0 }}
                            className="bg-dark-800 rounded-xl border border-dark-600 w-full max-w-md overflow-hidden"
                            onClick={e => e.stopPropagation()}
                        >
                            <div className="px-6 py-4 border-b border-dark-600 flex justify-between items-center">
                                <h3 className="text-lg font-semibold text-white">
                                    {editingAgent ? 'Edit Agent' : 'Add New Agent'}
                                </h3>
                                <button onClick={() => setShowAgentDialog(false)} className="text-gray-500 hover:text-white"><X size={20} /></button>
                            </div>

                            <div className="p-6 space-y-4">
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
                                        placeholder="/path/to/workspace"
                                    />
                                </div>
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Agent Directory (Optional)</label>
                                    <input
                                        type="text"
                                        value={agentForm.agent_dir || ''}
                                        onChange={e => setAgentForm({ ...agentForm, agent_dir: e.target.value || null })}
                                        className="input-base"
                                        placeholder="e.g. agents/investing"
                                    />
                                </div>
                                <div>
                                    <label className="block text-sm text-gray-400 mb-1">Model Override (Optional)</label>
                                    <input
                                        type="text"
                                        value={agentForm.model || ''}
                                        onChange={e => setAgentForm({ ...agentForm, model: e.target.value || null })}
                                        className="input-base"
                                        placeholder="provider/model-id"
                                    />
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
                            </div>

                            <div className="px-6 py-4 border-t border-dark-600 flex justify-end gap-3">
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
                                    <p className="text-xs text-gray-500 mb-3 uppercase font-semibold">Match Criteria (Leave empty to ignore)</p>

                                    <div className="space-y-3">
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1">Channel (e.g. whatsapp)</label>
                                            <input
                                                type="text"
                                                value={bindingForm.match_rule.channel || ''}
                                                onChange={e => setBindingForm({
                                                    ...bindingForm,
                                                    match_rule: { ...bindingForm.match_rule, channel: e.target.value || null }
                                                })}
                                                className="input-base"
                                                placeholder="Any channel"
                                            />
                                        </div>
                                        <div>
                                            <label className="block text-sm text-gray-400 mb-1">Account ID (e.g. email or phone)</label>
                                            <input
                                                type="text"
                                                value={bindingForm.match_rule.account_id || ''}
                                                onChange={e => setBindingForm({
                                                    ...bindingForm,
                                                    match_rule: { ...bindingForm.match_rule, account_id: e.target.value || null }
                                                })}
                                                className="input-base"
                                                placeholder="Any account"
                                            />
                                        </div>

                                        {/* Peer Type Selection */}
                                        <div className="space-y-2">
                                            <label className="block text-sm text-gray-400">Peer Match</label>
                                            <div className="flex gap-2 mb-2">
                                                <button
                                                    onClick={() => { setPeerType('any'); setBindingForm({ ...bindingForm, match_rule: { ...bindingForm.match_rule, peer: null } }); }}
                                                    className={`px-3 py-1.5 text-xs rounded border ${peerType === 'any' ? 'bg-claw-500/20 border-claw-500 text-claw-400' : 'bg-dark-600 border-dark-500 text-gray-400'}`}
                                                >
                                                    Any
                                                </button>
                                                <button
                                                    onClick={() => setPeerType('user')}
                                                    className={`px-3 py-1.5 text-xs rounded border ${peerType === 'user' ? 'bg-claw-500/20 border-claw-500 text-claw-400' : 'bg-dark-600 border-dark-500 text-gray-400'}`}
                                                >
                                                    User ID
                                                </button>
                                                <button
                                                    onClick={() => setPeerType('group')}
                                                    className={`px-3 py-1.5 text-xs rounded border ${peerType === 'group' ? 'bg-claw-500/20 border-claw-500 text-claw-400' : 'bg-dark-600 border-dark-500 text-gray-400'}`}
                                                >
                                                    Group ID
                                                </button>
                                            </div>

                                            {peerType !== 'any' && (
                                                <input
                                                    type="text"
                                                    value={peerId}
                                                    onChange={e => {
                                                        const val = e.target.value;
                                                        setPeerId(val);
                                                        if (peerType === 'user') {
                                                            setBindingForm({ ...bindingForm, match_rule: { ...bindingForm.match_rule, peer: val || null } });
                                                        } else {
                                                            setBindingForm({ ...bindingForm, match_rule: { ...bindingForm.match_rule, peer: val ? { kind: 'group', id: val } : null } });
                                                        }
                                                    }}
                                                    className="input-base"
                                                    placeholder={peerType === 'group' ? "e.g. -100123456789" : "e.g. user_123"}
                                                />
                                            )}
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

            {error && (
                <div className="fixed bottom-4 right-4 bg-red-500 text-white px-4 py-2 rounded-lg shadow-lg flex items-center gap-2 animate-in slide-in-from-bottom-2">
                    <AlertCircle size={18} />
                    {error}
                    <button onClick={() => setError(null)} className="ml-2 hover:bg-white/20 p-1 rounded"><X size={14} /></button>
                </div>
            )}
        </div>
    );
}
