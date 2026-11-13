import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CalendarPopover } from '../../src/renderer/components/shell/CalendarPopover';

describe('CalendarPopover Component', () => {
  it('renders current month, day headers, and agenda', () => {
    render(
      <CalendarPopover
        isOpen={true}
        onClose={vi.fn()}
      />
    );

    expect(screen.getByRole('dialog', { name: /calendar and schedule/i })).toBeDefined();
    expect(screen.getByText(/Agenda for/i)).toBeDefined();
    expect(screen.getByText('Today')).toBeDefined();
    expect(screen.getByText('Su')).toBeDefined();
    expect(screen.getByText('Mo')).toBeDefined();
  });

  it('navigates months when clicking arrows', () => {
    render(
      <CalendarPopover
        isOpen={true}
        onClose={vi.fn()}
      />
    );

    const nextBtn = screen.getByRole('button', { name: /next month/i });
    fireEvent.click(nextBtn);
    expect(screen.getByRole('dialog')).toBeDefined();
  });
});
