import React, { useState } from 'react';
import { ShellNotification } from '@shared/types';
import { Bell, BellOff, Trash2, CheckCircle2, ShieldAlert } from 'lucide-react';
import { Badge } from '../common/Badge';

interface NotificationCenterProps {
  isOpen: boolean;
  onClose: () => void;
  notifications: ShellNotification[];
  onDismiss: (id: string) => void;
  onClearAll: () => void;
}

export const NotificationCenter: React.FC<NotificationCenterProps> = ({
  isOpen,
  onClose,
  notifications,
  onDismiss,
  onClearAll,
}) => {
  const [dnd, setDnd] = useState(false);

  if (!isOpen) return null;

  const handleTestNativeNotification = async () => {
    if (window.nova?.system?.showNotification) {
      await window.nova.system.showNotification(
        'NovaOS Native Alert',
        'This is a real host notification sent through the Electron bridge.',
        'normal'
      );
    }
  };

  return (
    <div
      role="dialog"
      aria-label="Notification Center"
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.4)',
        zIndex: 'var(--nova-z-popover)' as any,
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: '48px',
          right: '16px',
          width: '360px',
          maxWidth: '92vw',
          maxHeight: '80vh',
          backgroundColor: 'var(--nova-surface-base)',
          border: '1px solid var(--nova-border-strong)',
          borderRadius: 'var(--nova-radius-xl)',
          boxShadow: 'var(--nova-shadow-lg)',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          animation: 'fadeIn var(--nova-duration-fast) var(--nova-ease-standard)',
        }}
      >
        {/* Header */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '14px 18px',
            borderBottom: '1px solid var(--nova-border-subtle)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Bell size={18} color="var(--nova-accent-primary)" />
            <h3 style={{ fontSize: 'var(--nova-text-md)', fontWeight: 600 }}>Notifications</h3>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <button
              onClick={() => setDnd(!dnd)}
              title={dnd ? 'Disable Focus Mode' : 'Enable Focus Mode'}
              style={{
                padding: '4px 8px',
                borderRadius: 'var(--nova-radius-sm)',
                backgroundColor: dnd ? 'rgba(244, 63, 94, 0.15)' : 'var(--nova-surface-raised)',
                color: dnd ? '#f43f5e' : 'var(--nova-text-secondary)',
                fontSize: 'var(--nova-text-xs)',
                cursor: 'pointer',
              }}
            >
              {dnd ? 'Focus On' : 'Focus Off'}
            </button>
            <button
              onClick={onClose}
              style={{ fontSize: '14px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
            >
              ✕
            </button>
          </div>
        </div>

        {/* Action Bar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '8px 18px',
            backgroundColor: 'var(--nova-surface-raised)',
            borderBottom: '1px solid var(--nova-border-subtle)',
          }}
        >
          <button
            onClick={handleTestNativeNotification}
            style={{
              fontSize: 'var(--nova-text-xs)',
              color: 'var(--nova-accent-primary)',
              cursor: 'pointer',
              fontWeight: 500,
            }}
          >
            + Test Native Dispatch
          </button>
          {notifications.length > 0 && (
            <button
              onClick={onClearAll}
              style={{
                fontSize: 'var(--nova-text-xs)',
                color: 'var(--nova-text-muted)',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                cursor: 'pointer',
              }}
            >
              <Trash2 size={12} /> Clear all
            </button>
          )}
        </div>

        {/* Notification List */}
        <div style={{ flex: 1, overflowY: 'auto', padding: '12px' }}>
          {notifications.length === 0 ? (
            <div style={{ padding: '36px 0', textAlign: 'center', color: 'var(--nova-text-muted)' }}>
              <BellOff size={28} style={{ margin: '0 auto 8px', opacity: 0.5 }} />
              <div style={{ fontSize: 'var(--nova-text-sm)' }}>No notifications</div>
            </div>
          ) : (
            notifications.map((item) => (
              <div
                key={item.id}
                style={{
                  padding: '12px',
                  marginBottom: '8px',
                  borderRadius: 'var(--nova-radius-md)',
                  backgroundColor: 'var(--nova-surface-raised)',
                  border: '1px solid var(--nova-border-subtle)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '4px',
                  position: 'relative',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600, color: 'var(--nova-text-primary)' }}>
                    {item.title}
                  </span>
                  <button
                    onClick={() => onDismiss(item.id)}
                    style={{ fontSize: '11px', color: 'var(--nova-text-muted)', cursor: 'pointer' }}
                  >
                    ✕
                  </button>
                </div>
                <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)' }}>
                  {item.message}
                </p>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: '4px' }}>
                  <span style={{ fontSize: '10px', color: 'var(--nova-text-muted)' }}>
                    {new Date(item.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  </span>
                  <Badge tier="simulated">In-App</Badge>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
