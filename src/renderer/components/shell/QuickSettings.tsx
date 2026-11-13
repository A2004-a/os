import React, { useState, useEffect } from 'react';
import { UserPreferences, SystemMetrics, ThemeMode, AccentColor, LauncherMode } from '@shared/types';
import {
  Sun,
  Moon,
  Eye,
  Zap,
  Sliders,
  Monitor,
  Volume2,
  VolumeX,
  Layers,
  BellOff,
  SunMedium,
} from 'lucide-react';
import { Badge } from '../common/Badge';

interface QuickSettingsProps {
  isOpen: boolean;
  onClose: () => void;
  preferences: UserPreferences;
  onSetTheme: (theme: ThemeMode) => void;
  onSetAccent: (accent: AccentColor) => void;
  onSetLauncherMode: (mode: LauncherMode) => void;
  onToggleReducedMotion: () => void;
  onToggleReducedTransparency: () => void;
  focusMode?: boolean;
  onToggleFocusMode?: () => void;
}

export const QuickSettings: React.FC<QuickSettingsProps> = ({
  isOpen,
  onClose,
  preferences,
  onSetTheme,
  onSetAccent,
  onSetLauncherMode,
  onToggleReducedMotion,
  onToggleReducedTransparency,
  focusMode = false,
  onToggleFocusMode,
}) => {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [volume, setVolume] = useState(80);
  const [isMuted, setIsMuted] = useState(false);
  const [brightness, setBrightness] = useState(100);

  useEffect(() => {
    if (isOpen && window.nova?.system?.getMetrics) {
      window.nova.system.getMetrics().then((res) => {
        if (res.success && res.metrics) {
          setMetrics(res.metrics);
        }
      }).catch(console.error);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const accents: AccentColor[] = ['cyan', 'blue', 'purple', 'amber', 'emerald', 'rose'];

  return (
    <div
      role="dialog"
      aria-label="Quick Settings Drawer"
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
          bottom: preferences.launcherMode === 'dock' || preferences.launcherMode === 'taskbar' ? '56px' : 'auto',
          top: preferences.launcherMode === 'panel' ? '42px' : 'auto',
          right: '16px',
          width: '390px',
          maxWidth: '92vw',
          backgroundColor: 'var(--nova-surface-glass)',
          backdropFilter: 'var(--nova-blur-glass)',
          border: '1px solid var(--nova-border-strong)',
          borderRadius: 'var(--nova-radius-xl)',
          boxShadow: 'var(--nova-shadow-lg)',
          padding: '20px',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px',
          animation: 'fadeIn var(--nova-duration-fast) var(--nova-ease-standard)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Sliders size={18} color="var(--nova-accent-primary)" />
            <h3 style={{ fontSize: 'var(--nova-text-md)', fontWeight: 600, color: 'var(--nova-text-primary)' }}>Quick Settings</h3>
          </div>
          <button
            onClick={onClose}
            aria-label="Close Quick Settings"
            style={{ fontSize: '14px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
          >
            ✕
          </button>
        </div>

        {/* Real Host Metrics (Native Integration) */}
        <div
          style={{
            padding: '10px 12px',
            borderRadius: 'var(--nova-radius-md)',
            backgroundColor: 'var(--nova-surface-raised)',
            border: '1px solid var(--nova-border-subtle)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '6px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <Monitor size={14} color="#10b981" />
              <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>Host Platform Telemetry</span>
            </div>
            <Badge tier="native">Native</Badge>
          </div>
          {metrics ? (
            <div style={{ fontSize: '11px', color: 'var(--nova-text-secondary)', display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px' }}>
              <div>OS: {metrics.platform} ({metrics.arch})</div>
              <div>CPU Load: {metrics.cpuUsagePercent}%</div>
              <div>Free RAM: {(metrics.freeMemoryBytes / (1024 * 1024 * 1024)).toFixed(1)} GB</div>
              <div>Cores: {metrics.cpuCount}</div>
            </div>
          ) : (
            <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)' }}>Querying hardware metrics...</div>
          )}
        </div>

        {/* Theme Selector (Simulated) */}
        <div>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '6px' }}>
            <span style={{ fontSize: '11px', fontWeight: 600, color: 'var(--nova-text-secondary)', textTransform: 'uppercase' }}>
              Surface Theme
            </span>
            <Badge tier="simulated">Simulated</Badge>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
            <button
              onClick={() => onSetTheme('dark')}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '6px',
                padding: '8px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: preferences.theme === 'dark' ? 'var(--nova-accent-primary)' : 'var(--nova-surface-raised)',
                color: preferences.theme === 'dark' ? 'var(--nova-text-inverse)' : 'var(--nova-text-primary)',
                fontSize: 'var(--nova-text-xs)',
                fontWeight: 500,
                cursor: 'pointer',
              }}
            >
              <Moon size={13} /> Dark
            </button>
            <button
              onClick={() => onSetTheme('light')}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '6px',
                padding: '8px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: preferences.theme === 'light' ? 'var(--nova-accent-primary)' : 'var(--nova-surface-raised)',
                color: preferences.theme === 'light' ? 'var(--nova-text-inverse)' : 'var(--nova-text-primary)',
                fontSize: 'var(--nova-text-xs)',
                fontWeight: 500,
                cursor: 'pointer',
              }}
            >
              <Sun size={13} /> Light
            </button>
            <button
              onClick={() => onSetTheme('high-contrast')}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '6px',
                padding: '8px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: preferences.theme === 'high-contrast' ? 'var(--nova-accent-primary)' : 'var(--nova-surface-raised)',
                color: preferences.theme === 'high-contrast' ? 'var(--nova-text-inverse)' : 'var(--nova-text-primary)',
                fontSize: 'var(--nova-text-xs)',
                fontWeight: 500,
                cursor: 'pointer',
              }}
            >
              <Eye size={13} /> Contrast
            </button>
          </div>
        </div>

        {/* Accent Color Palette */}
        <div>
          <span style={{ fontSize: '11px', fontWeight: 600, color: 'var(--nova-text-secondary)', display: 'block', marginBottom: '6px', textTransform: 'uppercase' }}>
            Accent Palette
          </span>
          <div style={{ display: 'flex', gap: '8px', justifyContent: 'space-between' }}>
            {accents.map((acc) => (
              <button
                key={acc}
                onClick={() => onSetAccent(acc)}
                title={acc}
                aria-label={`Select ${acc} accent`}
                style={{
                  width: '32px',
                  height: '32px',
                  borderRadius: '50%',
                  border: preferences.accent === acc ? '2px solid #ffffff' : '2px solid transparent',
                  cursor: 'pointer',
                  backgroundColor:
                    acc === 'cyan' ? '#06b6d4' :
                    acc === 'blue' ? '#3b82f6' :
                    acc === 'purple' ? '#a855f7' :
                    acc === 'amber' ? '#f59e0b' :
                    acc === 'emerald' ? '#10b981' : '#f43f5e',
                }}
              />
            ))}
          </div>
        </div>

        {/* Launcher Layout Mode */}
        <div>
          <span style={{ fontSize: '11px', fontWeight: 600, color: 'var(--nova-text-secondary)', display: 'block', marginBottom: '6px', textTransform: 'uppercase' }}>
            Launcher Layout
          </span>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
            {(['dock', 'taskbar', 'panel'] as LauncherMode[]).map((mode) => (
              <button
                key={mode}
                onClick={() => onSetLauncherMode(mode)}
                style={{
                  padding: '6px',
                  borderRadius: 'var(--nova-radius-md)',
                  backgroundColor: preferences.launcherMode === mode ? 'var(--nova-surface-raised)' : 'transparent',
                  border: `1px solid ${preferences.launcherMode === mode ? 'var(--nova-border-strong)' : 'var(--nova-border-subtle)'}`,
                  color: preferences.launcherMode === mode ? 'var(--nova-accent-primary)' : 'var(--nova-text-secondary)',
                  fontSize: 'var(--nova-text-xs)',
                  textTransform: 'capitalize',
                  cursor: 'pointer',
                }}
              >
                {mode === 'dock' ? 'Dock' : mode === 'taskbar' ? 'Taskbar' : 'Top Panel'}
              </button>
            ))}
          </div>
        </div>

        {/* Sliders for Volume & Display Brightness Simulation */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          {/* Volume */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <button
              onClick={() => setIsMuted(!isMuted)}
              style={{ cursor: 'pointer', color: isMuted ? '#f43f5e' : 'var(--nova-text-secondary)' }}
            >
              {isMuted ? <VolumeX size={16} /> : <Volume2 size={16} />}
            </button>
            <input
              type="range"
              min="0"
              max="100"
              value={isMuted ? 0 : volume}
              onChange={(e) => {
                setVolume(Number(e.target.value));
                if (isMuted) setIsMuted(false);
              }}
              style={{ flex: 1, accentColor: 'var(--nova-accent-primary)', cursor: 'pointer' }}
            />
            <span style={{ fontSize: '11px', width: '32px', textAlign: 'right', color: 'var(--nova-text-muted)' }}>
              {isMuted ? '0%' : `${volume}%`}
            </span>
          </div>

          {/* Brightness */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <SunMedium size={16} color="var(--nova-text-secondary)" />
            <input
              type="range"
              min="30"
              max="100"
              value={brightness}
              onChange={(e) => setBrightness(Number(e.target.value))}
              style={{ flex: 1, accentColor: 'var(--nova-accent-primary)', cursor: 'pointer' }}
            />
            <span style={{ fontSize: '11px', width: '32px', textAlign: 'right', color: 'var(--nova-text-muted)' }}>
              {brightness}%
            </span>
          </div>
        </div>

        {/* Quick Toggles: Focus Mode, Reduced Motion, Reduced Transparency */}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px' }}>
          {onToggleFocusMode && (
            <button
              onClick={onToggleFocusMode}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '6px',
                padding: '8px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: focusMode ? 'rgba(168, 85, 247, 0.2)' : 'var(--nova-surface-raised)',
                border: `1px solid ${focusMode ? 'var(--nova-accent-primary)' : 'var(--nova-border-subtle)'}`,
                color: focusMode ? 'var(--nova-accent-primary)' : 'var(--nova-text-primary)',
                fontSize: 'var(--nova-text-xs)',
                cursor: 'pointer',
              }}
            >
              <BellOff size={14} />
              {focusMode ? 'Focus Active' : 'Focus Mode'}
            </button>
          )}

          <button
            onClick={onToggleReducedMotion}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '6px',
              padding: '8px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: preferences.motionReduced ? 'rgba(245, 158, 11, 0.2)' : 'var(--nova-surface-raised)',
              border: `1px solid ${preferences.motionReduced ? '#f59e0b' : 'var(--nova-border-subtle)'}`,
              color: preferences.motionReduced ? '#f59e0b' : 'var(--nova-text-primary)',
              fontSize: 'var(--nova-text-xs)',
              cursor: 'pointer',
            }}
          >
            <Zap size={14} />
            {preferences.motionReduced ? 'Reduced Motion' : 'Smooth Motion'}
          </button>

          <button
            onClick={onToggleReducedTransparency}
            style={{
              gridColumn: 'span 2',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '6px',
              padding: '8px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: preferences.transparencyReduced ? 'rgba(6, 182, 212, 0.2)' : 'var(--nova-surface-raised)',
              border: `1px solid ${preferences.transparencyReduced ? 'var(--nova-accent-primary)' : 'var(--nova-border-subtle)'}`,
              color: preferences.transparencyReduced ? 'var(--nova-accent-primary)' : 'var(--nova-text-primary)',
              fontSize: 'var(--nova-text-xs)',
              cursor: 'pointer',
            }}
          >
            <Layers size={14} />
            {preferences.transparencyReduced ? 'Opaque Surfaces' : 'Acrylic / Mica Glass'}
          </button>
        </div>
      </div>
    </div>
  );
};
