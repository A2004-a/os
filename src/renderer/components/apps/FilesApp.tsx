import React, { useState, useEffect } from 'react';
import { FileItem } from '@shared/types';
import {
  Folder,
  FileText,
  Plus,
  Trash2,
  HardDrive,
  Search,
  ArrowLeft,
  RefreshCw,
  ExternalLink,
} from 'lucide-react';
import { Badge } from '../common/Badge';
import { Button } from '../common/Button';

export const FilesApp: React.FC = () => {
  const [currentPath, setCurrentPath] = useState<string>('nova://documents');
  const [history, setHistory] = useState<string[]>(['nova://documents']);
  const [items, setItems] = useState<FileItem[]>([]);
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(null);
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [newFileName, setNewFileName] = useState('');
  const [isCreatingFile, setIsCreatingFile] = useState(false);
  const [permittedHostFolders, setPermittedHostFolders] = useState<string[]>([]);

  const loadDirectory = async (dirPath: string) => {
    setLoading(true);
    setError(null);
    setSelectedFile(null);
    setFileContent(null);

    try {
      if (window.nova?.fs?.list) {
        const res = await window.nova.fs.list(dirPath);
        if (res.success && res.items) {
          setItems(res.items);
          setCurrentPath(dirPath);
        } else {
          setError(res.error || 'Failed to list directory');
        }
      }
    } catch (err: unknown) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadDirectory(currentPath);
  }, []);

  const handleNavigate = (path: string) => {
    setHistory((prev) => [...prev, path]);
    loadDirectory(path);
  };

  const handleBack = () => {
    if (history.length > 1) {
      const nextHist = [...history];
      nextHist.pop();
      const prevPath = nextHist[nextHist.length - 1];
      setHistory(nextHist);
      loadDirectory(prevPath);
    }
  };

  const handleItemClick = async (item: FileItem) => {
    setSelectedFile(item);
    if (item.isDirectory) {
      handleNavigate(item.path);
    } else {
      try {
        if (window.nova?.fs?.readFile) {
          const res = await window.nova.fs.readFile(item.path);
          if (res.success && res.content !== undefined) {
            setFileContent(res.content);
          } else {
            setFileContent(`[Error reading file: ${res.error}]`);
          }
        }
      } catch (err: unknown) {
        setFileContent(`[Error: ${(err as Error).message}]`);
      }
    }
  };

  const handleCreateFile = async () => {
    if (!newFileName.trim()) return;
    const targetPath = `${currentPath}/${newFileName.trim()}`;
    try {
      if (window.nova?.fs?.writeFile) {
        const res = await window.nova.fs.writeFile(targetPath, '# New Document\nCreated inside NovaOS');
        if (res.success) {
          setNewFileName('');
          setIsCreatingFile(false);
          loadDirectory(currentPath);
        } else {
          setError(res.error || 'Failed to create file');
        }
      }
    } catch (err: unknown) {
      setError((err as Error).message);
    }
  };

  const handleDelete = async (item: FileItem) => {
    try {
      if (window.nova?.fs?.deleteFile) {
        const res = await window.nova.fs.deleteFile(item.path, false);
        if (res.success) {
          loadDirectory(currentPath);
        } else {
          setError(res.error || 'Failed to delete file');
        }
      }
    } catch (err: unknown) {
      setError((err as Error).message);
    }
  };

  const handleAuthorizeHostFolder = async () => {
    try {
      if (window.nova?.fs?.pickFolder) {
        const res = await window.nova.fs.pickFolder();
        if (res.success && res.folder) {
          setPermittedHostFolders((prev) => [...prev.filter((f) => f !== res.folder), res.folder!]);
          handleNavigate(res.folder);
        }
      }
    } catch (err: unknown) {
      setError((err as Error).message);
    }
  };

  const filteredItems = items.filter((item) =>
    item.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const isVirtual = currentPath.startsWith('nova://');

  return (
    <div style={{ display: 'flex', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Sidebar Navigation */}
      <div
        style={{
          width: '200px',
          backgroundColor: 'var(--nova-surface-base)',
          borderRight: '1px solid var(--nova-border-subtle)',
          padding: '12px 8px',
          display: 'flex',
          flexDirection: 'column',
          gap: '4px',
        }}
      >
        <div style={{ padding: '4px 8px', fontSize: '10px', fontWeight: 600, color: 'var(--nova-text-muted)', textTransform: 'uppercase' }}>
          Nova Home
        </div>
        {[
          { name: 'Documents', path: 'nova://documents' },
          { name: 'Downloads', path: 'nova://downloads' },
          { name: 'Projects', path: 'nova://projects' },
          { name: 'Media', path: 'nova://media' },
        ].map((loc) => (
          <button
            key={loc.path}
            onClick={() => handleNavigate(loc.path)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '6px 10px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: currentPath === loc.path ? 'var(--nova-surface-raised)' : 'transparent',
              color: currentPath === loc.path ? 'var(--nova-accent-primary)' : 'var(--nova-text-secondary)',
              fontSize: 'var(--nova-text-xs)',
              cursor: 'pointer',
              textAlign: 'left',
            }}
          >
            <Folder size={14} />
            {loc.name}
          </button>
        ))}

        <div style={{ height: '1px', backgroundColor: 'var(--nova-border-subtle)', margin: '8px 0' }} />

        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '4px 8px' }}>
          <span style={{ fontSize: '10px', fontWeight: 600, color: 'var(--nova-text-muted)', textTransform: 'uppercase' }}>
            Host Folders
          </span>
          <Badge tier="native">Native</Badge>
        </div>

        {permittedHostFolders.map((folder) => (
          <button
            key={folder}
            onClick={() => handleNavigate(folder)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '6px 10px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: currentPath === folder ? 'var(--nova-surface-raised)' : 'transparent',
              color: currentPath === folder ? '#10b981' : 'var(--nova-text-secondary)',
              fontSize: 'var(--nova-text-xs)',
              cursor: 'pointer',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            <HardDrive size={14} />
            <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>{folder.split(/[\\/]/).pop()}</span>
          </button>
        ))}

        <button
          onClick={handleAuthorizeHostFolder}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            padding: '6px 10px',
            marginTop: '4px',
            borderRadius: 'var(--nova-radius-md)',
            border: '1px dashed var(--nova-border-subtle)',
            color: 'var(--nova-text-muted)',
            fontSize: 'var(--nova-text-xs)',
            cursor: 'pointer',
          }}
        >
          <ExternalLink size={12} />
          + Authorize Folder
        </button>
      </div>

      {/* Main Content Area */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* Toolbar */}
        <div
          style={{
            height: '42px',
            borderBottom: '1px solid var(--nova-border-subtle)',
            backgroundColor: 'var(--nova-surface-base)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '0 12px',
            gap: '8px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <button
              onClick={handleBack}
              disabled={history.length <= 1}
              style={{ padding: '4px', color: history.length > 1 ? 'var(--nova-text-secondary)' : 'var(--nova-text-muted)', cursor: history.length > 1 ? 'pointer' : 'default' }}
            >
              <ArrowLeft size={16} />
            </button>
            <button
              onClick={() => loadDirectory(currentPath)}
              style={{ padding: '4px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
            >
              <RefreshCw size={14} />
            </button>
            <span style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-muted)', maxWidth: '320px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {currentPath}
            </span>
            <Badge tier={isVirtual ? 'simulated' : 'native'}>
              {isVirtual ? 'Virtual Home' : 'Permitted Host Folder'}
            </Badge>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '4px', backgroundColor: 'var(--nova-surface-raised)', padding: '4px 8px', borderRadius: 'var(--nova-radius-md)' }}>
              <Search size={13} color="var(--nova-text-muted)" />
              <input
                type="text"
                placeholder="Search in folder..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                style={{ fontSize: 'var(--nova-text-xs)', width: '120px' }}
              />
            </div>

            {isVirtual && (
              <Button size="sm" variant="secondary" icon={<Plus size={12} />} onClick={() => setIsCreatingFile(true)}>
                New File
              </Button>
            )}
          </div>
        </div>

        {/* New File Inline Bar */}
        {isCreatingFile && (
          <div style={{ padding: '8px 12px', backgroundColor: 'var(--nova-surface-raised)', borderBottom: '1px solid var(--nova-border-subtle)', display: 'flex', gap: '8px' }}>
            <input
              type="text"
              placeholder="Filename.md or note.txt"
              value={newFileName}
              onChange={(e) => setNewFileName(e.target.value)}
              autoFocus
              style={{ flex: 1, padding: '4px 8px', borderRadius: '4px', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-strong)', fontSize: 'var(--nova-text-xs)' }}
            />
            <Button size="sm" variant="primary" onClick={handleCreateFile}>Create</Button>
            <Button size="sm" variant="subtle" onClick={() => setIsCreatingFile(false)}>Cancel</Button>
          </div>
        )}

        {/* Error Display */}
        {error && (
          <div style={{ padding: '8px 12px', backgroundColor: 'rgba(244, 63, 94, 0.1)', color: '#f43f5e', fontSize: 'var(--nova-text-xs)', borderBottom: '1px solid rgba(244, 63, 94, 0.2)' }}>
            {error}
          </div>
        )}

        {/* File Browser Grid & Preview */}
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          {/* File Items Grid */}
          <div style={{ flex: selectedFile && !selectedFile.isDirectory ? 0.6 : 1, padding: '16px', overflowY: 'auto', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(110px, 1fr))', gap: '12px', alignContent: 'flex-start' }}>
            {loading ? (
              <div style={{ padding: '24px', color: 'var(--nova-text-muted)', fontSize: 'var(--nova-text-xs)' }}>Loading directory...</div>
            ) : filteredItems.length === 0 ? (
              <div style={{ padding: '24px', color: 'var(--nova-text-muted)', fontSize: 'var(--nova-text-xs)' }}>Empty folder</div>
            ) : (
              filteredItems.map((item) => {
                const isSelected = selectedFile?.id === item.id;
                return (
                  <div
                    key={item.id}
                    onClick={() => handleItemClick(item)}
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: '6px',
                      padding: '10px 6px',
                      borderRadius: 'var(--nova-radius-md)',
                      backgroundColor: isSelected ? 'var(--nova-surface-raised)' : 'transparent',
                      border: `1px solid ${isSelected ? 'var(--nova-border-focus)' : 'transparent'}`,
                      cursor: 'pointer',
                      textAlign: 'center',
                    }}
                  >
                    {item.isDirectory ? <Folder size={36} color="#06b6d4" /> : <FileText size={36} color="#94a3b8" />}
                    <span style={{ fontSize: '11px', color: 'var(--nova-text-primary)', wordBreak: 'break-all', lineHeight: 1.2 }}>
                      {item.name}
                    </span>
                    <span style={{ fontSize: '9px', color: 'var(--nova-text-muted)' }}>
                      {item.isDirectory ? 'Dir' : `${item.sizeBytes} B`}
                    </span>
                  </div>
                );
              })
            )}
          </div>

          {/* File Preview Sidebar */}
          {selectedFile && !selectedFile.isDirectory && (
            <div style={{ flex: 0.4, borderLeft: '1px solid var(--nova-border-subtle)', backgroundColor: 'var(--nova-surface-base)', display: 'flex', flexDirection: 'column' }}>
              <div style={{ padding: '12px', borderBottom: '1px solid var(--nova-border-subtle)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>{selectedFile.name}</span>
                {isVirtual && (
                  <button onClick={() => handleDelete(selectedFile)} title="Delete file" style={{ color: '#f43f5e', cursor: 'pointer' }}>
                    <Trash2 size={14} />
                  </button>
                )}
              </div>
              <div style={{ flex: 1, padding: '12px', overflowY: 'auto' }}>
                <pre style={{ fontFamily: 'var(--nova-font-mono)', fontSize: '11px', color: 'var(--nova-text-secondary)', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
                  {fileContent !== null ? fileContent : 'Loading preview...'}
                </pre>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
