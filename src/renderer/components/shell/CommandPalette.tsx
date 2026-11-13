import React, { useState, useEffect, useRef } from 'react';
import { Search, Folder, FileText, Terminal, Settings, Activity, Calculator, Clock, Package, Wrench, SunMoon, Layers } from 'lucide-react';
import { Badge } from '../common/Badge';

export interface PaletteItem {
  id: string;
  title: string;
  category: 'Applications' | 'Actions' | 'Files';
  icon: React.ReactNode;
  tier: 'native' | 'managed' | 'simulated' | 'neutral';
  action: () => void;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onOpenApp: (appId: string, name: string) => void;
  onSwitchWorkspace: (workspaceId: string) => void;
  onToggleTheme: () => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  onOpenApp,
  onSwitchWorkspace,
  onToggleTheme,
}) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const allItems: PaletteItem[] = [
    {
      id: 'app-files',
      title: 'Files (Nova Home & Permitted Folders)',
      category: 'Applications',
      icon: <Folder size={18} color="#06b6d4" />,
      tier: 'simulated',
      action: () => onOpenApp('io.novaos.files', 'Files'),
    },
    {
      id: 'app-notes',
      title: 'Notes (Markdown Editor)',
      category: 'Applications',
      icon: <FileText size={18} color="#3b82f6" />,
      tier: 'simulated',
      action: () => onOpenApp('io.novaos.notes', 'Notes'),
    },
    {
      id: 'app-terminal',
      title: 'Terminal (Host Shell)',
      category: 'Applications',
      icon: <Terminal size={18} color="#10b981" />,
      tier: 'native',
      action: () => onOpenApp('io.novaos.terminal', 'Terminal'),
    },
    {
      id: 'app-calc',
      title: 'Calculator',
      category: 'Applications',
      icon: <Calculator size={18} color="#f59e0b" />,
      tier: 'simulated',
      action: () => onOpenApp('io.novaos.calc', 'Calculator'),
    },
    {
      id: 'app-clock',
      title: 'Clock & Timers',
      category: 'Applications',
      icon: <Clock size={18} color="#a855f7" />,
      tier: 'simulated',
      action: () => onOpenApp('io.novaos.clock', 'Clock'),
    },
    {
      id: 'app-sysmon',
      title: 'System Monitor (Host Hardware Metrics)',
      category: 'Applications',
      icon: <Activity size={18} color="#f43f5e" />,
      tier: 'native',
      action: () => onOpenApp('io.novaos.sysmon', 'System Monitor'),
    },
    {
      id: 'app-software',
      title: 'Software Center (App Catalog)',
      category: 'Applications',
      icon: <Package size={18} color="#38bdf8" />,
      tier: 'simulated',
      action: () => onOpenApp('io.novaos.software', 'Software Center'),
    },
    {
      id: 'app-settings',
      title: 'Settings (Appearance, Permissions, Storage)',
      category: 'Applications',
      icon: <Settings size={18} color="#94a3b8" />,
      tier: 'simulated',
      action: () => onOpenApp('io.novaos.settings', 'Settings'),
    },
    {
      id: 'app-devconsole',
      title: 'Developer Console (IPC & Audit)',
      category: 'Applications',
      icon: <Wrench size={18} color="#f97316" />,
      tier: 'managed',
      action: () => onOpenApp('io.novaos.devconsole', 'Developer Console'),
    },
    {
      id: 'act-theme',
      title: 'Switch Color Theme',
      category: 'Actions',
      icon: <SunMoon size={18} color="#eab308" />,
      tier: 'neutral',
      action: onToggleTheme,
    },
    {
      id: 'act-ws-1',
      title: 'Switch to Workspace 1',
      category: 'Actions',
      icon: <Layers size={18} color="#a855f7" />,
      tier: 'neutral',
      action: () => onSwitchWorkspace('ws-1'),
    },
    {
      id: 'act-ws-2',
      title: 'Switch to Workspace 2',
      category: 'Actions',
      icon: <Layers size={18} color="#a855f7" />,
      tier: 'neutral',
      action: () => onSwitchWorkspace('ws-2'),
    },
    {
      id: 'act-ws-3',
      title: 'Switch to Workspace 3',
      category: 'Actions',
      icon: <Layers size={18} color="#a855f7" />,
      tier: 'neutral',
      action: () => onSwitchWorkspace('ws-3'),
    },
  ];

  const filteredItems = allItems.filter(
    (item) =>
      item.title.toLowerCase().includes(query.toLowerCase()) ||
      item.category.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % Math.max(1, filteredItems.length));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + filteredItems.length) % Math.max(1, filteredItems.length));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const selected = filteredItems[selectedIndex];
      if (selected) {
        selected.action();
        onClose();
      }
    } else if (e.key === 'Escape') {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-label="Command Search"
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.65)',
        backdropFilter: 'var(--nova-blur-glass)',
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'center',
        paddingTop: '15vh',
        zIndex: 'var(--nova-z-popover)' as any,
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        style={{
          width: '600px',
          maxWidth: '92vw',
          backgroundColor: 'var(--nova-surface-base)',
          border: '1px solid var(--nova-border-strong)',
          borderRadius: 'var(--nova-radius-xl)',
          boxShadow: 'var(--nova-shadow-lg)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          animation: 'fadeIn var(--nova-duration-fast) var(--nova-ease-standard)',
        }}
      >
        {/* Search Bar Input */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
            padding: '14px 18px',
            borderBottom: '1px solid var(--nova-border-subtle)',
          }}
        >
          <Search size={20} color="var(--nova-accent-primary)" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Search applications, actions, files... (Ctrl+Space)"
            aria-label="Command Search input"
            style={{
              flex: 1,
              fontSize: 'var(--nova-text-md)',
              color: 'var(--nova-text-primary)',
            }}
          />
          <kbd
            style={{
              padding: '2px 6px',
              fontSize: '11px',
              borderRadius: '4px',
              backgroundColor: 'var(--nova-surface-raised)',
              border: '1px solid var(--nova-border-subtle)',
              color: 'var(--nova-text-muted)',
            }}
          >
            ESC
          </kbd>
        </div>

        {/* Results List */}
        <div
          style={{
            maxHeight: '340px',
            overflowY: 'auto',
            padding: '8px',
          }}
        >
          {filteredItems.length === 0 ? (
            <div style={{ padding: '24px', textAlign: 'center', color: 'var(--nova-text-muted)', fontSize: 'var(--nova-text-sm)' }}>
              No matches found for "{query}"
            </div>
          ) : (
            filteredItems.map((item, index) => {
              const isSelected = index === selectedIndex;
              return (
                <div
                  key={item.id}
                  onClick={() => {
                    item.action();
                    onClose();
                  }}
                  onMouseEnter={() => setSelectedIndex(index)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '10px 14px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: isSelected ? 'var(--nova-surface-raised)' : 'transparent',
                    cursor: 'pointer',
                    transition: 'background-color var(--nova-duration-fast) var(--nova-ease-standard)',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <span>{item.icon}</span>
                    <span style={{ fontSize: 'var(--nova-text-sm)', fontWeight: isSelected ? 600 : 400, color: 'var(--nova-text-primary)' }}>
                      {item.title}
                    </span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Badge tier={item.tier}>{item.tier}</Badge>
                    <span style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-muted)' }}>
                      {item.category}
                    </span>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};

