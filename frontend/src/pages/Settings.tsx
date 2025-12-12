import { useEffect, useState } from 'react';
import type { QuickAction, Setting } from '../lib/api';
import { api } from '../lib/api';
import { Plus, Trash2 } from 'lucide-react';

export function Settings() {
  const [_settings, setSettings] = useState<Setting[]>([]);
  const [quickActions, setQuickActions] = useState<QuickAction[]>([]);
  const [scriptsDir, setScriptsDir] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [newActionName, setNewActionName] = useState('');
  const [newActionScript, setNewActionScript] = useState('');
  const [scripts, setScripts] = useState<string[]>([]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [settingsData, actions, scriptsList] = await Promise.all([
          api.getSettings(),
          api.getQuickActions(),
          api.listScripts(),
        ]);
        
        setSettings(settingsData.settings);
        setQuickActions(actions);
        setScripts(scriptsList);
        
        const scriptsDirSetting = settingsData.settings.find((s) => s.key === 'scripts_dir');
        if (scriptsDirSetting) {
          setScriptsDir(scriptsDirSetting.value);
        }
      } catch (err) {
        console.error('Failed to fetch settings:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  const handleSaveScriptsDir = async () => {
    setSaving(true);
    try {
      await api.updateSetting('scripts_dir', scriptsDir);
      alert('Settings saved successfully!');
    } catch (err) {
      alert('Failed to save settings');
      console.error(err);
    } finally {
      setSaving(false);
    }
  };

  const handleAddQuickAction = async () => {
    if (!newActionName || !newActionScript) {
      alert('Please fill in both name and script');
      return;
    }

    setSaving(true);
    try {
      const newAction = await api.createQuickAction({
        name: newActionName,
        script_path: newActionScript,
        icon: null,
        display_order: quickActions.length,
      });
      setQuickActions([...quickActions, newAction]);
      setNewActionName('');
      setNewActionScript('');
    } catch (err) {
      alert('Failed to create quick action');
      console.error(err);
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteQuickAction = async (id: string) => {
    if (!confirm('Are you sure you want to delete this quick action?')) {
      return;
    }

    try {
      await api.deleteQuickAction(id);
      setQuickActions(quickActions.filter((a) => a.id !== id));
    } catch (err) {
      alert('Failed to delete quick action');
      console.error(err);
    }
  };

  if (loading) {
    return <div className="text-gray-500 dark:text-gray-400">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Settings</h1>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          Configure application settings and quick actions
        </p>
      </div>

      {/* Scripts Directory */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Scripts Directory
        </h2>
        <div className="flex flex-col sm:flex-row gap-4">
          <input
            type="text"
            value={scriptsDir}
            onChange={(e) => setScriptsDir(e.target.value)}
            className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            placeholder="./scripts"
          />
          <button
            onClick={handleSaveScriptsDir}
            disabled={saving}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Quick Actions
        </h2>

        {/* Add New Quick Action */}
        <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <h3 className="text-sm font-medium text-gray-900 dark:text-white mb-3">
            Add New Quick Action
          </h3>
          <div className="space-y-3">
            <input
              type="text"
              value={newActionName}
              onChange={(e) => setNewActionName(e.target.value)}
              placeholder="Action name"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            />
            <select
              value={newActionScript}
              onChange={(e) => setNewActionScript(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            >
              <option value="">Select a script...</option>
              {scripts.map((script) => (
                <option key={script} value={script}>
                  {script}
                </option>
              ))}
            </select>
            <button
              onClick={handleAddQuickAction}
              disabled={saving || !newActionName || !newActionScript}
              className="flex items-center gap-2 px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
            >
              <Plus className="w-4 h-4" />
              Add Quick Action
            </button>
          </div>
        </div>

        {/* Quick Actions List */}
        <div className="space-y-2">
          {quickActions.length === 0 ? (
            <p className="text-gray-500 dark:text-gray-400">No quick actions configured.</p>
          ) : (
            quickActions.map((action) => (
              <div
                key={action.id}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
              >
                <div>
                  <p className="font-medium text-gray-900 dark:text-white">{action.name}</p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">{action.script_path}</p>
                </div>
                <button
                  onClick={() => handleDeleteQuickAction(action.id)}
                  className="p-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
