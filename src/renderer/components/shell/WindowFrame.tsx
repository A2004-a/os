import React, { useState, useRef } from 'react';
import { ShellWindow, Bounds, SnapZone } from '@shared/types';
import { Minus, Square, X, LayoutGrid } from 'lucide-react';

interface WindowFrameProps {
  window: ShellWindow;
  containerBounds: { width: number; height: number };
  onFocus: () => void;
  onClose: () => void;
  onMinimize: () => void;
  onMaximize: () => void;
  onSnap: (zone: SnapZone) => void;
  onMove: (x: number, y: number) => void;
  onResize: (bounds: Bounds) => void;
  onSnapPreview: (zone: SnapZone | null) => void;
  children: React.ReactNode;
}

export const WindowFrame: React.FC<WindowFrameProps> = ({
  window: win,
  containerBounds,
  onFocus,
  onClose,
  onMinimize,
  onMaximize,
  onSnap,
  onMove,
  onResize,
  onSnapPreview,
  children,
}) => {
  const [showSnapMenu, setShowSnapMenu] = useState(false);
  const dragRef = useRef<{ startX: number; startY: number; initialX: number; initialY: number } | null>(null);
  const resizeRef = useRef<{
    direction: string;
    startX: number;
    startY: number;
    initialBounds: Bounds;
  } | null>(null);

  if (win.state === 'minimized') {
    return null;
  }

  // Handle Dragging
  const handleTitleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    onFocus();

    dragRef.current = {
      startX: e.clientX,
      startY: e.clientY,
      initialX: win.bounds.x,
      initialY: win.bounds.y,
    };

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!dragRef.current) return;
      const dx = moveEvent.clientX - dragRef.current.startX;
      const dy = moveEvent.clientY - dragRef.current.startY;
      const newX = Math.max(0, Math.min(containerBounds.width - 100, dragRef.current.initialX + dx));
      const newY = Math.max(0, Math.min(containerBounds.height - 40, dragRef.current.initialY + dy));
      onMove(newX, newY);

      // Detect Edge Snapping
      const threshold = 24;
      if (moveEvent.clientX <= threshold) {
        if (moveEvent.clientY <= threshold) onSnapPreview('top-left');
        else if (moveEvent.clientY >= containerBounds.height - threshold) onSnapPreview('bottom-left');
        else onSnapPreview('left-half');
      } else if (moveEvent.clientX >= containerBounds.width - threshold) {
        if (moveEvent.clientY <= threshold) onSnapPreview('top-right');
        else if (moveEvent.clientY >= containerBounds.height - threshold) onSnapPreview('bottom-right');
        else onSnapPreview('right-half');
      } else if (moveEvent.clientY <= threshold) {
        onSnapPreview('maximize');
      } else {
        onSnapPreview(null);
      }
    };

    const handleMouseUp = (upEvent: MouseEvent) => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      dragRef.current = null;

      const threshold = 24;
      if (upEvent.clientX <= threshold) {
        if (upEvent.clientY <= threshold) onSnap('top-left');
        else if (upEvent.clientY >= containerBounds.height - threshold) onSnap('bottom-left');
        else onSnap('left-half');
      } else if (upEvent.clientX >= containerBounds.width - threshold) {
        if (upEvent.clientY <= threshold) onSnap('top-right');
        else if (upEvent.clientY >= containerBounds.height - threshold) onSnap('bottom-right');
        else onSnap('right-half');
      } else if (upEvent.clientY <= threshold) {
        onSnap('maximize');
      }
      onSnapPreview(null);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  // Handle Resizing
  const handleResizeMouseDown = (direction: string, e: React.MouseEvent) => {
    e.stopPropagation();
    onFocus();

    resizeRef.current = {
      direction,
      startX: e.clientX,
      startY: e.clientY,
      initialBounds: { ...win.bounds },
    };

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!resizeRef.current) return;
      const { direction, startX, startY, initialBounds } = resizeRef.current;
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;

      let newX = initialBounds.x;
      let newY = initialBounds.y;
      let newW = initialBounds.width;
      let newH = initialBounds.height;

      const minW = 320;
      const minH = 200;

      if (direction.includes('e')) newW = Math.max(minW, initialBounds.width + dx);
      if (direction.includes('s')) newH = Math.max(minH, initialBounds.height + dy);
      if (direction.includes('w')) {
        const potentialW = initialBounds.width - dx;
        if (potentialW >= minW) {
          newW = potentialW;
          newX = initialBounds.x + dx;
        }
      }
      if (direction.includes('n')) {
        const potentialH = initialBounds.height - dy;
        if (potentialH >= minH) {
          newH = potentialH;
          newY = initialBounds.y + dy;
        }
      }

      onResize({ x: newX, y: newY, width: newW, height: newH });
    };

    const handleMouseUp = () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      resizeRef.current = null;
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  const isMax = win.state === 'maximized';

  return (
    <div
      role="dialog"
      aria-label={win.title}
      style={{
        position: 'absolute',
        left: `${win.bounds.x}px`,
        top: `${win.bounds.y}px`,
        width: `${win.bounds.width}px`,
        height: `${win.bounds.height}px`,
        zIndex: win.zIndex,
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: 'var(--nova-surface-base)',
        border: `1px solid ${win.focused ? 'var(--nova-border-focus)' : 'var(--nova-border-subtle)'}`,
        borderRadius: isMax ? 0 : 'var(--nova-radius-lg)',
        boxShadow: win.focused ? 'var(--nova-shadow-window)' : 'var(--nova-shadow-md)',
        overflow: 'hidden',
        transition: win.state === 'snapped' || win.state === 'maximized' ? 'all var(--nova-duration-normal) var(--nova-ease-standard)' : 'none',
      }}
      onMouseDown={onFocus}
    >
      {/* Title Bar */}
      <div
        onMouseDown={isMax ? undefined : handleTitleMouseDown}
        onDoubleClick={onMaximize}
        style={{
          height: '38px',
          backgroundColor: 'var(--nova-surface-raised)',
          borderBottom: '1px solid var(--nova-border-subtle)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 12px',
          cursor: isMax ? 'default' : 'move',
          userSelect: 'none',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0 }}>
          {win.icon && <span style={{ fontSize: '14px' }}>{win.icon}</span>}
          <span
            style={{
              fontSize: 'var(--nova-text-sm)',
              fontWeight: 500,
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              color: win.focused ? 'var(--nova-text-primary)' : 'var(--nova-text-secondary)',
            }}
          >
            {win.title}
          </span>
        </div>

        {/* Window Controls */}
        <div
          style={{ display: 'flex', alignItems: 'center', gap: '4px' }}
          onMouseDown={(e) => e.stopPropagation()}
        >
          {/* Snap Selector Menu */}
          <div style={{ position: 'relative' }}>
            <button
              onClick={() => setShowSnapMenu(!showSnapMenu)}
              title="Snap Window"
              aria-label="Snap Window"
              style={{
                width: '28px',
                height: '24px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: 'var(--nova-radius-sm)',
                color: 'var(--nova-text-secondary)',
                cursor: 'pointer',
              }}
            >
              <LayoutGrid size={13} />
            </button>

            {showSnapMenu && (
              <div
                style={{
                  position: 'absolute',
                  top: '100%',
                  right: 0,
                  marginTop: '4px',
                  backgroundColor: 'var(--nova-surface-raised)',
                  border: '1px solid var(--nova-border-strong)',
                  borderRadius: 'var(--nova-radius-md)',
                  padding: '8px',
                  boxShadow: 'var(--nova-shadow-lg)',
                  display: 'grid',
                  gridTemplateColumns: 'repeat(3, 1fr)',
                  gap: '4px',
                  zIndex: 10,
                }}
              >
                <button
                  onClick={() => { onSnap('left-half'); setShowSnapMenu(false); }}
                  style={{ padding: '6px', fontSize: '11px', backgroundColor: 'var(--nova-surface-base)', borderRadius: '4px', color: 'var(--nova-text-secondary)' }}
                >
                  Left 1/2
                </button>
                <button
                  onClick={() => { onSnap('center-third'); setShowSnapMenu(false); }}
                  style={{ padding: '6px', fontSize: '11px', backgroundColor: 'var(--nova-surface-base)', borderRadius: '4px', color: 'var(--nova-text-secondary)' }}
                >
                  Center 1/3
                </button>
                <button
                  onClick={() => { onSnap('right-half'); setShowSnapMenu(false); }}
                  style={{ padding: '6px', fontSize: '11px', backgroundColor: 'var(--nova-surface-base)', borderRadius: '4px', color: 'var(--nova-text-secondary)' }}
                >
                  Right 1/2
                </button>
                <button
                  onClick={() => { onSnap('top-left'); setShowSnapMenu(false); }}
                  style={{ padding: '6px', fontSize: '11px', backgroundColor: 'var(--nova-surface-base)', borderRadius: '4px', color: 'var(--nova-text-secondary)' }}
                >
                  Top-Left
                </button>
                <button
                  onClick={() => { onSnap('top-half'); setShowSnapMenu(false); }}
                  style={{ padding: '6px', fontSize: '11px', backgroundColor: 'var(--nova-surface-base)', borderRadius: '4px', color: 'var(--nova-text-secondary)' }}
                >
                  Top 1/2
                </button>
                <button
                  onClick={() => { onSnap('top-right'); setShowSnapMenu(false); }}
                  style={{ padding: '6px', fontSize: '11px', backgroundColor: 'var(--nova-surface-base)', borderRadius: '4px', color: 'var(--nova-text-secondary)' }}
                >
                  Top-Right
                </button>
              </div>
            )}
          </div>

          <button
            onClick={onMinimize}
            title="Minimize"
            aria-label="Minimize Window"
            style={{
              width: '28px',
              height: '24px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              borderRadius: 'var(--nova-radius-sm)',
              color: 'var(--nova-text-secondary)',
              cursor: 'pointer',
            }}
          >
            <Minus size={13} />
          </button>
          <button
            onClick={onMaximize}
            title={isMax ? 'Restore' : 'Maximize'}
            aria-label={isMax ? 'Restore Window' : 'Maximize Window'}
            style={{
              width: '28px',
              height: '24px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              borderRadius: 'var(--nova-radius-sm)',
              color: 'var(--nova-text-secondary)',
              cursor: 'pointer',
            }}
          >
            <Square size={12} />
          </button>
          <button
            onClick={onClose}
            title="Close"
            aria-label="Close Window"
            style={{
              width: '28px',
              height: '24px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              borderRadius: 'var(--nova-radius-sm)',
              color: 'var(--nova-text-secondary)',
              cursor: 'pointer',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.color = '#f43f5e')}
            onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--nova-text-secondary)')}
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Window Content Body */}
      <div style={{ flex: 1, position: 'relative', overflow: 'auto', backgroundColor: 'var(--nova-surface-canvas)' }}>
        {children}
      </div>

      {/* 8 Resize Handles (when not maximized) */}
      {!isMax && (
        <>
          <div onMouseDown={(e) => handleResizeMouseDown('n', e)} style={{ position: 'absolute', top: 0, left: 8, right: 8, height: 4, cursor: 'ns-resize' }} />
          <div onMouseDown={(e) => handleResizeMouseDown('s', e)} style={{ position: 'absolute', bottom: 0, left: 8, right: 8, height: 4, cursor: 'ns-resize' }} />
          <div onMouseDown={(e) => handleResizeMouseDown('w', e)} style={{ position: 'absolute', left: 0, top: 8, bottom: 8, width: 4, cursor: 'ew-resize' }} />
          <div onMouseDown={(e) => handleResizeMouseDown('e', e)} style={{ position: 'absolute', right: 0, top: 8, bottom: 8, width: 4, cursor: 'ew-resize' }} />
          <div onMouseDown={(e) => handleResizeMouseDown('nw', e)} style={{ position: 'absolute', top: 0, left: 0, width: 8, height: 8, cursor: 'nwse-resize' }} />
          <div onMouseDown={(e) => handleResizeMouseDown('ne', e)} style={{ position: 'absolute', top: 0, right: 0, width: 8, height: 8, cursor: 'nesw-resize' }} />
          <div onMouseDown={(e) => handleResizeMouseDown('sw', e)} style={{ position: 'absolute', bottom: 0, left: 0, width: 8, height: 8, cursor: 'nesw-resize' }} />
          <div onMouseDown={(e) => handleResizeMouseDown('se', e)} style={{ position: 'absolute', bottom: 0, right: 0, width: 8, height: 8, cursor: 'nwse-resize' }} />
        </>
      )}
    </div>
  );
};
