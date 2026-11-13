import React, { useState } from 'react';
import { Clipboard, Copy, Check, Trash2, X } from 'lucide-react';

export interface ClipboardItem {
  id: string;
  text: string;
  timestamp: number;
}

interface ClipboardHistoryProps {
  isOpen: boolean;
  onClose: () => void;
  items: ClipboardItem[];
  onSelectItem: (text: string) => void;
  onClearAll: () => void;
}

export const ClipboardHistory: React.FC<ClipboardHistoryProps> = ({
  isOpen,
  onClose,
  items,
  onSelectItem,
  onClearAll,
}) => {
  const [copiedId, setCopiedId] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleCopy = (item: ClipboardItem) => {
    onSelectItem(item.text);
    setCopiedId(item.id);
    setTimeout(() => setCopiedId(null), 1500);
  };

  return (
    <div
      role="dialog"
      aria-label="Clipboard History"
      style={{
        position: 'fixed',
        bottom: '56px',
        right: '260px',
        width: '320px',
        maxHeight: '400px',
        backgroundColor: 'var(--nova-surface-glass)',
        backdropFilter: 'var(--nova-blur-glass)',
        border: '1px solid var(--nova-border-strong)',
        borderRadius: 'var(--nova-radius-xl)',
        boxShadow: 'var(--nova-shadow-lg)',
        zIndex: 'var(--nova-z-popover)' as any,
        padding: '14px',
        display: 'flex',
        flexDirection: 'column',
        gap: '10px',
        animation: 'fadeIn var(--nova-duration-fast) var(--nova-ease-standard)',
      }}
      onClick={(e) => e.stopPropagation()}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Clipboard size={16} color="var(--nova-accent-primary)" />
          <span style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600, color: 'var(--nova-text-primary)' }}>
            Clipboard History
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          {items.length > 0 && (
            <button
              onClick={onClearAll}
              title="Clear All"
              aria-label="Clear All"
              style={{
                padding: '4px',
                borderRadius: 'var(--nova-radius-sm)',
                color: 'var(--nova-text-muted)',
                cursor: 'pointer',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.color = '#f43f5e')}
              onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--nova-text-muted)')}
            >
              <Trash2 size={14} />
            </button>
          )}
          <button
            onClick={onClose}
            aria-label="Close"
            style={{ padding: '4px', color: 'var(--nova-text-muted)', cursor: 'pointer' }}
          >
            <X size={15} />
          </button>
        </div>
      </div>

      <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {items.length === 0 ? (
          <div style={{ padding: '24px 8px', textAlign: 'center', color: 'var(--nova-text-muted)', fontSize: 'var(--nova-text-xs)' }}>
            No recent copied text in this NovaOS session.
          </div>
        ) : (
          items.map((item) => {
            const isCopied = copiedId === item.id;
            return (
              <div
                key={item.id}
                onClick={() => handleCopy(item)}
                style={{
                  padding: '8px 10px',
                  borderRadius: 'var(--nova-radius-md)',
                  backgroundColor: 'var(--nova-surface-base)',
                  border: '1px solid var(--nova-border-subtle)',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  gap: '8px',
                }}
                onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
                onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)')}
              >
                <span
                  style={{
                    fontSize: '12px',
                    color: 'var(--nova-text-primary)',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    maxWidth: '240px',
                  }}
                >
                  {item.text}
                </span>
                <span style={{ color: isCopied ? '#10b981' : 'var(--nova-text-muted)', flexShrink: 0 }}>
                  {isCopied ? <Check size={14} /> : <Copy size={13} />}
                </span>
              </div>
            );
          })
        )}
      </div>

      <div style={{ fontSize: '10px', color: 'var(--nova-text-muted)', textAlign: 'center' }}>
        Private to NovaOS session • Press Esc to close
      </div>
    </div>
  );
};
