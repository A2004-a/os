/**
 * NovaOS System Metrics & Notification Broker
 * Safely extracts host metrics and dispatches desktop notifications
 */

import * as os from 'node:os';
import { Notification, shell } from 'electron';
import { SystemMetrics } from '@shared/types';

export class SystemBroker {
  private lastCpuMeasure: { idle: number; total: number } | null = null;

  public getSystemMetrics(): SystemMetrics {
    const cpus = os.cpus();
    let idle = 0;
    let total = 0;

    for (const cpu of cpus) {
      idle += cpu.times.idle;
      total += cpu.times.user + cpu.times.nice + cpu.times.sys + cpu.times.idle + cpu.times.irq;
    }

    let cpuPercent = 0;
    if (this.lastCpuMeasure) {
      const idleDelta = idle - this.lastCpuMeasure.idle;
      const totalDelta = total - this.lastCpuMeasure.total;
      if (totalDelta > 0) {
        cpuPercent = Math.max(0, Math.min(100, Math.round(((totalDelta - idleDelta) / totalDelta) * 100)));
      }
    }
    this.lastCpuMeasure = { idle, total };

    return {
      platform: process.platform as 'win32' | 'darwin' | 'linux',
      arch: os.arch(),
      osRelease: os.release(),
      cpuModel: cpus[0]?.model || 'Generic Processor',
      cpuCount: cpus.length,
      cpuUsagePercent: cpuPercent,
      totalMemoryBytes: os.totalmem(),
      freeMemoryBytes: os.freemem(),
      uptimeSeconds: os.uptime(),
    };
  }

  public showNotification(title: string, message: string, urgency: 'low' | 'normal' | 'critical' = 'normal'): boolean {
    if (Notification.isSupported()) {
      const notification = new Notification({
        title,
        body: message,
        urgency,
        silent: urgency === 'low',
      });
      notification.show();
      return true;
    }
    return false;
  }

  public openExternal(safeUrl: string): Promise<void> {
    return shell.openExternal(safeUrl);
  }
}
