const API_BASE = '/api';

export interface CpuCore {
  name: string;
  usage: number;
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  total_space: number;
  available_space: number;
  used_space: number;
  usage_percent: number;
}

export interface NetworkInterface {
  name: string;
  received: number;
  transmitted: number;
}

export interface SystemResources {
  cpu_percent: number;
  cpu_cores: CpuCore[];
  memory_percent: number;
  memory_used: number;
  memory_total: number;
  swap_used: number;
  swap_total: number;
  uptime_seconds: number;
  disks: DiskInfo[];
  network: NetworkInterface[];
  process_count: number;
  system_name: string | null;
  kernel_version: string | null;
  os_version: string | null;
  host_name: string | null;
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

async function handleResponse<T>(res: Response, endpoint: string): Promise<T> {
  if (!res.ok) {
    const errorText = await res.text().catch(() => 'Unknown error');
    console.error(`API Error [${endpoint}]:`, {
      status: res.status,
      statusText: res.statusText,
      body: errorText,
      url: res.url,
    });
    throw new Error(`API request failed: ${res.status} ${res.statusText} - ${errorText}`);
  }
  
  try {
    return await res.json();
  } catch (err) {
    console.error(`JSON Parse Error [${endpoint}]:`, err);
    throw new Error(`Failed to parse JSON response from ${endpoint}`);
  }
}

export const api = {
  health: async (): Promise<{ status: string }> => {
    console.log('API: Fetching health status');
    const res = await fetch(`${API_BASE}/health`);
    return handleResponse(res, '/health');
  },

  getResources: async (): Promise<SystemResources> => {
    console.log('API: Fetching system resources');
    const res = await fetch(`${API_BASE}/resources`);
    return handleResponse(res, '/resources');
  },

  listScripts: async (): Promise<string[]> => {
    console.log('API: Listing scripts');
    const res = await fetch(`${API_BASE}/scripts`);
    return handleResponse(res, '/scripts');
  },

  getSettings: async (): Promise<{ settings: Setting[] }> => {
    console.log('API: Fetching settings');
    const res = await fetch(`${API_BASE}/settings`);
    return handleResponse(res, '/settings');
  },

  updateSetting: async (key: string, value: string): Promise<void> => {
    console.log('API: Updating setting', key);
    const res = await fetch(`${API_BASE}/settings/${key}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ value }),
    });
    await handleResponse(res, `/settings/${key}`);
  },

  getHistory: async (): Promise<TaskHistory[]> => {
    console.log('API: Fetching history');
    const res = await fetch(`${API_BASE}/history`);
    return handleResponse(res, '/history');
  },

  getQuickActions: async (): Promise<QuickAction[]> => {
    console.log('API: Fetching quick actions');
    const res = await fetch(`${API_BASE}/quick-actions`);
    return handleResponse(res, '/quick-actions');
  },

  createQuickAction: async (action: Omit<QuickAction, 'id'>): Promise<QuickAction> => {
    console.log('API: Creating quick action', action);
    const res = await fetch(`${API_BASE}/quick-actions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(action),
    });
    return handleResponse(res, '/quick-actions');
  },

  deleteQuickAction: async (id: string): Promise<void> => {
    console.log('API: Deleting quick action', id);
    const res = await fetch(`${API_BASE}/quick-actions/${id}`, {
      method: 'DELETE',
    });
    await handleResponse(res, `/quick-actions/${id}`);
  },
};
