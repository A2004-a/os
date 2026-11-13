import { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallbackTitle?: string;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error, errorInfo: null };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('NovaOS Uncaught Shell Error:', error, errorInfo);
    this.setState({ errorInfo });
  }

  private handleReload = () => {
    window.location.reload();
  };

  public render() {
    if (this.state.hasError) {
      return (
        <div
          role="alert"
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100%',
            padding: '24px',
            backgroundColor: 'var(--nova-surface-canvas)',
            color: 'var(--nova-text-primary)',
            textAlign: 'center',
          }}
        >
          <div
            style={{
              padding: '24px',
              borderRadius: 'var(--nova-radius-lg)',
              backgroundColor: 'var(--nova-surface-raised)',
              border: '1px solid var(--nova-border-subtle)',
              maxWidth: '520px',
              boxShadow: 'var(--nova-shadow-md)',
            }}
          >
            <h2 style={{ fontSize: 'var(--nova-text-lg)', marginBottom: '8px', color: 'var(--nova-accent-primary)' }}>
              {this.props.fallbackTitle || 'Application Surface Recovered'}
            </h2>
            <p style={{ fontSize: 'var(--nova-text-sm)', color: 'var(--nova-text-secondary)', marginBottom: '16px' }}>
              NovaOS isolated an unexpected error in this surface to prevent a desktop crash.
            </p>
            {this.state.error && (
              <pre
                style={{
                  fontFamily: 'var(--nova-font-mono)',
                  fontSize: 'var(--nova-text-xs)',
                  padding: '12px',
                  borderRadius: 'var(--nova-radius-md)',
                  backgroundColor: 'var(--nova-surface-base)',
                  color: 'var(--nova-text-muted)',
                  textAlign: 'left',
                  overflowX: 'auto',
                  marginBottom: '16px',
                  maxHeight: '120px',
                }}
              >
                {this.state.error.message}
              </pre>
            )}
            <button
              onClick={this.handleReload}
              style={{
                padding: '8px 16px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: 'var(--nova-accent-primary)',
                color: 'var(--nova-text-inverse)',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Restart Component
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
