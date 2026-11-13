import React, { useState, useEffect } from 'react';
import { Clock as ClockIcon, Timer, Hourglass, Play, Pause, RotateCcw, Target, CheckCircle2 } from 'lucide-react';
import { Badge } from '../common/Badge';
import { Button } from '../common/Button';

export const ClockApp: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'world' | 'stopwatch' | 'timer' | 'focus'>('world');
  const [time, setTime] = useState(new Date());

  // Stopwatch state
  const [swTime, setSwTime] = useState(0);
  const [swRunning, setSwRunning] = useState(false);

  // Timer state
  const [timerSeconds, setTimerSeconds] = useState(300); // 5 mins
  const [timerRunning, setTimerRunning] = useState(false);

  // Focus Session state
  const [focusSeconds, setFocusSeconds] = useState(1500); // 25 mins Pomodoro
  const [focusRunning, setFocusRunning] = useState(false);
  const [completedSessions, setCompletedSessions] = useState(2);

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    let interval: any;
    if (swRunning) {
      interval = setInterval(() => setSwTime((prev) => prev + 10), 10);
    }
    return () => clearInterval(interval);
  }, [swRunning]);

  useEffect(() => {
    let interval: any;
    if (timerRunning && timerSeconds > 0) {
      interval = setInterval(() => setTimerSeconds((prev) => prev - 1), 1000);
    } else if (timerRunning && timerSeconds === 0) {
      setTimerRunning(false);
      if (window.nova?.system?.showNotification) {
        window.nova.system.showNotification('NovaOS Timer Completed', 'Your countdown timer has expired!', 'critical');
      }
    }
    return () => clearInterval(interval);
  }, [timerRunning, timerSeconds]);

  useEffect(() => {
    let interval: any;
    if (focusRunning && focusSeconds > 0) {
      interval = setInterval(() => setFocusSeconds((prev) => prev - 1), 1000);
    } else if (focusRunning && focusSeconds === 0) {
      setFocusRunning(false);
      setCompletedSessions((prev) => prev + 1);
      if (window.nova?.system?.showNotification) {
        window.nova.system.showNotification('Focus Session Complete!', 'Well done! Time for a 5-minute break.', 'normal');
      }
    }
    return () => clearInterval(interval);
  }, [focusRunning, focusSeconds]);

  const formatStopwatch = (ms: number) => {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    const centis = Math.floor((ms % 1000) / 10);
    return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}.${String(centis).padStart(2, '0')}`;
  };

  const formatTimer = (sec: number) => {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Tabs */}
      <div style={{ display: 'flex', borderBottom: '1px solid var(--nova-border-subtle)', backgroundColor: 'var(--nova-surface-base)', padding: '0 16px' }}>
        {[
          { id: 'world', label: 'World Clock', icon: <ClockIcon size={14} /> },
          { id: 'focus', label: 'Focus Sessions', icon: <Target size={14} /> },
          { id: 'stopwatch', label: 'Stopwatch', icon: <Timer size={14} /> },
          { id: 'timer', label: 'Timer', icon: <Hourglass size={14} /> },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as any)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '12px 16px',
              borderBottom: `2px solid ${activeTab === tab.id ? 'var(--nova-accent-primary)' : 'transparent'}`,
              color: activeTab === tab.id ? 'var(--nova-accent-primary)' : 'var(--nova-text-secondary)',
              fontSize: 'var(--nova-text-xs)',
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Body */}
      <div style={{ flex: 1, padding: '24px', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
        {activeTab === 'world' && (
          <div style={{ width: '100%', maxWidth: '480px', display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div style={{ textAlign: 'center', marginBottom: '16px' }}>
              <div style={{ fontSize: '48px', fontWeight: 700, fontFamily: 'var(--nova-font-mono)' }}>
                {time.toLocaleTimeString()}
              </div>
              <div style={{ fontSize: 'var(--nova-text-sm)', color: 'var(--nova-text-secondary)' }}>
                Local Time • {time.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric' })}
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              {[
                { city: 'London', offset: 0 },
                { city: 'New York', offset: -5 },
                { city: 'Tokyo', offset: 9 },
                { city: 'San Francisco', offset: -8 },
              ].map((c) => {
                const cityDate = new Date(time.getTime() + c.offset * 3600000);
                return (
                  <div key={c.city} style={{ padding: '12px', borderRadius: 'var(--nova-radius-md)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
                    <div style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-muted)' }}>{c.city}</div>
                    <div style={{ fontSize: 'var(--nova-text-md)', fontWeight: 600, fontFamily: 'var(--nova-font-mono)' }}>
                      {cityDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {activeTab === 'focus' && (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '20px', maxWidth: '420px', textAlign: 'center' }}>
            <div>
              <h3 style={{ fontSize: 'var(--nova-text-lg)', fontWeight: 700, color: 'var(--nova-text-primary)' }}>
                Focus Session (Pomodoro)
              </h3>
              <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-muted)', marginTop: '4px' }}>
                Achieve deep work with distraction-free intervals. Notifications silenced.
              </p>
            </div>

            <div style={{ fontSize: '64px', fontWeight: 700, fontFamily: 'var(--nova-font-mono)', color: 'var(--nova-accent-primary)' }}>
              {formatTimer(focusSeconds)}
            </div>

            <div style={{ display: 'flex', gap: '12px' }}>
              <Button size="lg" variant={focusRunning ? 'danger' : 'primary'} onClick={() => setFocusRunning(!focusRunning)}>
                {focusRunning ? <Pause size={18} /> : <Play size={18} />} {focusRunning ? 'Pause Focus' : 'Start Focus Session'}
              </Button>
              <Button size="lg" variant="secondary" onClick={() => { setFocusRunning(false); setFocusSeconds(1500); }}>
                <RotateCcw size={18} /> Reset (25m)
              </Button>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 14px', borderRadius: 'var(--nova-radius-md)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
              <CheckCircle2 size={16} color="#10b981" />
              <span style={{ fontSize: '12px', color: 'var(--nova-text-secondary)' }}>
                Today's Completed Sessions: <strong>{completedSessions}</strong> (50 mins focused)
              </span>
            </div>
          </div>
        )}

        {activeTab === 'stopwatch' && (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '24px' }}>
            <div style={{ fontSize: '56px', fontWeight: 700, fontFamily: 'var(--nova-font-mono)' }}>
              {formatStopwatch(swTime)}
            </div>
            <div style={{ display: 'flex', gap: '12px' }}>
              <Button size="lg" variant={swRunning ? 'danger' : 'primary'} onClick={() => setSwRunning(!swRunning)}>
                {swRunning ? <Pause size={18} /> : <Play size={18} />} {swRunning ? 'Pause' : 'Start'}
              </Button>
              <Button size="lg" variant="secondary" onClick={() => { setSwRunning(false); setSwTime(0); }}>
                <RotateCcw size={18} /> Reset
              </Button>
            </div>
          </div>
        )}

        {activeTab === 'timer' && (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '24px' }}>
            <div style={{ fontSize: '64px', fontWeight: 700, fontFamily: 'var(--nova-font-mono)' }}>
              {formatTimer(timerSeconds)}
            </div>
            <div style={{ display: 'flex', gap: '12px' }}>
              <Button size="lg" variant={timerRunning ? 'danger' : 'primary'} onClick={() => setTimerRunning(!timerRunning)}>
                {timerRunning ? <Pause size={18} /> : <Play size={18} />} {timerRunning ? 'Pause' : 'Start'}
              </Button>
              <Button size="lg" variant="secondary" onClick={() => { setTimerRunning(false); setTimerSeconds(300); }}>
                <RotateCcw size={18} /> 5 Min Reset
              </Button>
            </div>
            <Badge tier="native">Triggers Native Desktop Notification on Complete</Badge>
          </div>
        )}
      </div>
    </div>
  );
};
