import React from 'react';
import { ShellWindow } from '@shared/types';

interface TaskSwitcherProps {
  isOpen: boolean;
  windows: ShellWindow[];
  selectedIndex: number;
  onSelectIndex: (index: number) => void;
}

export const TaskSwitcher: React.FC<TaskSwitcherProps> = ({
  isOpen,
  windows,
  selectedIndex,
  onSelectIndex,
}) => {
  if (!isOpen || windows.length === 0) return null;

  return (
    <div
      role="dialog"
      aria-label="Window Switcher"
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        backdropFilter: 'var(--nova-blur-glass)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 'var(--nova-z-modal)' as any,
      }}
    >
      <div
        style={{
          display: 'flex',
          gap: '12px',
          padding: '20px',
          backgroundColor: 'var(--nova-surface-base)',
          border: '1px solid var(--nova-border-strong)',
          borderRadius: 'var(--nova-radius-xl)',
          boxShadow: 'var(--nova-shadow-lg)',
          maxWidth: '85vw',
          overflowX: 'auto',
        }}
      >
        {windows.map((win, idx) => {
          const isSelected = idx === selectedIndex;
          return (
            <div
              key={win.id}
              onClick={() => onSelectIndex(idx)}
              style={{
                width: '140px',
                height: '110px',
                padding: '12px',
                borderRadius: 'var(--nova-radius-lg)',
                backgroundColor: isSelected ? 'var(--nova-surface-raised)' : 'transparent',
                border: `2px solid ${isSelected ? 'var(--nova-accent-primary)' : 'var(--nova-border-subtle)'}`,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '8px',
                cursor: 'pointer',
                transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
              }}
            >
              <div style={{ fontSize: '24px' }}>{win.icon || '📄'}</div>
              <div
                style={{
                  fontSize: 'var(--nova-text-xs)',
                  fontWeight: 600,
                  textAlign: 'center',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                  width: '100%',
                }}
              >
                {win.title}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
