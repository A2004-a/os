import React from 'react';
import { SnapZone, Bounds } from '@shared/types';
import { computeSnapBounds } from '../../state/windowStore';

interface SnapPreviewProps {
  snapZone: SnapZone | null;
  containerBounds: { width: number; height: number };
}

export const SnapPreview: React.FC<SnapPreviewProps> = ({ snapZone, containerBounds }) => {
  if (!snapZone) return null;

  const bounds: Bounds = computeSnapBounds(snapZone, containerBounds);

  return (
    <div
      style={{
        position: 'absolute',
        left: `${bounds.x}px`,
        top: `${bounds.y}px`,
        width: `${bounds.width}px`,
        height: `${bounds.height}px`,
        backgroundColor: 'var(--nova-accent-glow)',
        border: '2px dashed var(--nova-accent-primary)',
        borderRadius: 'var(--nova-radius-lg)',
        pointerEvents: 'none',
        zIndex: 9999,
        transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
      }}
    />
  );
};
