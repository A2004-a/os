import React, { useState, useEffect, useRef } from 'react';
import { Terminal as TermIcon, Trash2 } from 'lucide-react';
import { Badge } from '../common/Badge';

export const TerminalApp: React.FC = () => {
  const [output, setOutput] = useState<string[]>([]);
  const [inputVal, setInputVal] = useState('');
  const [history, setHistory] = useState<string[]>([]);
  const [historyIdx, setHistoryIdx] = useState<number>(-1);
  const [shellName, setShellName] = useState('powershell');
  const [cwd, setCwd] = useState('Nova Home');
  const [sessionId] = useState(() => `term-${Date.now()}`);
  const bottomRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Initial welcome banner
    setOutput([
      'NovaOS Terminal v0.1.0 (Sandboxed Host Shell Broker)',
      'Type commands to interact with your permitted host environment.',
      '------------------------------------------------------------',
    ]);

    // Spawn session via IPC Bridge
    if (window.nova?.terminal?.spawn) {
      window.nova.terminal.spawn(sessionId).then((res) => {
        if (res.success) {
          if (res.shell) setShellName(res.shell);
          if (res.cwd) setCwd(res.cwd);
        } else {
          setOutput((prev) => [...prev, `[Terminal spawn error: ${res.error}]`]);
        }
      }).catch((err) => {
        setOutput((prev) => [...prev, `[Terminal error: ${err.message}]`]);
      });
    }

    // Subscribe to stdout/stderr stream
    let cleanupData: (() => void) | undefined;
    if (window.nova?.terminal?.onData) {
      cleanupData = window.nova.terminal.onData((payload) => {
        if (payload.sessionId === sessionId) {
          setOutput((prev) => [...prev, payload.data]);
        }
      });
    }

    return () => {
      if (cleanupData) cleanupData();
      if (window.nova?.terminal?.kill) {
        window.nova.terminal.kill(sessionId).catch(console.error);
      }
    };
  }, [sessionId]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [output]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      const cmd = inputVal;
      if (!cmd) return;

      setOutput((prev) => [...prev, `> ${cmd}\n`]);
      setHistory((prev) => [...prev, cmd]);
      setHistoryIdx(-1);
      setInputVal('');

      if (cmd.trim() === 'clear' || cmd.trim() === 'cls') {
        setOutput([]);
        return;
      }

      if (window.nova?.terminal?.sendInput) {
        window.nova.terminal.sendInput(sessionId, `${cmd}\n`);
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (history.length > 0) {
        const nextIdx = historyIdx === -1 ? history.length - 1 : Math.max(0, historyIdx - 1);
        setHistoryIdx(nextIdx);
        setInputVal(history[nextIdx]);
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIdx !== -1) {
        const nextIdx = historyIdx + 1;
        if (nextIdx < history.length) {
          setHistoryIdx(nextIdx);
          setInputVal(history[nextIdx]);
        } else {
          setHistoryIdx(-1);
          setInputVal('');
        }
      }
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#0a0d14',
        color: '#f1f5f9',
        fontFamily: 'var(--nova-font-mono)',
        fontSize: '12px',
      }}
      onClick={() => inputRef.current?.focus()}
    >
      {/* Terminal Status Bar */}
      <div
        style={{
          height: '36px',
          backgroundColor: '#111722',
          borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 12px',
          userSelect: 'none',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <TermIcon size={14} color="#10b981" />
          <span style={{ fontWeight: 600, color: '#f1f5f9' }}>{shellName}</span>
          <span style={{ color: '#64748b' }}>•</span>
          <span style={{ color: '#94a3b8', fontSize: '11px', maxWidth: '300px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {cwd}
          </span>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Badge tier="native">Native Host Shell</Badge>
          <button
            onClick={(e) => {
              e.stopPropagation();
              setOutput([]);
            }}
            title="Clear output"
            style={{ color: '#64748b', cursor: 'pointer', padding: '4px' }}
          >
            <Trash2 size={13} />
          </button>
        </div>
      </div>

      {/* Terminal Stream Console */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: '12px',
          lineHeight: 1.5,
          whiteSpace: 'pre-wrap',
          wordBreak: 'break-word',
        }}
      >
        {output.map((line, idx) => (
          <div key={idx}>{line}</div>
        ))}
        <div ref={bottomRef} />
      </div>

      {/* Interactive Input Prompt */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          padding: '8px 12px',
          backgroundColor: '#0d111a',
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
        }}
      >
        <span style={{ color: '#10b981', marginRight: '8px', fontWeight: 600 }}>❯</span>
        <input
          ref={inputRef}
          type="text"
          value={inputVal}
          onChange={(e) => setInputVal(e.target.value)}
          onKeyDown={handleKeyDown}
          autoFocus
          placeholder="Run commands (e.g. dir, ls, node -v)..."
          style={{
            flex: 1,
            color: '#f8fafc',
            fontFamily: 'inherit',
            fontSize: 'inherit',
          }}
        />
      </div>
    </div>
  );
};
