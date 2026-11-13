import { useState, useEffect, useCallback } from 'react';
import { ShellWindow, Bounds, SnapZone, Workspace } from '@shared/types';

export const INITIAL_WORKSPACES: Workspace[] = [
  { id: 'ws-1', name: 'Workspace 1', index: 0, active: true },
  { id: 'ws-2', name: 'Workspace 2', index: 1, active: false },
  { id: 'ws-3', name: 'Workspace 3', index: 2, active: false },
];

export function computeSnapBounds(
  zone: SnapZone,
  container: { width: number; height: number }
): Bounds {
  const { width, height } = container;
  const halfW = Math.round(width / 2);
  const halfH = Math.round(height / 2);

  switch (zone) {
    case 'left-half':
      return { x: 0, y: 0, width: halfW, height };
    case 'right-half':
      return { x: halfW, y: 0, width: width - halfW, height };
    case 'top-half':
      return { x: 0, y: 0, width, height: halfH };
    case 'bottom-half':
      return { x: 0, y: halfH, width, height: height - halfH };
    case 'top-left':
      return { x: 0, y: 0, width: halfW, height: halfH };
    case 'top-right':
      return { x: halfW, y: 0, width: width - halfW, height: halfH };
    case 'bottom-left':
      return { x: 0, y: halfH, width: halfW, height: height - halfH };
    case 'bottom-right':
      return { x: halfW, y: halfH, width: width - halfW, height: height - halfH };
    case 'center-third': {
      const thirdW = Math.round(width / 3);
      return { x: thirdW, y: 0, width: thirdW, height };
    }
    case 'maximize':
      return { x: 0, y: 0, width, height };
  }
}

export function useWindowManager() {
  const [windows, setWindows] = useState<ShellWindow[]>([]);
  const [activeWorkspaceId, setActiveWorkspaceId] = useState<string>('ws-1');
  const [workspaces] = useState<Workspace[]>(INITIAL_WORKSPACES);
  const [highestZ, setHighestZ] = useState<number>(100);

  // Restore session from SQLite on initial load
  useEffect(() => {
    if (window.nova?.db?.getWindowState) {
      window.nova.db.getWindowState().then((res) => {
        if (res.success && res.windows && res.windows.length > 0) {
          setWindows(res.windows);
          const maxZ = Math.max(...res.windows.map((w) => w.zIndex), 100);
          setHighestZ(maxZ + 1);
        }
      }).catch(console.error);
    }
  }, []);

  // Autosave session on changes
  useEffect(() => {
    if (window.nova?.db?.saveWindowState && windows.length > 0) {
      window.nova.db.saveWindowState(windows).catch(console.error);
    }
  }, [windows]);

  const focusWindow = useCallback((id: string) => {
    setWindows((prev) => {
      const target = prev.find((w) => w.id === id);
      if (!target) return prev;

      const nextZ = highestZ + 1;
      setHighestZ(nextZ);

      return prev.map((w) =>
        w.id === id
          ? { ...w, zIndex: nextZ, focused: true, state: w.state === 'minimized' ? 'normal' : w.state }
          : { ...w, focused: false }
      );
    });
  }, [highestZ]);

  const openApp = useCallback(
    (appId: string, title: string, icon?: string, defaultBounds?: Partial<Bounds>) => {
      setWindows((prev) => {
        // If app window is already open, focus it
        const existing = prev.find((w) => w.appId === appId && w.workspaceId === activeWorkspaceId);
        if (existing) {
          const nextZ = highestZ + 1;
          setHighestZ(nextZ);
          return prev.map((w) =>
            w.id === existing.id
              ? { ...w, zIndex: nextZ, focused: true, state: w.state === 'minimized' ? 'normal' : w.state }
              : { ...w, focused: false }
          );
        }

        const id = `win-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
        const nextZ = highestZ + 1;
        setHighestZ(nextZ);

        // Stagger window positions
        const offset = (prev.length % 6) * 32;
        const width = defaultBounds?.width || 840;
        const height = defaultBounds?.height || 560;

        const newWin: ShellWindow = {
          id,
          appId,
          title,
          workspaceId: activeWorkspaceId,
          bounds: {
            x: Math.max(40, 120 + offset),
            y: Math.max(40, 60 + offset),
            width,
            height,
          },
          state: 'normal',
          zIndex: nextZ,
          focused: true,
          permissions: [],
          icon,
        };

        return [...prev.map((w) => ({ ...w, focused: false })), newWin];
      });
    },
    [activeWorkspaceId, highestZ]
  );

  const closeWindow = useCallback((id: string) => {
    setWindows((prev) => prev.filter((w) => w.id !== id));
  }, []);

  const minimizeWindow = useCallback((id: string) => {
    setWindows((prev) =>
      prev.map((w) => (w.id === id ? { ...w, state: 'minimized', focused: false } : w))
    );
  }, []);

  const maximizeWindow = useCallback(
    (id: string, container: { width: number; height: number }) => {
      setWindows((prev) =>
        prev.map((w) => {
          if (w.id !== id) return w;
          if (w.state === 'maximized') {
            // Restore from maximize
            return {
              ...w,
              state: 'normal',
              bounds: w.restoreBounds || w.bounds,
              restoreBounds: undefined,
            };
          }
          // Maximize
          return {
            ...w,
            state: 'maximized',
            restoreBounds: { ...w.bounds },
            bounds: { x: 0, y: 0, width: container.width, height: container.height },
          };
        })
      );
    },
    []
  );

  const snapWindow = useCallback(
    (id: string, zone: SnapZone, container: { width: number; height: number }) => {
      setWindows((prev) =>
        prev.map((w) => {
          if (w.id !== id) return w;
          const snapBounds = computeSnapBounds(zone, container);
          return {
            ...w,
            state: 'snapped',
            snapZone: zone,
            restoreBounds: w.state !== 'snapped' && w.state !== 'maximized' ? { ...w.bounds } : w.restoreBounds,
            bounds: snapBounds,
            focused: true,
          };
        })
      );
    },
    []
  );

  const moveWindow = useCallback((id: string, x: number, y: number) => {
    setWindows((prev) =>
      prev.map((w) => (w.id === id ? { ...w, bounds: { ...w.bounds, x, y }, state: 'normal' } : w))
    );
  }, []);

  const resizeWindow = useCallback((id: string, bounds: Bounds) => {
    setWindows((prev) =>
      prev.map((w) => (w.id === id ? { ...w, bounds, state: 'normal' } : w))
    );
  }, []);

  const switchWorkspace = useCallback((workspaceId: string) => {
    setActiveWorkspaceId(workspaceId);
  }, []);

  return {
    windows,
    activeWorkspaceId,
    workspaces,
    openApp,
    focusWindow,
    closeWindow,
    minimizeWindow,
    maximizeWindow,
    snapWindow,
    moveWindow,
    resizeWindow,
    switchWorkspace,
  };
}
