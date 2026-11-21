import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  IconChevronDown,
  IconChevronLeft,
  IconChevronRight,
  IconDownload,
  IconFilter,
  IconRefresh,
  IconSearch,
  IconTrash,
  IconX,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { formatDate, formatLabel, money, statusColor, statusLabel } from '../../lib/format';
import { t } from '../../i18n';

export interface GridColumn {
  index: string;
  label: string;
  type: string;
  searchable: boolean;
  filterable: boolean;
  sortable: boolean;
  filterable_options?: { label: string; value: string | number }[] | null;
}

interface GridAction {
  title: string;
  method: string;
  url: string;
  action?: string;
}

interface DataGridProps {
  path: string;
  emptyLabel?: string;
  onRowClick?: (record: any) => void;
  actions?: (record: any) => React.ReactNode;
  toolbar?: React.ReactNode;
  externalSearch?: string;
  hideColumn?: string[];
  hideActions?: string[];
  initialSort?: { column: string; order: 'asc' | 'desc' };
}

export default function DataGrid({
  path,
  emptyLabel = 'No records found.',
  onRowClick,
  actions,
  toolbar,
  externalSearch,
  hideColumn = [],
  hideActions = [],
  initialSort,
}: DataGridProps) {
  const navigate = useNavigate();
  const [columns, setColumns] = useState<GridColumn[]>([]);
  const [rowActions, setRowActions] = useState<GridAction[]>([]);
  const [massActions, setMassActions] = useState<GridAction[]>([]);
  const [records, setRecords] = useState<any[]>([]);
  const [meta, setMeta] = useState<any>({ current_page: 1, last_page: 1, total: 0, per_page: 10, per_page_options: [10, 20, 30, 50, 100] });
  const [search, setSearch] = useState(externalSearch ?? '');
  const [debouncedSearch, setDebouncedSearch] = useState(externalSearch ?? '');
  const [filters, setFilters] = useState<Record<string, any>>({});
  const [filterOpen, setFilterOpen] = useState(false);
  const [sort, setSort] = useState(initialSort ?? { column: '', order: 'desc' as 'asc' | 'desc' });
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState(10);
  const [selected, setSelected] = useState<number[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [refreshKey, setRefreshKey] = useState(0);
  const [dateFilters, setDateFilters] = useState<Record<string, { from: string; to: string }>>({});

  useEffect(() => {
    setSearch(externalSearch ?? '');
    setDebouncedSearch(externalSearch ?? '');
    setPage(1);
  }, [externalSearch]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearch(search);
      setPage(1);
    }, 350);
    return () => window.clearTimeout(timer);
  }, [search]);

  useEffect(() => {
    setPage(1);
    setSelected([]);
  }, [path]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError('');

    const payloadFilters: Record<string, any> = { ...filters };
    if (debouncedSearch.trim()) payloadFilters.all = debouncedSearch.trim();
    Object.entries(dateFilters).forEach(([key, range]) => {
      if (range.from || range.to) payloadFilters[key] = [range.from || '1900-01-01', range.to || '2100-12-31'];
    });

    api
      .adminPost(path, {
        filters: payloadFilters,
        sort: { column: sort.column, order: sort.order },
        pagination: { page, per_page: perPage },
      })
      .then((data) => {
        if (cancelled) return;
        setColumns(data?.columns ?? []);
        setRowActions(data?.actions ?? []);
        setMassActions(data?.mass_actions ?? []);
        setRecords(data?.records ?? []);
        setMeta(data?.meta ?? { current_page: 1, last_page: 1, total: 0, per_page: perPage });
      })
      .catch((requestError) => {
        if (cancelled) return;
        setError(requestError?.message ?? t('Unable to load the data grid.'));
        setRecords([]);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [path, debouncedSearch, JSON.stringify(filters), JSON.stringify(dateFilters), sort.column, sort.order, page, perPage, refreshKey]);

  const visibleColumns = useMemo(
    () => columns.filter((column) => !hideColumn.includes(column.index)),
    [columns, hideColumn],
  );

  const filterColumns = useMemo(() => columns.filter((column) => column.filterable && column.index !== 'id'), [columns]);

  const visibleRowActions = useMemo(
    () => rowActions.filter((action) => !hideActions.includes(action.title)),
    [rowActions, hideActions],
  );

  const activeFilterCount = useMemo(
    () =>
      Object.values(filters).filter((value) => value !== '' && value !== null && value !== undefined).length +
      Object.values(dateFilters).filter((range) => range.from || range.to).length,
    [filters, dateFilters],
  );

  const resolveUrl = useCallback(
    (url: string, record?: any) => {
      let resolved = url.replace('{id}', String(record?.__primary ?? record?.id ?? ''));
      resolved = resolved.replace(/^\/api\//, '').replace(/^\//, '');
      return resolved;
    },
    [],
  );

  const runAction = async (action: GridAction, record?: any) => {
    const url = resolveUrl(action.url, record);
    const method = action.method.toUpperCase();
    if (method === 'GET') {
      if (action.url.startsWith('/api/')) {
        try {
          await api.openAdminDocument(action.url);
        } catch (requestError: any) {
          setError(requestError?.message ?? t('Unable to open the document.'));
        }
        return;
      }
      navigate(`/${url}`);
      return;
    }
    if (method === 'DELETE') {
      if (!window.confirm(t('Delete this record? This action cannot be undone.'))) return;
      try {
        await api.delete(url, true);
        setRefreshKey((key) => key + 1);
      } catch (requestError: any) {
        setError(requestError?.message ?? t('Unable to delete the record.'));
      }
      return;
    }
    try {
      await api.adminPost(url, { indices: [record?.__primary ?? record?.id].filter(Boolean) });
      setRefreshKey((key) => key + 1);
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to complete the action.'));
    }
  };

  const runMassAction = async (action: GridAction) => {
    if (!selected.length) return;
    if (action.action === 'delete' && !window.confirm(t('Delete {count} selected record(s)?', { count: selected.length }))) return;
    try {
      await api.adminPost(resolveUrl(action.url), { indices: selected });
      setSelected([]);
      setRefreshKey((key) => key + 1);
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to complete the action.'));
    }
  };

  const exportCsv = async () => {
    const payloadFilters: Record<string, any> = { ...filters };
    if (debouncedSearch.trim()) payloadFilters.all = debouncedSearch.trim();
    Object.entries(dateFilters).forEach(([key, range]) => {
      if (range.from || range.to) payloadFilters[key] = [range.from || '1900-01-01', range.to || '2100-12-31'];
    });
    try {
      await api.downloadGridCsv(path, {
        filters: payloadFilters,
        sort: { column: sort.column, order: sort.order },
        pagination: { page: 1, per_page: 10000 },
      });
    } catch {
      setError(t('Unable to export the CSV.'));
    }
  };

  const sortBy = (column: GridColumn) => {
    if (!column.sortable) return;
    setSort((current) =>
      current.column === column.index
        ? { column: column.index, order: current.order === 'asc' ? 'desc' : 'asc' }
        : { column: column.index, order: 'asc' },
    );
    setPage(1);
  };

  const toggleAll = () => {
    if (selected.length === records.length) setSelected([]);
    else setSelected(records.map((record) => record.__primary ?? record.id));
  };

  const pages = useMemo(() => {
    const total = Math.max(1, meta.last_page ?? 1);
    const current = meta.current_page ?? 1;
    const set = new Set<number>([1, total, current, current - 1, current + 1]);
    return Array.from(set).filter((value) => value >= 1 && value <= total).sort((a, b) => a - b);
  }, [meta]);

  const renderCell = (column: GridColumn, record: any) => {
    const value = record[column.index];
    if (value === null || value === undefined || value === '') {
      return <span style={{ color: '#9ca3af' }}>—</span>;
    }
    const option = column.filterable_options?.find((entry) => String(entry.value) === String(value));
    if (column.index === 'status') {
      const label = option?.label ? t(option.label) : column.type === 'boolean' ? statusLabel(value) : t(formatLabel(String(value)));
      const active = value === 1 || value === '1' || ['active', 'enabled', 'approved', 'completed', 'processing'].includes(label.toLowerCase());
      return <span className={`status-badge ${active ? statusColor(value) : 'gray'}`}>{label}</span>;
    }
    if (column.filterable_options?.length) {
      return <span className="admin-pill">{option?.label ? t(option.label) : t(formatLabel(String(value)))}</span>;
    }
    if (column.type === 'boolean') {
      const active = value === 1 || value === '1' || value === true;
      return <span className={`status-badge ${active ? 'green' : 'gray'}`}>{statusLabel(value)}</span>;
    }
    if (column.index === 'sku' || column.index === 'code') {
      return <span className="admin-code">{String(value)}</span>;
    }
    if (column.index === 'type') {
      return <span className="admin-pill">{t(formatLabel(String(value)))}</span>;
    }
    if (column.type === 'decimal' && typeof value === 'number') {
      return money(value);
    }
    if (column.index.includes('created_at') || column.index.includes('date') || column.type === 'date' || column.type === 'datetime') {
      const formatted = formatDate(String(value));
      return formatted || String(value);
    }
    if (column.index === 'name' && typeof value === 'string') {
      return <span style={{ color: '#111827', fontWeight: 500 }}>{value}</span>;
    }
    return <span>{String(value)}</span>;
  };

  return (
    <div>
      <div className="admin-datagrid-toolbar">
        <div className="admin-datagrid-search">
          <span style={{ position: 'absolute', left: 11, display: 'flex', color: '#9ca3af' }}>
            <IconSearch size={15} />
          </span>
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={t('Search...')}
            aria-label={t('Search records')}
          />
          {search && (
            <button type="button" className="admin-input-clear" onClick={() => setSearch('')} aria-label={t('Clear search')}>
              <IconX size={13} />
            </button>
          )}
        </div>

        <button
          type="button"
          className="admin-toolbar-button"
          data-active={filterOpen || activeFilterCount > 0}
          onClick={() => setFilterOpen((open) => !open)}
        >
          <IconFilter size={15} /> {t('Filters')}
          {activeFilterCount > 0 && <span className="admin-toolbar-count">{activeFilterCount}</span>}
        </button>

        {massActions?.map((action) => (
          <button
            key={action.title + action.url}
            type="button"
            className="admin-toolbar-button danger"
            disabled={!selected.length}
            onClick={() => runMassAction(action)}
          >
            <IconTrash size={15} /> {t(action.title)}
            {selected.length > 0 && ` (${selected.length})`}
          </button>
        ))}

        <div style={{ flex: 1 }} />

        {toolbar}

        <button type="button" className="admin-toolbar-button" onClick={() => setRefreshKey((key) => key + 1)}>
          <IconRefresh size={15} /> {t('Refresh')}
        </button>

        <button type="button" className="admin-toolbar-button" onClick={exportCsv}>
          <IconDownload size={15} /> {t('Export')}
        </button>
      </div>

      {filterOpen && (
        <div className="admin-filter-panel">
          <div className="admin-filter-grid">
            {filterColumns.map((column) => (
              <div key={column.index}>
                <label className="admin-filter-label">{t(column.label)}</label>
                {column.filterable_options ? (
                  <select
                    className="admin-input"
                    value={filters[column.index] ?? ''}
                    onChange={(event) =>
                      setFilters((current) => {
                        const next = { ...current };
                        if (event.target.value === '') delete next[column.index];
                        else next[column.index] = event.target.value;
                        return next;
                      })
                    }
                  >
                    <option value="">{t('All')}</option>
                    {column.filterable_options.map((option) => (
                      <option key={String(option.value)} value={option.value}>
                        {t(option.label)}
                      </option>
                    ))}
                  </select>
                ) : column.type === 'date' || column.type === 'datetime' ? (
                  <div style={{ display: 'flex', gap: 6 }}>
                    <input
                      type="date"
                      className="admin-input"
                      value={dateFilters[column.index]?.from ?? ''}
                      onChange={(event) =>
                        setDateFilters((current) => ({
                          ...current,
                          [column.index]: { from: event.target.value, to: current[column.index]?.to ?? '' },
                        }))
                      }
                    />
                    <input
                      type="date"
                      className="admin-input"
                      value={dateFilters[column.index]?.to ?? ''}
                      onChange={(event) =>
                        setDateFilters((current) => ({
                          ...current,
                          [column.index]: { from: current[column.index]?.from ?? '', to: event.target.value },
                        }))
                      }
                    />
                  </div>
                ) : (
                  <input
                    className="admin-input"
                    value={filters[column.index] ?? ''}
                    onChange={(event) =>
                      setFilters((current) => {
                        const next = { ...current };
                        if (event.target.value === '') delete next[column.index];
                        else next[column.index] = event.target.value;
                        return next;
                      })
                    }
                    placeholder={t('Filter by {column}', { column: t(column.label).toLowerCase() })}
                  />
                )}
              </div>
            ))}
          </div>

          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 14 }}>
            <button
              type="button"
              className="admin-button ghost"
              onClick={() => {
                setFilters({});
                setDateFilters({});
              }}
            >
              {t('Clear filters')}
            </button>
            <button type="button" className="admin-button primary" onClick={() => setFilterOpen(false)}>
              {t('Apply filters')}
            </button>
          </div>
        </div>
      )}

      {error && <div className="admin-alert danger">{error}</div>}

      <div className="admin-table-wrap">
        <table className="admin-table">
          <thead>
            <tr>
              <th style={{ width: 42 }}>
                <input
                  type="checkbox"
                  checked={records.length > 0 && selected.length === records.length}
                  onChange={toggleAll}
                  aria-label={t('Select all')}
                />
              </th>
              {visibleColumns.map((column) => (
                <th
                  key={column.index}
                  className={column.sortable ? 'sortable' : undefined}
                  onClick={() => sortBy(column)}
                >
                  {t(column.label)}
                  {sort.column === column.index && (
                    <span style={{ marginLeft: 6, fontSize: 10 }}>{sort.order === 'asc' ? '▲' : '▼'}</span>
                  )}
                </th>
              ))}
              {(visibleRowActions.length > 0 || actions) && <th className="actions">{t('Action')}</th>}
            </tr>
          </thead>
          <tbody>
            {loading ? (
              Array.from({ length: 6 }).map((_, index) => (
                <tr key={`skeleton-${index}`}>
                  <td colSpan={visibleColumns.length + 2}>
                    <div className="admin-skeleton-row" />
                  </td>
                </tr>
              ))
            ) : records.length === 0 ? (
              <tr>
                <td colSpan={visibleColumns.length + 2}>
                  <div className="admin-empty">{t(emptyLabel)}</div>
                </td>
              </tr>
            ) : (
              records.map((record, index) => {
                const primary = record.__primary ?? record.id ?? index;
                return (
                  <tr
                    key={primary}
                    onClick={onRowClick ? () => onRowClick(record) : undefined}
                    style={onRowClick ? { cursor: 'pointer' } : undefined}
                  >
                    <td onClick={(event) => event.stopPropagation()}>
                      <input
                        type="checkbox"
                        checked={selected.includes(primary)}
                        onChange={() =>
                          setSelected((current) =>
                            current.includes(primary) ? current.filter((value) => value !== primary) : [...current, primary],
                          )
                        }
                        aria-label={t('Select row')}
                      />
                    </td>
                    {visibleColumns.map((column) => (
                      <td key={column.index}>{renderCell(column, record)}</td>
                    ))}
                    {(visibleRowActions.length > 0 || actions) && (
                      <td className="actions" onClick={(event) => event.stopPropagation()}>
                        <div className="admin-row-actions">
                          {actions?.(record)}
                          {visibleRowActions.map((action) => (
                            <button
                              key={action.title + action.url}
                              type="button"
                              className={action.method.toUpperCase() === 'DELETE' ? 'admin-link danger' : 'admin-link'}
                              onClick={() => runAction(action, record)}
                            >
                              {t(action.title)}
                            </button>
                          ))}
                        </div>
                      </td>
                    )}
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      <div className="admin-pagination">
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span>
            {t('Showing {from}–{to} of {total} records', { from: meta.from ?? 0, to: meta.to ?? 0, total: meta.total ?? 0 })}
          </span>
          <select
            className="admin-input"
            style={{ width: 76, padding: '5px 8px' }}
            value={perPage}
            onChange={(event) => {
              setPerPage(Number(event.target.value));
              setPage(1);
            }}
          >
            {(meta.per_page_options ?? [10, 20, 30, 50, 100]).map((option: number) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          <button
            type="button"
            className="admin-page-button"
            disabled={(meta.current_page ?? 1) <= 1}
            onClick={() => setPage((current) => Math.max(1, current - 1))}
            aria-label={t('Previous page')}
          >
            <IconChevronLeft size={15} />
          </button>
          {pages.map((value, index) => (
            <span key={value} style={{ display: 'contents' }}>
              {index > 0 && value - pages[index - 1] > 1 && <span style={{ color: '#9ca3af' }}>…</span>}
              <button
                type="button"
                className="admin-page-button"
                data-active={value === meta.current_page}
                onClick={() => setPage(value)}
              >
                {value}
              </button>
            </span>
          ))}
          <button
            type="button"
            className="admin-page-button"
            disabled={(meta.current_page ?? 1) >= Math.max(1, meta.last_page ?? 1)}
            onClick={() => setPage((current) => Math.min(Math.max(1, meta.last_page ?? 1), current + 1))}
            aria-label={t('Next page')}
          >
            <IconChevronRight size={15} />
          </button>
          <span style={{ marginLeft: 6, display: 'inline-flex', alignItems: 'center', gap: 4 }}>
            <IconChevronDown size={13} /> {t('Page {page} / {last}', { page: meta.current_page ?? 1, last: Math.max(1, meta.last_page ?? 1) })}
          </span>
        </div>
      </div>
    </div>
  );
}
