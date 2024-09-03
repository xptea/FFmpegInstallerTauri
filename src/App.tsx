import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface ProgressPayload {
  action: string;
  progress: number;
}

function App() {
  const [status, setStatus] = useState("");
  const [installing, setInstalling] = useState(false);
  const [action, setAction] = useState("");
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const unlisten = listen<ProgressPayload>("install_progress", (event) => {
      setAction(event.payload.action);
      setProgress(event.payload.progress);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  async function installAll() {
    setInstalling(true);
    setAction("");
    setProgress(0);
    setStatus("");
    try {
      await invoke("install_ffmpeg");
      // The app will close automatically after installation, so we don't need to set a success status
    } catch (error) {
      setStatus(`Error: ${error}`);
      setInstalling(false);
      console.error("Installation error:", error);
    }
  }

  return (
    <div className="container">
      <div className="content">
        <h1 className="title">FFmpeg & SkibidiSlicer Installer</h1>
        <p className="subtitle">Quick setup for FFmpeg and SkibidiSlicer on Windows</p>
        
        <button
          className={`install-button ${installing ? 'installing' : ''}`}
          onClick={installAll}
          disabled={installing}
        >
          {installing ? "Installing..." : "Install"}
        </button>

        {installing && (
          <div className="installation-progress">
            <p className="progress-text">{action}</p>
            <div className="progress-bar">
              <div className="progress" style={{ width: `${progress}%` }}></div>
            </div>
            <p className="progress-percentage">{progress}%</p>
          </div>
        )}

        {status && (
          <div className={`status-message ${status.includes('Error') ? 'error' : 'success'}`}>
            <p>{status}</p>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
