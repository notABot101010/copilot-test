import { useSignal } from '@preact/signals';
import { steerSession } from '../api';
import type { SteerCommand } from '../types';

interface Props {
  sessionId: string;
  isRunning: boolean;
  isPaused: boolean;
}

export function SteeringControls({ sessionId, isRunning, isPaused }: Props) {
  const loading = useSignal(false);

  const handleSteer = async (command: SteerCommand) => {
    loading.value = true;
    try {
      await steerSession(sessionId, command);
    } catch (err) {
      console.error('Failed to steer:', err);
    } finally {
      loading.value = false;
    }
  };

  if (!isRunning && !isPaused) {
    return null;
  }

  return (
    <div className="flex items-center gap-2 p-2 bg-gray-100 rounded-lg">
      <span className="text-sm text-gray-600">Agent is {isPaused ? 'paused' : 'working'}...</span>
      <div className="flex gap-2">
        {isPaused ? (
          <button
            onClick={() => handleSteer('resume')}
            disabled={loading.value}
            className="px-3 py-1 text-sm bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
          >
            Resume
          </button>
        ) : (
          <button
            onClick={() => handleSteer('pause')}
            disabled={loading.value}
            className="px-3 py-1 text-sm bg-yellow-600 text-white rounded hover:bg-yellow-700 disabled:opacity-50"
          >
            Pause
          </button>
        )}
        <button
          onClick={() => handleSteer('cancel')}
          disabled={loading.value}
          className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          onClick={() => handleSteer('focus')}
          disabled={loading.value}
          className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          Focus
        </button>
      </div>
    </div>
  );
}
