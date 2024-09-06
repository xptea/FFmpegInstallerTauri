import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface ProgressPayload {
  message: string;
  percent: number;
}

function App() {
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<ProgressPayload>({ message: '', percent: 0 });
  const [status, setStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [statusMessage, setStatusMessage] = useState('');

  useEffect(() => {
    const unsubscribe = listen<ProgressPayload>('progress', (event) => {
      setProgress(event.payload);
    });

    return () => {
      unsubscribe.then(f => f());
    };
  }, []);

  const handleInstall = async () => {
    setInstalling(true);
    setStatus('idle');
    setStatusMessage('');

    try {
      await invoke('install_ffmpeg_and_skibidi');
      setStatus('success');
      setStatusMessage('Installation completed successfully!');
    } catch (error) {
      setStatus('error');
      setStatusMessage(`Error: ${error}`);
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div className="container">
      <div className="content">
        <h1 className="title">FFmpeg Installer</h1>
        <p className="subtitle">Install FFmpeg and SkibidiSlicer</p>

        <button
          className="install-button"
          onClick={handleInstall}
          disabled={installing}
        >
          {installing ? 'Installing...' : 'Install'}
        </button>

        {installing && (
          <div className="installation-progress">
            <p className="progress-text">{progress.message}</p>
            <div className="progress-bar">
              <div className="progress" style={{ width: `${progress.percent}%` }}></div>
            </div>
            <p className="progress-percentage">{progress.percent.toFixed(1)}%</p>
          </div>
        )}

        {status !== 'idle' && (
          <div className={`status ${status}`}>
            {statusMessage}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
