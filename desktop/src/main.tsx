import { StrictMode, Component, type ReactNode } from 'react';
import { createRoot } from 'react-dom/client';
import * as tauriHttp from '@tauri-apps/plugin-http';
import App from './App';
import './index.css';

(window as unknown as Record<string, unknown>).__RUCKCHAT_FETCH__ = tauriHttp.fetch;

interface ErrorBoundaryState {
  error: Error | null;
}

class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{ padding: '1rem', fontFamily: 'system-ui, sans-serif' }}>
          <h1 style={{ color: '#ef4444' }}>RuckChat failed to start</h1>
          <pre style={{ whiteSpace: 'pre-wrap' }}>{this.state.error.stack ?? this.state.error.message}</pre>
        </div>
      );
    }
    return this.props.children;
  }
}

const root = document.getElementById('root');
if (!root) {
  throw new Error('Root element not found');
}

createRoot(root).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>,
);
