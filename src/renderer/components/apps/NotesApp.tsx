import React, { useState, useEffect } from 'react';
import { Plus, Trash2, Search, Check, Download } from 'lucide-react';
import { Button } from '../common/Button';
import { Badge } from '../common/Badge';

export interface Note {
  id: string;
  title: string;
  content: string;
  category: string;
  updatedAt: number;
}

const DEFAULT_NOTES: Note[] = [
  {
    id: 'note-1',
    title: 'Welcome to NovaOS Notes',
    content: `# Welcome to NovaOS Notes\n\nThis application stores notes locally and autosaves your edits in real time.\n\n### Hybrid Architecture Highlights:\n- **Windows Productivity**: Snapping, multitasking, and clear discovery.\n- **macOS Elegance**: Fluid ergonomics and focused single-window workflows.\n- **Linux Agency**: Transparent permissions and terminal capabilities.`,
    category: 'Getting Started',
    updatedAt: Date.now() - 3600000,
  },
  {
    id: 'note-2',
    title: 'System Architecture Checklist',
    content: `- [x] Context isolation verified\n- [x] Node integration disabled\n- [x] Chromium sandbox active\n- [x] Strict CSP enforcement\n- [x] SQLite state persistence`,
    category: 'Architecture',
    updatedAt: Date.now() - 1800000,
  },
];

