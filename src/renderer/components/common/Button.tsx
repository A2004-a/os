import React, { ButtonHTMLAttributes } from 'react';

export type ButtonVariant = 'primary' | 'secondary' | 'subtle' | 'danger';
export type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  icon?: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  variant = 'secondary',
  size = 'md',
  icon,
  children,
  style,
  disabled,
  ...props
}) => {
  const getVariantStyles = (): React.CSSProperties => {
    switch (variant) {
      case 'primary':
        return {
          backgroundColor: 'var(--nova-accent-primary)',
          color: 'var(--nova-text-inverse)',
          border: '1px solid transparent',
          fontWeight: 600,
        };
      case 'secondary':
        return {
          backgroundColor: 'var(--nova-surface-raised)',
          color: 'var(--nova-text-primary)',
          border: '1px solid var(--nova-border-subtle)',
        };
      case 'subtle':
        return {
          backgroundColor: 'transparent',
          color: 'var(--nova-text-secondary)',
          border: '1px solid transparent',
        };
      case 'danger':
        return {
          backgroundColor: 'rgba(244, 63, 94, 0.15)',
          color: '#f43f5e',
          border: '1px solid rgba(244, 63, 94, 0.3)',
        };
    }
  };

  const getSizeStyles = (): React.CSSProperties => {
    switch (size) {
      case 'sm':
        return { padding: '4px 8px', fontSize: 'var(--nova-text-xs)' };
      case 'md':
        return { padding: '6px 12px', fontSize: 'var(--nova-text-sm)' };
      case 'lg':
        return { padding: '10px 18px', fontSize: 'var(--nova-text-base)' };
    }
  };

  return (
    <button
      disabled={disabled}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '6px',
        borderRadius: 'var(--nova-radius-md)',
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.5 : 1,
        transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
        ...getSizeStyles(),
        ...getVariantStyles(),
        ...style,
      }}
      {...props}
    >
      {icon && <span style={{ display: 'inline-flex' }}>{icon}</span>}
      {children}
    </button>
  );
};
