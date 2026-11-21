import { useCallback, useEffect, useMemo, useState } from 'react';
import { Modal } from '@mantine/core';
import { IconEdit, IconPlus, IconSearch, IconTrash, IconX } from '@tabler/icons-react';
import { api } from '../../api/client';
import { AdminPageHeader } from './ui';
import { t } from '../../i18n';

export type FieldType = 'text' | 'email' | 'password' | 'number' | 'date' | 'textarea' | 'select' | 'checkbox' | 'color';

export interface FieldDef {
  name: string;
  label: string;
  type?: FieldType;
  required?: boolean;
  placeholder?: string;
  hint?: string;
  options?: { label: string; value: string | number }[];
  defaultValue?: any;
  colSpan?: 1 | 2;
  min?: number;
  step?: number;
  /** Property to read when editing (defaults to name). */
  from?: string;
  /** Transform the value before returning it in the payload. */
  map?: (value: any) => any;
  /** Hide the field in the create form. */
  createOnly?: boolean;
  editOnly?: boolean;
  /** Hide this field from the list table. */
  hideInTable?: boolean;
  /** Hide this field from the create/edit form. */
  formHidden?: boolean;
  render?: (value: any, record: any) => React.ReactNode;
}

interface ResourcePageProps {
  title: string;
  subtitle?: string;
  endpoint: string;
  idKey?: string;
  fields: FieldDef[];
  createLabel?: string;
  emptyLabel?: string;
  canCreate?: boolean;
  canEdit?: boolean;
  canDelete?: boolean;
  searchPlaceholder?: string;
  /** Extra searchable text for a row. */
  searchText?: (row: any) => string;
  /** Optional row action nodes rendered before edit/delete. */
  rowActions?: (row: any, reload: () => void) => React.ReactNode;
  toolbar?: React.ReactNode;
  /** Optional transform before POST/PUT. */
  transform?: (values: Record<string, any>, editing: any | null) => Record<string, any>;
  /** Endpoint overrides (edit includes `{id}`). */
  createEndpoint?: string;
  updateEndpoint?: string;
  deleteEndpoint?: string;
  /** Callback after write. */
  onChanged?: () => void;
  /** Render values in the table instead of the default renderer. */
  cell?: (field: FieldDef, row: any) => React.ReactNode;
  columns?: string[];
}

function rowValues(fields: FieldDef[], row: any | null): Record<string, any> {
  const values: Record<string, any> = {};
  fields.forEach((field) => {
    const raw = row ? row[field.from ?? field.name] : field.defaultValue;
    if (field.type === 'checkbox') values[field.name] = raw === true || raw === 1 || raw === '1';
    else if (field.type === 'number') values[field.name] = raw === null || raw === undefined || raw === '' ? '' : Number(raw);
    else values[field.name] = raw ?? '';
  });
  return values;
}

