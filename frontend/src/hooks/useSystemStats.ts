import { useEffect, useState } from 'react';
import type { SystemResources } from '../lib/api';
import { api } from '../lib/api';

export function useSystemStats(intervalMs: number = 2000) {
  const [stats, setStats] = useState<SystemResources | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        console.log('useSystemStats: Fetching system stats...');
        const data = await api.getResources();
        console.log('useSystemStats: Successfully fetched system stats', data);
        setStats(data);
        setError(null);
      } catch (err) {
        console.error('useSystemStats: Error fetching system stats', err);
        const errorMessage = err instanceof Error ? err.message : 'Failed to fetch system stats';
        setError(new Error(errorMessage));
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, intervalMs);

    return () => {
      console.log('useSystemStats: Cleaning up interval');
      clearInterval(interval);
    };
  }, [intervalMs]);

  return { stats, loading, error };
}
