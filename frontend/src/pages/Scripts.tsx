import { useEffect, useState, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useWebSocket } from '../hooks/useWebSocket';
import { api } from '../lib/api';
import { Play, Square } from 'lucide-react';

export function Scripts() {
  const [searchParams] = useSearchParams();
  const [scripts, setScripts] = useState<string[]>([]);
  const [selectedScript, setSelectedScript] = useState<string>('');
  const [currentTaskId, setCurrentTaskId] = useState<string | null>(null);
  const terminalRef = useRef<HTMLDivElement>(null);
  
  const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/api/ws`;
  const { connected, messages, send, clearMessages } = useWebSocket(wsUrl);

  useEffect(() => {
    const fetchScripts = async () => {
      try {
        const scriptList = await api.listScripts();
        setScripts(scriptList);
        
        // Check if script is pre-selected from URL
        const scriptParam = searchParams.get('script');
        if (scriptParam) {
          const scriptName = scriptParam.split('/').pop() || scriptParam;
          if (scriptList.includes(scriptName)) {
            setSelectedScript(scriptName);
          }
        }
      } catch (err) {
        console.error('Failed to fetch scripts:', err);
      }
    };

    fetchScripts();
  }, [searchParams]);

  useEffect(() => {
    // Auto-scroll to bottom when new messages arrive
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [messages]);

  useEffect(() => {
    // Handle WebSocket messages
    messages.forEach((msg) => {
      if (msg.type === 'started' && msg.task_id) {
        setCurrentTaskId(msg.task_id);
      } else if (msg.type === 'exit' || msg.type === 'cancelled' || msg.type === 'error') {
        setCurrentTaskId(null);
      }
    });
  }, [messages]);

  const handleRun = () => {
    if (!selectedScript) return;
    clearMessages();
    send({ type: 'run', script: selectedScript });
  };

  const handleCancel = () => {
    if (currentTaskId) {
      send({ type: 'cancel', task_id: currentTaskId });
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Scripts</h1>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          Execute and monitor script execution in real-time
        </p>
      </div>

      {/* Script Selector */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-4">
        <div className="flex flex-col sm:flex-row gap-4">
          <select
            value={selectedScript}
            onChange={(e) => setSelectedScript(e.target.value)}
            className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            disabled={!!currentTaskId}
          >
            <option value="">Select a script...</option>
            {scripts.map((script) => (
              <option key={script} value={script}>
                {script}
              </option>
            ))}
          </select>
          <div className="flex gap-2">
            <button
              onClick={handleRun}
              disabled={!selectedScript || !!currentTaskId || !connected}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              <Play className="w-4 h-4" />
              Run
            </button>
            {currentTaskId && (
              <button
                onClick={handleCancel}
                className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 flex items-center gap-2"
              >
                <Square className="w-4 h-4" />
                Cancel
              </button>
            )}
          </div>
        </div>
        {!connected && (
          <p className="mt-2 text-sm text-yellow-600 dark:text-yellow-400">
            WebSocket disconnected. Reconnecting...
          </p>
        )}
      </div>

      {/* Terminal Output */}
      <div className="bg-gray-900 rounded-lg shadow-lg overflow-hidden">
        <div className="bg-gray-800 px-4 py-2 flex items-center justify-between">
          <span className="text-gray-300 text-sm font-medium">Terminal Output</span>
          {currentTaskId && (
            <span className="text-green-400 text-xs">Running...</span>
          )}
        </div>
        <div
          ref={terminalRef}
          className="p-4 h-96 overflow-y-auto font-mono text-sm text-green-400"
          style={{ backgroundColor: '#1a1a1a' }}
        >
          {messages.length === 0 ? (
            <div className="text-gray-500">No output yet. Select a script and click Run.</div>
          ) : (
            messages.map((msg, idx) => {
              if (msg.type === 'stdout') {
                return (
                  <div key={idx} className="text-green-400">
                    {msg.data}
                  </div>
                );
              } else if (msg.type === 'stderr') {
                return (
                  <div key={idx} className="text-red-400">
                    {msg.data}
                  </div>
                );
              } else if (msg.type === 'started') {
                return (
                  <div key={idx} className="text-blue-400">
                    [Started] Task ID: {msg.task_id}
                  </div>
                );
              } else if (msg.type === 'exit') {
                return (
                  <div key={idx} className={msg.code === 0 ? 'text-green-400' : 'text-red-400'}>
                    [Exit] Code: {msg.code}
                  </div>
                );
              } else if (msg.type === 'cancelled') {
                return (
                  <div key={idx} className="text-yellow-400">
                    [Cancelled]
                  </div>
                );
              } else if (msg.type === 'error') {
                return (
                  <div key={idx} className="text-red-400">
                    [Error] {msg.data}
                  </div>
                );
              }
              return null;
            })
          )}
        </div>
      </div>
    </div>
  );
}