export default function ResourcePage({
  title,
  subtitle,
  endpoint,
  idKey = 'id',
  fields,
  createLabel = 'Create',
  emptyLabel = 'No records found.',
  canCreate = true,
  canEdit = true,
  canDelete = true,
  searchPlaceholder = 'Search...',
  searchText,
  rowActions,
  toolbar,
  transform,
  createEndpoint,
  updateEndpoint,
  deleteEndpoint,
  onChanged,
  cell,
  columns,
}: ResourcePageProps) {
  const [rows, setRows] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [search, setSearch] = useState('');
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<any | null>(null);
  const [values, setValues] = useState<Record<string, any>>(() => rowValues(fields, null));
  const [saving, setSaving] = useState(false);
  const [formError, setFormError] = useState('');

  const load = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const data = await api.adminGet(endpoint);
      setRows(Array.isArray(data) ? data : Array.isArray(data?.data) ? data.data : []);
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to load records.'));
      setRows([]);
    } finally {
      setLoading(false);
    }
  }, [endpoint]);

  useEffect(() => {
    load();
  }, [load]);

  const filtered = useMemo(() => {
    if (!search.trim()) return rows;
    const term = search.trim().toLowerCase();
    return rows.filter((row) => {
      const haystack = searchText
        ? searchText(row)
        : Object.values(row)
            .filter((value) => typeof value === 'string' || typeof value === 'number')
            .join(' ');
      return String(haystack).toLowerCase().includes(term);
    });
  }, [rows, search, searchText]);

  const visibleFields = useMemo(
    () => fields.filter((field) => !field.hideInTable && !field.editOnly && !field.createOnly),
    [fields],
  );

  const startCreate = () => {
    setEditing(null);
    setValues(rowValues(fields, null));
    setFormError('');
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setValues(rowValues(fields, row));
    setFormError('');
    setOpen(true);
  };

  const submit = async () => {
    setSaving(true);
    setFormError('');
    try {
      const missing = fields.find((field) => field.required && !values[field.name]);
      if (missing) {
        setFormError(t('The {field} field is required.', { field: t(missing.label).toLowerCase() }));
        setSaving(false);
        return;
      }
      const baseValues = { ...values };
      fields.forEach((field) => {
        if (field.map) baseValues[field.name] = field.map(baseValues[field.name]);
        if (field.type === 'checkbox') baseValues[field.name] = baseValues[field.name] ? 1 : 0;
      });
      const payload = transform ? transform(baseValues, editing) : baseValues;
      if (editing) {
        const url = (updateEndpoint ?? `${endpoint}/edit/{id}`).replace('{id}', String(editing[idKey]));
        await api.adminPut(url, payload);
        setMessage(t('The record has been updated successfully.'));
      } else {
        const url = createEndpoint ?? `${endpoint}/store`;
        await api.adminPost(url, payload);
        setMessage(t('The record has been created successfully.'));
      }
      setOpen(false);
      await load();
      onChanged?.();
    } catch (requestError: any) {
      const detail = requestError?.errors
        ? Object.values(requestError.errors).flat().join(' ')
        : requestError?.message;
      setFormError(detail || t('Unable to save the record.'));
    } finally {
      setSaving(false);
    }
  };

  const remove = async (row: any) => {
    if (!window.confirm(t('Delete this record? This action cannot be undone.'))) return;
    setError('');
    try {
      const url = (deleteEndpoint ?? `${endpoint}/{id}`).replace('{id}', String(row[idKey]));
      await api.adminDelete(url);
      setMessage(t('The record has been deleted successfully.'));
      await load();
      onChanged?.();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to delete the record.'));
    }
  };

  const shownColumns = columns
    ? fields.filter((field) => columns.includes(field.name))
    : visibleFields;

  return (
    <div>
      <AdminPageHeader
        title={title}
        subtitle={subtitle}
        actions={
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            {toolbar}
            {canCreate && (
              <button type="button" className="admin-button primary" onClick={startCreate}>
                <IconPlus size={15} /> {t(createLabel)}
              </button>
            )}
          </div>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card">
        <div className="admin-resource-toolbar">
          <div className="admin-datagrid-search">
            <span style={{ position: 'absolute', left: 11, display: 'flex', color: '#9ca3af' }}>
              <IconSearch size={15} />
            </span>
            <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder={t(searchPlaceholder)} />
            {search && (
              <button type="button" className="admin-input-clear" onClick={() => setSearch('')} aria-label={t('Clear search')}>
                <IconX size={13} />
              </button>
            )}
          </div>
          <div style={{ flex: 1 }} />
          <span className="admin-muted admin-small">{t('{count} record(s)', { count: filtered.length })}</span>
          <button type="button" className="admin-toolbar-button" onClick={load}>
            {t('Refresh')}
          </button>
        </div>

        <div className="admin-table-wrap" style={{ marginTop: 12, boxShadow: 'none' }}>
          <table className="admin-table">
            <thead>
              <tr>
                {shownColumns.map((field) => (
                  <th key={field.name}>{t(field.label)}</th>
                ))}
                {(canEdit || canDelete || rowActions) && <th className="actions">{t('Action')}</th>}
              </tr>
            </thead>
            <tbody>
              {loading ? (
                Array.from({ length: 5 }).map((_, index) => (
                  <tr key={`skeleton-${index}`}>
                    <td colSpan={shownColumns.length + 1}>
                      <div className="admin-skeleton-row" />
                    </td>
                  </tr>
                ))
              ) : filtered.length === 0 ? (
                <tr>
                  <td colSpan={shownColumns.length + 1}>
                    <div className="admin-empty">{t(emptyLabel)}</div>
                  </td>
                </tr>
              ) : (
                filtered.map((row) => (
                  <tr key={row[idKey]}>
                    {shownColumns.map((field) => {
                      const value = row[field.from ?? field.name];
                      return (
                        <td key={field.name}>
                          {field.render
                            ? field.render(value, row)
                            : cell
                              ? cell(field, row)
                              : field.type === 'checkbox'
                                ? <span className={`status-badge ${value ? 'green' : 'red'}`}>{value ? t('Active') : t('Inactive')}</span>
                                : value ?? <span className="admin-muted">—</span>}
                        </td>
                      );
                    })}
                    {(canEdit || canDelete || rowActions) && (
                      <td className="actions">
                        <div className="admin-row-actions">
                          {rowActions?.(row, load)}
                          {canEdit && (
                            <button type="button" className="admin-icon-action" onClick={() => startEdit(row)} title={t('Edit')}>
                              <IconEdit size={15} />
                            </button>
                          )}
                          {canDelete && (
                            <button type="button" className="admin-icon-action danger" onClick={() => remove(row)} title={t('Delete')}>
                              <IconTrash size={15} />
                            </button>
                          )}
                        </div>
                      </td>
                    )}
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      <Modal
        opened={open}
        onClose={() => setOpen(false)}
        title={editing ? t('Edit {item}', { item: t(title).replace(/s$/i, '').toLowerCase() }) : t(createLabel)}
        size="lg"
        centered
        overlayProps={{ backgroundOpacity: 0.45, blur: 2 }}
      >
        <div className="admin-modal-form">
          {formError && <div className="admin-alert danger">{formError}</div>}
          <div className="admin-form-grid">
            {fields
              .filter((field) => !field.formHidden && (editing ? !field.createOnly : !field.editOnly))
              .map((field) => (
                <label
                  key={field.name}
                  className="admin-field"
                  style={field.colSpan === 2 ? { gridColumn: '1 / -1' } : undefined}
                >
                  <span className="admin-field-label">
                    {t(field.label)} {field.required && <em>*</em>}
                  </span>
                  {field.type === 'textarea' ? (
                    <textarea
                      className="admin-input"
                      rows={4}
                      value={values[field.name] ?? ''}
                      placeholder={field.placeholder ? t(field.placeholder) : undefined}
                      onChange={(event) => setValues((current) => ({ ...current, [field.name]: event.target.value }))}
                    />
                  ) : field.type === 'select' ? (
                    <select
                      className="admin-input"
                      value={values[field.name] ?? ''}
                      onChange={(event) =>
                        setValues((current) => ({
                          ...current,
                          [field.name]: event.target.value === '' ? '' : event.target.value,
                        }))
                      }
                    >
                      <option value="">{field.placeholder ? t(field.placeholder) : t('Select…')}</option>
                      {(field.options ?? []).map((option) => (
                        <option key={String(option.value)} value={option.value}>
                          {t(option.label)}
                        </option>
                      ))}
                    </select>
                  ) : field.type === 'checkbox' ? (
                    <span className="admin-switch-row">
                      <input
                        type="checkbox"
                        checked={!!values[field.name]}
                        onChange={(event) => setValues((current) => ({ ...current, [field.name]: event.target.checked }))}
                      />
                      <span>{values[field.name] ? t('Active') : t('Inactive')}</span>
                    </span>
                  ) : (
                    <input
                      className="admin-input"
                      type={field.type ?? 'text'}
                      value={values[field.name] ?? ''}
                      placeholder={field.placeholder ? t(field.placeholder) : undefined}
                      min={field.min}
                      step={field.step}
                      onChange={(event) =>
                        setValues((current) => ({
                          ...current,
                          [field.name]: field.type === 'number' ? event.target.value : event.target.value,
                        }))
                      }
                    />
                  )}
                  {field.hint && <span className="admin-field-hint">{t(field.hint)}</span>}
                </label>
              ))}
          </div>
          <div className="admin-modal-actions">
            <button type="button" className="admin-button" onClick={() => setOpen(false)}>
              {t('Cancel')}
            </button>
            <button type="button" className="admin-button primary" disabled={saving} onClick={submit}>
              {saving ? t('Saving…') : editing ? t('Save changes') : t(createLabel)}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
