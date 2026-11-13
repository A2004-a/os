import { useState, useEffect } from 'react';
import { UserPreferences, ThemeMode, AccentColor, LauncherMode } from '@shared/types';

const DEFAULT_PREFERENCES: UserPreferences = {
  theme: 'dark',
  accent: 'cyan',
  launcherMode: 'dock',
  motionReduced: false,
  transparencyReduced: false,
  scaleFactor: 1.0,
  activeWorkspaceId: 'ws-1',
  autoSnapThreshold: 20,
};

export function usePreferences() {
  const [prefs, setPrefs] = useState<UserPreferences>(() => {
    const cached = localStorage.getItem('novaos_preferences');
    return cached ? JSON.parse(cached) : DEFAULT_PREFERENCES;
  });

  useEffect(() => {
    // Sync with persistent SQLite database via preload bridge if available
    if (window.nova?.db?.getPreferences) {
      window.nova.db.getPreferences().then((res) => {
        if (res.success && res.preferences) {
          setPrefs(res.preferences);
        }
      }).catch(console.error);
    }
  }, []);

  useEffect(() => {
    // Apply theme and accent attributes to root HTML element
    const root = document.documentElement;
    root.setAttribute('data-theme', prefs.theme);
    root.setAttribute('data-accent', prefs.accent);
    root.setAttribute('data-reduced-motion', String(prefs.motionReduced));
    root.setAttribute('data-reduced-transparency', String(prefs.transparencyReduced));

    localStorage.setItem('novaos_preferences', JSON.stringify(prefs));

    // Save to SQLite
    if (window.nova?.db?.setPreferences) {
      window.nova.db.setPreferences(prefs).catch(console.error);
    }
  }, [prefs]);

  const setTheme = (theme: ThemeMode) => setPrefs((p) => ({ ...p, theme }));
  const setAccent = (accent: AccentColor) => setPrefs((p) => ({ ...p, accent }));
  const setLauncherMode = (launcherMode: LauncherMode) => setPrefs((p) => ({ ...p, launcherMode }));
  const toggleReducedMotion = () => setPrefs((p) => ({ ...p, motionReduced: !p.motionReduced }));
  const toggleReducedTransparency = () => setPrefs((p) => ({ ...p, transparencyReduced: !p.transparencyReduced }));

  return {
    preferences: prefs,
    setTheme,
    setAccent,
    setLauncherMode,
    toggleReducedMotion,
    toggleReducedTransparency,
  };
}
