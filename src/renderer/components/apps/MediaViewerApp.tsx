import React, { useState } from 'react';
import { Image as ImageIcon, ZoomIn, ZoomOut, RotateCw } from 'lucide-react';
import { Badge } from '../common/Badge';

export const MediaViewerApp: React.FC = () => {
  const [zoom, setZoom] = useState(100);
  const [rotation, setRotation] = useState(0);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Media Viewer Toolbar */}
      <div style={{ height: '42px', borderBottom: '1px solid var(--nova-border-subtle)', backgroundColor: 'var(--nova-surface-base)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 16px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <ImageIcon size={16} color="var(--nova-accent-primary)" />
          <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>Nova_Wallpaper_Geometric_Mesh.svg</span>
          <Badge tier="simulated">In-App Renderer</Badge>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <button onClick={() => setZoom((z) => Math.max(50, z - 25))} style={{ padding: '6px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}>
            <ZoomOut size={15} />
          </button>
          <span style={{ fontSize: '11px', fontFamily: 'var(--nova-font-mono)', minWidth: '40px', textAlign: 'center' }}>
            {zoom}%
          </span>
          <button onClick={() => setZoom((z) => Math.min(300, z + 25))} style={{ padding: '6px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}>
            <ZoomIn size={15} />
          </button>
          <button onClick={() => setRotation((r) => (r + 90) % 360)} style={{ padding: '6px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}>
            <RotateCw size={15} />
          </button>
        </div>
      </div>

      {/* Canvas Viewport */}
      <div style={{ flex: 1, overflow: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '24px' }}>
        <div
          style={{
            transform: `scale(${zoom / 100}) rotate(${rotation}deg)`,
            transition: 'transform var(--nova-duration-normal) var(--nova-ease-standard)',
            boxShadow: 'var(--nova-shadow-lg)',
            borderRadius: 'var(--nova-radius-lg)',
            overflow: 'hidden',
          }}
        >
          <svg width="480" height="300" viewBox="0 0 480 300" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect width="480" height="300" fill="#080c14" />
            <circle cx="240" cy="150" r="120" stroke="#06b6d4" strokeWidth="2" strokeDasharray="4 4" opacity="0.4" />
            <circle cx="240" cy="150" r="80" stroke="#3b82f6" strokeWidth="2" opacity="0.6" />
            <circle cx="240" cy="150" r="40" fill="#06b6d4" fillOpacity="0.2" stroke="#06b6d4" strokeWidth="2" />
            <path d="M120 220 L240 80 L360 220 Z" stroke="#a855f7" strokeWidth="2" fill="rgba(168, 85, 247, 0.15)" />
            <text x="240" y="270" textAnchor="middle" fill="#94a3b8" fontSize="12" fontFamily="sans-serif">
              Nova Surface Generative Vector Artwork
            </text>
          </svg>
        </div>
      </div>
    </div>
  );
};
