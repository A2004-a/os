import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { FilesApp } from '../../src/renderer/components/apps/FilesApp';

describe('FilesApp Component', () => {
  beforeEach(() => {
    (window as any).nova = {
      fs: {
        list: vi.fn().mockResolvedValue({
          success: true,
          items: [
            { id: 'f-1', name: 'Document1.md', path: 'nova://documents/Document1.md', isVirtual: true, isDirectory: false, sizeBytes: 120, modifiedMs: Date.now() },
            { id: 'd-1', name: 'SubFolder', path: 'nova://documents/SubFolder', isVirtual: true, isDirectory: true, sizeBytes: 0, modifiedMs: Date.now() },
          ],
        }),
        readFile: vi.fn().mockResolvedValue({
          success: true,
          content: '# Mock Document Content',
        }),
      },
    };
  });

  it('renders sidebar with Nova Home categories and host folder button', async () => {
    render(<FilesApp />);

    await waitFor(() => {
      expect(screen.getAllByText(/Nova Home/i).length).toBeGreaterThan(0);
      expect(screen.getAllByText(/Documents/i).length).toBeGreaterThan(0);
      expect(screen.getByText(/\+ Authorize Folder/i)).toBeDefined();
    });
  });

  it('loads and displays files from directory list', async () => {
    render(<FilesApp />);

    await waitFor(() => {
      expect(screen.getByText('Document1.md')).toBeDefined();
      expect(screen.getByText('SubFolder')).toBeDefined();
    });
  });
});
