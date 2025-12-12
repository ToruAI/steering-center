import { useEffect, useState } from 'react';
import { useSystemStats } from '../hooks/useSystemStats';
import type { QuickAction } from '../lib/api';
import { api } from '../lib/api';
import { formatUptime, formatBytes } from '../lib/utils';
import { Play } from 'lucide-react';

export function Dashboard() {
  const { stats } = useSystemStats(2000);
  const [quickActions, setQuickActions] = useState<QuickAction[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchQuickActions = async () => {
      try {
        const actions = await api.getQuickActions();
        setQuickActions(actions);
      } catch (err) {
        console.error('Failed to fetch quick actions:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchQuickActions();
  }, []);

  const handleQuickAction = async (action: QuickAction) => {
    // This will be handled by navigating to scripts page with the script pre-selected
    window.location.href = `/scripts?script=${encodeURIComponent(action.script_path)}`;
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Dashboard</h1>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          System monitoring and quick actions
        </p>
      </div>

      {/* System Stats */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-3">
        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                  <span className="text-white text-xs font-bold">CPU</span>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    CPU Usage
                  </dt>
                  <dd className="text-lg font-semibold text-gray-900 dark:text-white">
                    {stats ? `${stats.cpu_percent.toFixed(1)}%` : '--'}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center">
                  <span className="text-white text-xs font-bold">RAM</span>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Memory Usage
                  </dt>
                  <dd className="text-lg font-semibold text-gray-900 dark:text-white">
                    {stats
                      ? `${stats.memory_percent.toFixed(1)}% (${formatBytes(stats.memory_used)} / ${formatBytes(stats.memory_total)})`
                      : '--'}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-8 h-8 bg-purple-500 rounded-full flex items-center justify-center">
                  <span className="text-white text-xs font-bold">UP</span>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Uptime
                  </dt>
                  <dd className="text-lg font-semibold text-gray-900 dark:text-white">
                    {stats ? formatUptime(stats.uptime_seconds) : '--'}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Quick Actions
        </h2>
        {loading ? (
          <div className="text-gray-500 dark:text-gray-400">Loading...</div>
        ) : quickActions.length === 0 ? (
          <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 text-center">
            <p className="text-gray-500 dark:text-gray-400">
              No quick actions configured. Add some in Settings.
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {quickActions.map((action) => (
              <button
                key={action.id}
                onClick={() => handleQuickAction(action)}
                className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg p-4 hover:shadow-lg transition-shadow text-left"
              >
                <div className="flex items-center">
                  <div className="flex-shrink-0">
                    <Play className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                  </div>
                  <div className="ml-3 flex-1">
                    <p className="text-sm font-medium text-gray-900 dark:text-white">
                      {action.name}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                      {action.script_path}
                    </p>
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
