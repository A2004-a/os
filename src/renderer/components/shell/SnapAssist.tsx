import React from 'react';
import { ShellWindow, SnapZone } from '@shared/types';
import { LayoutGrid, X } from 'lucide-react';
import { computeSnapBounds } from '../../state/windowStore';

interface SnapAssistProps {
  remainingZone: SnapZone | null;
  candidateWindows: ShellWindow[];
  containerBounds: { width: number; height: number };
  onSelectWindow: (windowId: string, zone: SnapZone) => void;
  onDismiss: () => void;
}

export const SnapAssist: React.FC<SnapAssistProps> = ({
  remainingZone,
  candidateWindows,
  containerBounds,
  onSelectWindow,
  onDismiss,
}) => {
  if (!remainingZone || candidateWindows.length === 0) {
    return null;
  }

  const zoneBounds = computeSnapBounds(remainingZone, containerBounds);

  return (
    <div
      role="region"
      aria-label="Snap Assist - Choose another window to fill space"
      style={{
        position: 'absolute',
        left: `${zoneBounds.x + 16}px`,
        top: `${zoneBounds.y + 16}px`,
        width: `${Math.max(280, zoneBounds.width - 32)}px`,
        height: `${Math.max(200, zoneBounds.height - 32)}px`,
        backgroundColor: 'var(--nova-surface-glass)',
        backdropFilter: 'var(--nova-blur-glass)',
        border: '1px dashed var(--nova-accent-primary)',
        borderRadius: 'var(--nova-radius-xl)',
        boxShadow: 'var(--nova-shadow-lg)',
        zIndex: 150, // Between normal windows and dock/popovers
        padding: '16px',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
        overflow: 'hidden',
        animation: 'fadeIn var(--nova-duration-normal) var(--nova-ease-spring)',
      }}
      onClick={(e) => e.stopPropagation()}
    >
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <LayoutGrid size={16} color="var(--nova-accent-primary)" />
          <span style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600, color: 'var(--nova-text-primary)' }}>
            Snap Assist — Select companion window
          </span>
        </div>
        <button
          onClick={onDismiss}
          title="Dismiss Snap Assist"
          aria-label="Dismiss Snap Assist"
          style={{
            padding: '4px',
            borderRadius: 'var(--nova-radius-sm)',
            cursor: 'pointer',
            color: 'var(--nova-text-secondary)',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.color = '#f43f5e')}
          onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--nova-text-secondary)')}
        >
          <X size={15} />
        </button>
      </div>

      {/* Grid of Candidate Windows */}
      <div
        style={{
          flex: 1,
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
          gap: '12px',
          overflowY: 'auto',
          padding: '4px',
        }}
      >
        {candidateWindows.map((win) => (
          <button
            key={win.id}
            onClick={() => onSelectWindow(win.id, remainingZone)}
            aria-label={`Snap ${win.title} into remaining space`}
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '8px',
              padding: '16px 12px',
              backgroundColor: 'var(--nova-surface-base)',
              border: '1px solid var(--nova-border-subtle)',
              borderRadius: 'var(--nova-radius-lg)',
              boxShadow: 'var(--nova-shadow-sm)',
              cursor: 'pointer',
              transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
              textAlign: 'center',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.borderColor = 'var(--nova-accent-primary)';
              e.currentTarget.style.transform = 'translateY(-2px)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = 'var(--nova-border-subtle)';
              e.currentTarget.style.transform = 'translateY(0)';
            }}
          >
            <div
              style={{
                width: '40px',
                height: '40px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: 'var(--nova-surface-raised)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '18px',
              }}
            >
              {win.icon || '🪟'}
            </div>
            <span
              style={{
                fontSize: 'var(--nova-text-xs)',
                fontWeight: 600,
                color: 'var(--nova-text-primary)',
                maxWidth: '160px',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              {win.title}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};
