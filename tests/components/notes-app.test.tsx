import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { NotesApp } from '../../src/renderer/components/apps/NotesApp';

describe('NotesApp Component', () => {
  it('renders default note and allows editing content', () => {
    render(<NotesApp />);

    expect(screen.getAllByText(/Welcome to NovaOS Notes/i).length).toBeGreaterThan(0);

    const textarea = screen.getByPlaceholderText(/Write your note here/i) as HTMLTextAreaElement;
    expect(textarea).toBeDefined();

    fireEvent.change(textarea, { target: { value: 'Updated note content in test' } });
    expect(textarea.value).toBe('Updated note content in test');
  });

  it('creates new untitled note on clicking New button', () => {
    render(<NotesApp />);

    const newBtn = screen.getByRole('button', { name: /new/i });
    fireEvent.click(newBtn);

    expect(screen.getByDisplayValue(/Untitled Note/i)).toBeDefined();
  });
});
