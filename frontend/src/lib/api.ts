const API_BASE = '/api';

export interface SystemResources {
  cpu_percent: number;
  memory_percent: number;
  memory_used: number;
  memory_total: number;
  uptime_seconds: number;
}

export interface TaskHistory {
  id: string;
  script_name: string;
  started_at: string;
  finished_at: string | null;
  exit_code: number | null;
  output: string | null;
}

export interface QuickAction {
  id: string;
  name: string;
  script_path: string;
  icon: string | null;
  display_order: number;
}

export interface Setting {
  key: string;
  value: string;
}

export const api = {
  health: async (): Promise<{ status: string }> => {
    const res = await fetch(`${API_BASE}/health`);
    return res.json();
  },

  getResources: async (): Promise<SystemResources> => {
    const res = await fetch(`${API_BASE}/resources`);
    return res.json();
  },

  listScripts: async (): Promise<string[]> => {
    const res = await fetch(`${API_BASE}/scripts`);
    return res.json();
  },

  getSettings: async (): Promise<{ settings: Setting[] }> => {
    const res = await fetch(`${API_BASE}/settings`);
    return res.json();
  },

  updateSetting: async (key: string, value: string): Promise<void> => {
    await fetch(`${API_BASE}/settings/${key}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ value }),
    });
  },

  getHistory: async (): Promise<TaskHistory[]> => {
    const res = await fetch(`${API_BASE}/history`);
    return res.json();
  },

  getQuickActions: async (): Promise<QuickAction[]> => {
    const res = await fetch(`${API_BASE}/quick-actions`);
    return res.json();
  },

  createQuickAction: async (action: Omit<QuickAction, 'id'>): Promise<QuickAction> => {
    const res = await fetch(`${API_BASE}/quick-actions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(action),
    });
    return res.json();
  },

  deleteQuickAction: async (id: string): Promise<void> => {
    await fetch(`${API_BASE}/quick-actions/${id}`, {
      method: 'DELETE',
    });
  },
};
