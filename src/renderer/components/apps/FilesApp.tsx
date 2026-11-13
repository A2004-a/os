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
  ArrowRight,
  RefreshCw,
  ExternalLink,
  Grid,
  List,
  X,
  Eye,
} from 'lucide-react';
import { Badge } from '../common/Badge';
import { Button } from '../common/Button';

interface TabItem {
  id: string;
  title: string;
  path: string;
}

export const FilesApp: React.FC = () => {
  const [tabs, setTabs] = useState<TabItem[]>([
    { id: 'tab-1', title: 'Documents', path: 'nova://documents' },
  ]);
  const [activeTabId, setActiveTabId] = useState<string>('tab-1');

  const activeTab = tabs.find((t) => t.id === activeTabId) || tabs[0];
  const currentPath = activeTab.path;

  const [history, setHistory] = useState<string[]>(['nova://documents']);
  const [historyIdx, setHistoryIdx] = useState(0);

  const [items, setItems] = useState<FileItem[]>([]);
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(null);
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [newFileName, setNewFileName] = useState('');
  const [isCreatingFile, setIsCreatingFile] = useState(false);
  const [permittedHostFolders, setPermittedHostFolders] = useState<string[]>([]);
  const [viewMode, setViewMode] = useState<'grid' | 'list' | 'details'>('grid');

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
          // update active tab path
          setTabs((prev) =>
            prev.map((t) =>
              t.id === activeTabId
                ? { ...t, path: dirPath, title: dirPath.split(/[\\/]/).pop() || 'Folder' }
                : t
            )
          );
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
  }, [activeTabId]);

  const handleNavigate = (path: string) => {
    const nextHist = history.slice(0, historyIdx + 1);
    nextHist.push(path);
    setHistory(nextHist);
    setHistoryIdx(nextHist.length - 1);
    loadDirectory(path);
  };

  const handleBack = () => {
    if (historyIdx > 0) {
      const prevIdx = historyIdx - 1;
      setHistoryIdx(prevIdx);
      loadDirectory(history[prevIdx]);
    }
  };

  const handleForward = () => {
    if (historyIdx < history.length - 1) {
      const nextIdx = historyIdx + 1;
      setHistoryIdx(nextIdx);
      loadDirectory(history[nextIdx]);
    }
  };

  const handleNewTab = () => {
    const newTabId = `tab-${Date.now()}`;
    setTabs((prev) => [...prev, { id: newTabId, title: 'Documents', path: 'nova://documents' }]);
    setActiveTabId(newTabId);
  };

  const handleCloseTab = (tabId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (tabs.length <= 1) return;
    const remaining = tabs.filter((t) => t.id !== tabId);
    setTabs(remaining);
    if (activeTabId === tabId) {
      setActiveTabId(remaining[0].id);
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

  // Breadcrumb path parts
  const breadcrumbSegments = currentPath.startsWith('nova://')
    ? currentPath.replace('nova://', '').split('/').filter(Boolean)
    : currentPath.split(/[\\/]/).filter(Boolean);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Top Tab Bar (Windows 11-Style Tabs) */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          backgroundColor: 'var(--nova-surface-raised)',
          borderBottom: '1px solid var(--nova-border-subtle)',
          padding: '4px 8px 0 8px',
          gap: '4px',
        }}
      >
        {tabs.map((tab) => {
          const isActive = tab.id === activeTabId;
          return (
            <div
              key={tab.id}
              onClick={() => setActiveTabId(tab.id)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '6px 12px',
                borderRadius: 'var(--nova-radius-md) var(--nova-radius-md) 0 0',
                backgroundColor: isActive ? 'var(--nova-surface-base)' : 'transparent',
                border: `1px solid ${isActive ? 'var(--nova-border-subtle)' : 'transparent'}`,
                borderBottom: isActive ? '1px solid var(--nova-surface-base)' : 'none',
                fontSize: '12px',
                fontWeight: isActive ? 600 : 400,
                color: isActive ? 'var(--nova-text-primary)' : 'var(--nova-text-muted)',
                cursor: 'pointer',
                maxWidth: '180px',
              }}
            >
              <Folder size={14} color="var(--nova-accent-primary)" />
              <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {tab.title}
              </span>
              {tabs.length > 1 && (
                <button
                  onClick={(e) => handleCloseTab(tab.id, e)}
                  style={{ padding: '2px', borderRadius: '3px', cursor: 'pointer', color: 'var(--nova-text-muted)' }}
                  onMouseEnter={(e) => (e.currentTarget.style.color = '#f43f5e')}
                  onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--nova-text-muted)')}
                >
                  <X size={12} />
                </button>
              )}
            </div>
          );
        })}
        <button
          onClick={handleNewTab}
          title="New Tab"
          aria-label="New Tab"
          style={{
            padding: '4px 8px',
            borderRadius: 'var(--nova-radius-sm)',
            cursor: 'pointer',
            color: 'var(--nova-text-secondary)',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
        >
          <Plus size={14} />
        </button>
      </div>

      {/* Main Body */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
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

        {/* Content Area */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          {/* Breadcrumb & Navigation Toolbar */}
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
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <button
                onClick={handleBack}
                disabled={historyIdx <= 0}
                style={{ padding: '4px', color: historyIdx > 0 ? 'var(--nova-text-secondary)' : 'var(--nova-text-muted)', cursor: historyIdx > 0 ? 'pointer' : 'default' }}
              >
                <ArrowLeft size={16} />
              </button>
              <button
                onClick={handleForward}
                disabled={historyIdx >= history.length - 1}
                style={{ padding: '4px', color: historyIdx < history.length - 1 ? 'var(--nova-text-secondary)' : 'var(--nova-text-muted)', cursor: historyIdx < history.length - 1 ? 'pointer' : 'default' }}
              >
                <ArrowRight size={16} />
              </button>
              <button
                onClick={() => loadDirectory(currentPath)}
                style={{ padding: '4px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
              >
                <RefreshCw size={14} />
              </button>

              {/* Breadcrumb Path Segments */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  backgroundColor: 'var(--nova-surface-raised)',
                  padding: '3px 8px',
                  borderRadius: 'var(--nova-radius-md)',
                  fontSize: 'var(--nova-text-xs)',
                }}
              >
                <span
                  onClick={() => handleNavigate(isVirtual ? 'nova://documents' : currentPath)}
                  style={{ cursor: 'pointer', color: 'var(--nova-accent-primary)', fontWeight: 600 }}
                >
                  {isVirtual ? 'Nova Home' : 'Host'}
                </span>
                {breadcrumbSegments.map((seg, i) => (
                  <React.Fragment key={i}>
                    <span style={{ color: 'var(--nova-text-muted)' }}>/</span>
                    <span style={{ color: 'var(--nova-text-primary)' }}>{seg}</span>
                  </React.Fragment>
                ))}
              </div>

              <Badge tier={isVirtual ? 'simulated' : 'native'}>
                {isVirtual ? 'Virtual Storage' : 'Permitted Host'}
              </Badge>
            </div>

            {/* View Mode & Search */}
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <div style={{ display: 'flex', backgroundColor: 'var(--nova-surface-raised)', borderRadius: 'var(--nova-radius-md)', padding: '2px' }}>
                <button
                  onClick={() => setViewMode('grid')}
                  title="Grid View"
                  style={{ padding: '4px 6px', borderRadius: '4px', backgroundColor: viewMode === 'grid' ? 'var(--nova-surface-base)' : 'transparent', color: viewMode === 'grid' ? 'var(--nova-accent-primary)' : 'var(--nova-text-muted)', cursor: 'pointer' }}
                >
                  <Grid size={14} />
                </button>
                <button
                  onClick={() => setViewMode('details')}
                  title="Details Table"
                  style={{ padding: '4px 6px', borderRadius: '4px', backgroundColor: viewMode === 'details' ? 'var(--nova-surface-base)' : 'transparent', color: viewMode === 'details' ? 'var(--nova-accent-primary)' : 'var(--nova-text-muted)', cursor: 'pointer' }}
                >
                  <List size={14} />
                </button>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', gap: '4px', backgroundColor: 'var(--nova-surface-raised)', padding: '4px 8px', borderRadius: 'var(--nova-radius-md)' }}>
                <Search size={13} color="var(--nova-text-muted)" />
                <input
                  type="text"
                  placeholder="Filter files..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  style={{ fontSize: 'var(--nova-text-xs)', width: '110px' }}
                />
              </div>

              {isVirtual && (
                <Button size="sm" variant="secondary" icon={<Plus size={12} />} onClick={() => setIsCreatingFile(true)}>
                  New
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

          {/* File Browser Grid / Details & Preview */}
          <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
            <div style={{ flex: selectedFile && !selectedFile.isDirectory ? 0.6 : 1, padding: '16px', overflowY: 'auto' }}>
              {loading ? (
                <div style={{ padding: '24px', color: 'var(--nova-text-muted)', fontSize: 'var(--nova-text-xs)' }}>Loading directory...</div>
              ) : filteredItems.length === 0 ? (
                <div style={{ padding: '24px', color: 'var(--nova-text-muted)', fontSize: 'var(--nova-text-xs)' }}>Empty folder</div>
              ) : viewMode === 'grid' ? (
                /* GRID VIEW */
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(110px, 1fr))', gap: '12px', alignContent: 'flex-start' }}>
                  {filteredItems.map((item) => {
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
                          {item.isDirectory ? 'Folder' : `${item.sizeBytes} B`}
                        </span>
                      </div>
                    );
                  })}
                </div>
              ) : (
                /* DETAILS VIEW */
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 'var(--nova-text-xs)' }}>
                  <thead>
                    <tr style={{ borderBottom: '1px solid var(--nova-border-subtle)', color: 'var(--nova-text-muted)', textAlign: 'left' }}>
                      <th style={{ padding: '8px' }}>Name</th>
                      <th style={{ padding: '8px' }}>Type</th>
                      <th style={{ padding: '8px' }}>Size</th>
                      <th style={{ padding: '8px' }}>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredItems.map((item) => {
                      const isSelected = selectedFile?.id === item.id;
                      return (
                        <tr
                          key={item.id}
                          onClick={() => handleItemClick(item)}
                          style={{
                            backgroundColor: isSelected ? 'var(--nova-surface-raised)' : 'transparent',
                            cursor: 'pointer',
                            borderBottom: '1px solid var(--nova-border-subtle)',
                          }}
                        >
                          <td style={{ padding: '8px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                            {item.isDirectory ? <Folder size={16} color="#06b6d4" /> : <FileText size={16} color="#94a3b8" />}
                            <span>{item.name}</span>
                          </td>
                          <td style={{ padding: '8px', color: 'var(--nova-text-muted)' }}>{item.isDirectory ? 'Folder' : 'File'}</td>
                          <td style={{ padding: '8px', color: 'var(--nova-text-muted)' }}>{item.isDirectory ? '-' : `${item.sizeBytes} B`}</td>
                          <td style={{ padding: '8px' }}>
                            {isVirtual && !item.isDirectory && (
                              <button onClick={(e) => { e.stopPropagation(); handleDelete(item); }} style={{ color: '#f43f5e', cursor: 'pointer' }}>
                                <Trash2 size={13} />
                              </button>
                            )}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              )}
            </div>

            {/* File Preview Pane */}
            {selectedFile && !selectedFile.isDirectory && (
              <div style={{ flex: 0.4, borderLeft: '1px solid var(--nova-border-subtle)', backgroundColor: 'var(--nova-surface-base)', display: 'flex', flexDirection: 'column' }}>
                <div style={{ padding: '12px', borderBottom: '1px solid var(--nova-border-subtle)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <Eye size={14} color="var(--nova-accent-primary)" />
                    <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>{selectedFile.name}</span>
                  </div>
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

          {/* Bottom Status Bar: Storage Breakdown Insights */}
          <div
            style={{
              height: '28px',
              borderTop: '1px solid var(--nova-border-subtle)',
              backgroundColor: 'var(--nova-surface-base)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '0 12px',
              fontSize: '11px',
              color: 'var(--nova-text-muted)',
            }}
          >
            <span>{filteredItems.length} items ({isVirtual ? 'Virtual Nova Home' : 'Host File System'})</span>
            <span>Nova Storage: SQLite Virtual Sandbox • Host Access: User Approved</span>
          </div>
        </div>
      </div>
    </div>
  );
};