export const NotesApp: React.FC = () => {
  const [notes, setNotes] = useState<Note[]>(() => {
    try {
      const cached = typeof window !== 'undefined' && window.localStorage ? window.localStorage.getItem('novaos_notes') : null;
      return cached ? JSON.parse(cached) : DEFAULT_NOTES;
    } catch {
      return DEFAULT_NOTES;
    }
  });
  const [activeNoteId, setActiveNoteId] = useState<string>(notes[0]?.id || '');
  const [searchQuery, setSearchQuery] = useState('');
  const [savedStatus, setSavedStatus] = useState(true);

  const activeNote = notes.find((n) => n.id === activeNoteId) || notes[0];

  useEffect(() => {
    try {
      if (typeof window !== 'undefined' && window.localStorage) {
        window.localStorage.setItem('novaos_notes', JSON.stringify(notes));
      }
    } catch {
      // Ignore
    }
  }, [notes]);

  const handleCreateNote = () => {
    const newNote: Note = {
      id: `note-${Date.now()}`,
      title: 'Untitled Note',
      content: '',
      category: 'General',
      updatedAt: Date.now(),
    };
    setNotes((prev) => [newNote, ...prev]);
    setActiveNoteId(newNote.id);
  };

  const handleDeleteNote = (id: string) => {
    setNotes((prev) => {
      const next = prev.filter((n) => n.id !== id);
      if (activeNoteId === id && next.length > 0) {
        setActiveNoteId(next[0].id);
      }
      return next;
    });
  };

  const handleUpdateTitle = (title: string) => {
    if (!activeNote) return;
    setNotes((prev) =>
      prev.map((n) => (n.id === activeNote.id ? { ...n, title, updatedAt: Date.now() } : n))
    );
    setSavedStatus(false);
    setTimeout(() => setSavedStatus(true), 600);
  };

  const handleUpdateContent = (content: string) => {
    if (!activeNote) return;
    setNotes((prev) =>
      prev.map((n) => (n.id === activeNote.id ? { ...n, content, updatedAt: Date.now() } : n))
    );
    setSavedStatus(false);
    setTimeout(() => setSavedStatus(true), 600);
  };

  const handleExportToVirtualFs = async () => {
    if (!activeNote || !window.nova?.fs?.writeFile) return;
    const sanitizedTitle = activeNote.title.replace(/[^a-zA-Z0-9_-]/g, '_');
    const path = `nova://documents/${sanitizedTitle}.md`;
    await window.nova.fs.writeFile(path, activeNote.content);
    alert(`Exported to ${path}`);
  };

  const filteredNotes = notes.filter(
    (n) =>
      n.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      n.content.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div style={{ display: 'flex', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Notes Sidebar */}
      <div
        style={{
          width: '240px',
          backgroundColor: 'var(--nova-surface-base)',
          borderRight: '1px solid var(--nova-border-subtle)',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {/* Sidebar Header */}
        <div style={{ padding: '12px', borderBottom: '1px solid var(--nova-border-subtle)', display: 'flex', gap: '8px', alignItems: 'center' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', backgroundColor: 'var(--nova-surface-raised)', padding: '4px 8px', borderRadius: 'var(--nova-radius-md)', flex: 1 }}>
            <Search size={13} color="var(--nova-text-muted)" />
            <input
              type="text"
              placeholder="Search notes..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{ fontSize: 'var(--nova-text-xs)', width: '100%' }}
            />
          </div>
          <Button size="sm" variant="primary" icon={<Plus size={14} />} onClick={handleCreateNote}>
            New
          </Button>
        </div>

        {/* Notes List */}
        <div style={{ flex: 1, overflowY: 'auto', padding: '8px' }}>
          {filteredNotes.map((note) => {
            const isActive = note.id === activeNote?.id;
            return (
              <div
                key={note.id}
                onClick={() => setActiveNoteId(note.id)}
                style={{
                  padding: '10px 12px',
                  borderRadius: 'var(--nova-radius-md)',
                  backgroundColor: isActive ? 'var(--nova-surface-raised)' : 'transparent',
                  border: `1px solid ${isActive ? 'var(--nova-border-strong)' : 'transparent'}`,
                  cursor: 'pointer',
                  marginBottom: '4px',
                  transition: 'background-color var(--nova-duration-fast) var(--nova-ease-standard)',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '4px' }}>
                  <span style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600, color: 'var(--nova-text-primary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {note.title || 'Untitled Note'}
                  </span>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteNote(note.id);
                    }}
                    style={{ color: 'var(--nova-text-muted)', cursor: 'pointer', fontSize: '11px' }}
                  >
                    <Trash2 size={12} />
                  </button>
                </div>
                <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {note.content.substring(0, 50) || 'Empty note...'}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Note Editor Area */}
      {activeNote ? (
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          {/* Editor Header */}
          <div
            style={{
              height: '46px',
              borderBottom: '1px solid var(--nova-border-subtle)',
              backgroundColor: 'var(--nova-surface-base)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '0 16px',
            }}
          >
            <input
              type="text"
              value={activeNote.title}
              onChange={(e) => handleUpdateTitle(e.target.value)}
              placeholder="Note Title"
              style={{ fontSize: 'var(--nova-text-md)', fontWeight: 600, color: 'var(--nova-text-primary)', flex: 1 }}
            />
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <span style={{ fontSize: '11px', color: savedStatus ? '#10b981' : 'var(--nova-text-muted)', display: 'flex', alignItems: 'center', gap: '4px' }}>
                {savedStatus && <Check size={12} />} {savedStatus ? 'Autosaved' : 'Saving...'}
              </span>
              <Button size="sm" variant="subtle" icon={<Download size={13} />} onClick={handleExportToVirtualFs}>
                Export MD
              </Button>
              <Badge tier="simulated">Local SQLite</Badge>
            </div>
          </div>

          {/* Text Area */}
          <div style={{ flex: 1, padding: '16px', display: 'flex' }}>
            <textarea
              value={activeNote.content}
              onChange={(e) => handleUpdateContent(e.target.value)}
              placeholder="Write your note here using Markdown..."
              style={{
                width: '100%',
                height: '100%',
                resize: 'none',
                fontFamily: 'var(--nova-font-mono)',
                fontSize: 'var(--nova-text-sm)',
                lineHeight: 1.6,
                color: 'var(--nova-text-primary)',
                backgroundColor: 'transparent',
              }}
            />
          </div>
        </div>
      ) : (
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--nova-text-muted)' }}>
          No note selected
        </div>
      )}
    </div>
  );
};
