import React from 'react';

export type BadgeTier = 'native' | 'managed' | 'simulated' | 'unavailable' | 'neutral';

interface BadgeProps {
  tier?: BadgeTier;
  children: React.ReactNode;
  size?: 'sm' | 'md';
}

export const Badge: React.FC<BadgeProps> = ({ tier = 'neutral', children, size = 'sm' }) => {
  const getColors = () => {
    switch (tier) {
      case 'native':
        return { bg: 'rgba(16, 185, 129, 0.15)', border: 'rgba(16, 185, 129, 0.4)', text: '#10b981' };
      case 'managed':
        return { bg: 'rgba(245, 158, 11, 0.15)', border: 'rgba(245, 158, 11, 0.4)', text: '#f59e0b' };
      case 'simulated':
        return { bg: 'rgba(6, 182, 212, 0.15)', border: 'rgba(6, 182, 212, 0.4)', text: '#06b6d4' };
      case 'unavailable':
        return { bg: 'rgba(244, 63, 94, 0.15)', border: 'rgba(244, 63, 94, 0.4)', text: '#f43f5e' };
      case 'neutral':
      default:
        return { bg: 'var(--nova-surface-raised)', border: 'var(--nova-border-subtle)', text: 'var(--nova-text-secondary)' };
    }
  };

  const { bg, border, text } = getColors();
  const padding = size === 'sm' ? '2px 6px' : '4px 10px';
  const fontSize = size === 'sm' ? 'var(--nova-text-xs)' : 'var(--nova-text-sm)';

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '4px',
        padding,
        fontSize,
        fontWeight: 500,
        borderRadius: 'var(--nova-radius-full)',
        backgroundColor: bg,
        border: `1px solid ${border}`,
        color: text,
        lineHeight: 1.2,
      }}
    >
      {children}
    </span>
  );
};
