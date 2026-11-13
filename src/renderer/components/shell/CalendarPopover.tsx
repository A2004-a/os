import React, { useState } from 'react';
import { ChevronLeft, ChevronRight, Calendar as CalendarIcon, Clock } from 'lucide-react';

interface CalendarPopoverProps {
  isOpen: boolean;
  onClose: () => void;
  onOpenApp?: (appId: string, name: string) => void;
}

export const CalendarPopover: React.FC<CalendarPopoverProps> = ({
  isOpen,
  onClose,
  onOpenApp,
}) => {
  const [currentDate, setCurrentDate] = useState(() => new Date());
  const [selectedDate, setSelectedDate] = useState(() => new Date());

  if (!isOpen) return null;

  const year = currentDate.getFullYear();
  const month = currentDate.getMonth();

  const monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];

  const daysOfWeek = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];

  // Days in month calculation
  const firstDayIndex = new Date(year, month, 1).getDay();
  const totalDays = new Date(year, month + 1, 0).getDate();
  const prevMonthDays = new Date(year, month, 0).getDate();

  const handlePrevMonth = () => {
    setCurrentDate(new Date(year, month - 1, 1));
  };

  const handleNextMonth = () => {
    setCurrentDate(new Date(year, month + 1, 1));
  };

  const handleToday = () => {
    const today = new Date();
    setCurrentDate(today);
    setSelectedDate(today);
  };

  const isToday = (day: number) => {
    const today = new Date();
    return (
      today.getDate() === day &&
      today.getMonth() === month &&
      today.getFullYear() === year
    );
  };

  const isSelected = (day: number) => {
    return (
      selectedDate.getDate() === day &&
      selectedDate.getMonth() === month &&
      selectedDate.getFullYear() === year
    );
  };

  // Sample schedule items for the selected date
  const events = [
    { id: '1', time: '09:00 AM', title: 'Daily Engineering Standup', category: 'Work' },
    { id: '2', time: '11:30 AM', title: 'NovaOS Architecture Review', category: 'System' },
    { id: '3', time: '02:00 PM', title: 'Focus Sprint: Electron Hardening', category: 'Security' },
  ];

  return (
    <div
      role="dialog"
      aria-label="Calendar and Schedule"
      style={{
        position: 'fixed',
        bottom: '56px',
        right: '16px',
        width: '340px',
        backgroundColor: 'var(--nova-surface-glass)',
        backdropFilter: 'var(--nova-blur-glass)',
        border: '1px solid var(--nova-border-strong)',
        borderRadius: 'var(--nova-radius-xl)',
        boxShadow: 'var(--nova-shadow-lg)',
        zIndex: 'var(--nova-z-popover)' as any,
        padding: '16px',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
        userSelect: 'none',
        color: 'var(--nova-text-primary)',
        animation: 'fadeIn var(--nova-duration-fast) var(--nova-ease-standard)',
      }}
      onClick={(e) => e.stopPropagation()}
    >
      {/* Popover Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <CalendarIcon size={18} color="var(--nova-accent-primary)" />
          <span style={{ fontSize: 'var(--nova-text-md)', fontWeight: 600 }}>
            {monthNames[month]} {year}
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <button
            onClick={handlePrevMonth}
            aria-label="Previous Month"
            style={{
              padding: '6px',
              borderRadius: 'var(--nova-radius-sm)',
              cursor: 'pointer',
              color: 'var(--nova-text-secondary)',
              display: 'flex',
              alignItems: 'center',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <ChevronLeft size={16} />
          </button>
          <button
            onClick={handleToday}
            aria-label="Go to Today"
            style={{
              padding: '4px 8px',
              fontSize: '11px',
              fontWeight: 600,
              borderRadius: 'var(--nova-radius-sm)',
              cursor: 'pointer',
              color: 'var(--nova-accent-primary)',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            Today
          </button>
          <button
            onClick={handleNextMonth}
            aria-label="Next Month"
            style={{
              padding: '6px',
              borderRadius: 'var(--nova-radius-sm)',
              cursor: 'pointer',
              color: 'var(--nova-text-secondary)',
              display: 'flex',
              alignItems: 'center',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <ChevronRight size={16} />
          </button>
        </div>
      </div>

      {/* Days of Week Header */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', textAlign: 'center', gap: '2px' }}>
        {daysOfWeek.map((day) => (
          <span
            key={day}
            style={{
              fontSize: '11px',
              fontWeight: 600,
              color: 'var(--nova-text-muted)',
              padding: '4px 0',
            }}
          >
            {day}
          </span>
        ))}
      </div>

      {/* Calendar Days Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: '4px', textAlign: 'center' }}>
        {/* Previous Month trailing days */}
        {Array.from({ length: firstDayIndex }).map((_, i) => (
          <span
            key={`prev-${i}`}
            style={{
              fontSize: '12px',
              color: 'var(--nova-text-muted)',
              opacity: 0.35,
              padding: '6px 0',
            }}
          >
            {prevMonthDays - firstDayIndex + i + 1}
          </span>
        ))}

        {/* Current Month days */}
        {Array.from({ length: totalDays }).map((_, i) => {
          const day = i + 1;
          const today = isToday(day);
          const selected = isSelected(day);

          return (
            <button
              key={`day-${day}`}
              onClick={() => setSelectedDate(new Date(year, month, day))}
              style={{
                fontSize: '12px',
                fontWeight: today || selected ? 700 : 400,
                padding: '6px 0',
                borderRadius: '50%',
                backgroundColor: selected
                  ? 'var(--nova-accent-primary)'
                  : today
                  ? 'var(--nova-surface-raised)'
                  : 'transparent',
                color: selected
                  ? 'var(--nova-text-inverse)'
                  : today
                  ? 'var(--nova-accent-primary)'
                  : 'var(--nova-text-primary)',
                border: today && !selected ? '1px solid var(--nova-accent-primary)' : '1px solid transparent',
                cursor: 'pointer',
                transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
              }}
              onMouseEnter={(e) => {
                if (!selected) e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)';
              }}
              onMouseLeave={(e) => {
                if (!selected && !today) e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              {day}
            </button>
          );
        })}
      </div>

      {/* Divider */}
      <div style={{ height: '1px', backgroundColor: 'var(--nova-border-subtle)', margin: '4px 0' }} />

      {/* Selected Day Agenda */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontSize: '12px', fontWeight: 600, color: 'var(--nova-text-secondary)' }}>
            Agenda for {selectedDate.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}
          </span>
          {onOpenApp && (
            <button
              onClick={() => {
                onOpenApp('io.novaos.clock', 'Clock & Timers');
                onClose();
              }}
              style={{ fontSize: '11px', color: 'var(--nova-accent-primary)', cursor: 'pointer' }}
            >
              Open Clock
            </button>
          )}
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', maxHeight: '110px', overflowY: 'auto' }}>
          {events.map((ev) => (
            <div
              key={ev.id}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '6px 8px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: 'var(--nova-surface-base)',
                border: '1px solid var(--nova-border-subtle)',
                fontSize: '11px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                <Clock size={12} color="var(--nova-accent-primary)" />
                <span style={{ fontWeight: 500 }}>{ev.title}</span>
              </div>
              <span style={{ color: 'var(--nova-text-muted)' }}>{ev.time}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
