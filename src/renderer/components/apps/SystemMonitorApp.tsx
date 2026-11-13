import React, { useState, useEffect } from 'react';
import { Cpu, HardDrive, Clock, Monitor } from 'lucide-react';
import { SystemMetrics } from '@shared/types';
import { Badge } from '../common/Badge';

export const SystemMonitorApp: React.FC = () => {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [history, setHistory] = useState<number[]>([]);

  useEffect(() => {
    const fetchMetrics = async () => {
      if (window.nova?.system?.getMetrics) {
        try {
          const res = await window.nova.system.getMetrics();
          if (res.success && res.metrics) {
            setMetrics(res.metrics);
            setHistory((prev) => [...prev.slice(-20), res.metrics!.cpuUsagePercent]);
          }
        } catch {
          // Ignore
        }
      }
    };

    fetchMetrics();
    const interval = setInterval(fetchMetrics, 1500);
    return () => clearInterval(interval);
  }, []);

  const totalMemGb = metrics ? (metrics.totalMemoryBytes / (1024 ** 3)).toFixed(1) : '0';
  const freeMemGb = metrics ? (metrics.freeMemoryBytes / (1024 ** 3)).toFixed(1) : '0';
  const usedMemGb = metrics ? ((metrics.totalMemoryBytes - metrics.freeMemoryBytes) / (1024 ** 3)).toFixed(1) : '0';
  const memPercent = metrics ? Math.round(((metrics.totalMemoryBytes - metrics.freeMemoryBytes) / metrics.totalMemoryBytes) * 100) : 0;

  return (
    <div style={{ padding: '24px', height: '100%', overflowY: 'auto', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '20px' }}>
        <div>
          <h2 style={{ fontSize: 'var(--nova-text-lg)', fontWeight: 600 }}>System Monitor</h2>
          <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)' }}>
            Real-time host system resource telemetry
          </p>
        </div>
        <Badge tier="native">Native Metrics API</Badge>
      </div>

      {metrics ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Hardware Overview Cards */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px' }}>
            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--nova-accent-primary)', marginBottom: '8px' }}>
                <Cpu size={18} />
                <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>CPU UTILIZATION</span>
              </div>
              <div style={{ fontSize: '24px', fontWeight: 700 }}>{metrics.cpuUsagePercent}%</div>
              <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)', marginTop: '4px' }}>
                {metrics.cpuCount} Cores • {metrics.cpuModel.split('@')[0]}
              </div>
            </div>

            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#10b981', marginBottom: '8px' }}>
                <HardDrive size={18} />
                <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>MEMORY USAGE</span>
              </div>
              <div style={{ fontSize: '24px', fontWeight: 700 }}>{memPercent}%</div>
              <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)', marginTop: '4px' }}>
                {usedMemGb} GB used of {totalMemGb} GB ({freeMemGb} GB free)
              </div>
            </div>

            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#a855f7', marginBottom: '8px' }}>
                <Monitor size={18} />
                <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>PLATFORM HOST</span>
              </div>
              <div style={{ fontSize: '18px', fontWeight: 600, textTransform: 'capitalize' }}>
                {metrics.platform} ({metrics.arch})
              </div>
              <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)', marginTop: '4px' }}>
                Kernel: {metrics.osRelease}
              </div>
            </div>

            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#f59e0b', marginBottom: '8px' }}>
                <Clock size={18} />
                <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>HOST UPTIME</span>
              </div>
              <div style={{ fontSize: '20px', fontWeight: 700 }}>
                {Math.floor(metrics.uptimeSeconds / 3600)}h {Math.floor((metrics.uptimeSeconds % 3600) / 60)}m
              </div>
              <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)', marginTop: '4px' }}>
                Since system startup
              </div>
            </div>
          </div>

          {/* Real-time CPU Sparkline Chart */}
          <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
            <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600, color: 'var(--nova-text-secondary)', display: 'block', marginBottom: '12px' }}>
              CPU LOAD HISTORY (LAST 30 SECONDS)
            </span>
            <div style={{ display: 'flex', alignItems: 'flex-end', gap: '6px', height: '80px', paddingTop: '10px' }}>
              {history.map((val, idx) => (
                <div
                  key={idx}
                  style={{
                    flex: 1,
                    height: `${Math.max(8, val)}%`,
                    backgroundColor: 'var(--nova-accent-primary)',
                    borderRadius: '2px',
                    transition: 'height var(--nova-duration-normal) var(--nova-ease-standard)',
                  }}
                  title={`${val}%`}
                />
              ))}
            </div>
          </div>
        </div>
      ) : (
        <div style={{ padding: '36px', textAlign: 'center', color: 'var(--nova-text-muted)' }}>
          Reading system metrics from Electron bridge...
        </div>
      )}
    </div>
  );
};
