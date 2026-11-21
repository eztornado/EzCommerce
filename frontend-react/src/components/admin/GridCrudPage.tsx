import { useState } from 'react';
import { Modal } from '@mantine/core';
import { IconPlus } from '@tabler/icons-react';
import { api } from '../../api/client';
import DataGrid from './DataGrid';
import { AdminPageHeader } from './ui';
import { t } from '../../i18n';
import type { FieldDef } from './ResourcePage';

interface GridCrudPageProps {
  title: string;
  subtitle?: string;
  path: string;
  createLabel?: string;
  fields?: FieldDef[];
  createEndpoint?: string;
  transform?: (values: Record<string, any>) => Record<string, any>;
  toolbar?: React.ReactNode;
  rowActions?: (record: any, reload: () => void) => React.ReactNode;
  onRowClick?: (record: any) => void;
  emptyLabel?: string;
  canCreate?: boolean;
  hideColumn?: string[];
  initialSort?: { column: string; order: 'asc' | 'desc' };
}

export default function GridCrudPage({
  title,
  subtitle,
  path,
  createLabel = 'Create',
  fields = [],
  createEndpoint,
  transform,
  toolbar,
  rowActions,
  onRowClick,
  emptyLabel,
  canCreate = true,
  hideColumn,
  initialSort,
}: GridCrudPageProps) {
  const [version, setVersion] = useState(0);
  const [open, setOpen] = useState(false);
  const [values, setValues] = useState<Record<string, any>>({});
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);

  const reload = () => setVersion((value) => value + 1);

  const startCreate = () => {
    const defaults: Record<string, any> = {};
    fields.forEach((field) => {
      defaults[field.name] = field.defaultValue ?? (field.type === 'checkbox' ? false : '');
    });
    setValues(defaults);
    setError('');
    setOpen(true);
  };

  const submit = async () => {
    setSaving(true);
    setError('');
    try {
      const missing = fields.find((field) => field.required && !values[field.name]);
      if (missing) {
        setError(t('The {field} field is required.', { field: t(missing.label).toLowerCase() }));
        setSaving(false);
        return;
      }
      const mapped: Record<string, any> = { ...values };
      fields.forEach((field) => {
        if (field.map) mapped[field.name] = field.map(mapped[field.name]);
        if (field.type === 'checkbox') mapped[field.name] = mapped[field.name] ? 1 : 0;
      });
      await api.adminPost(createEndpoint ?? `${path}/store`, transform ? transform(mapped) : mapped);
      setOpen(false);
      reload();
    } catch (requestError: any) {
      const detail = requestError?.errors
        ? Object.values(requestError.errors).flat().join(' ')
        : requestError?.message;
      setError(detail || t('Unable to create the record.'));
    } finally {
      setSaving(false);
    }
  };

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
      <div className="admin-card admin-card-pad">
        <DataGrid
          key={version}
          path={path}
          emptyLabel={emptyLabel ?? t('No {items} found.', { items: t(title).toLowerCase() })}
          actions={rowActions ? (record) => rowActions(record, reload) : undefined}
          onRowClick={onRowClick}
          hideColumn={hideColumn}
          initialSort={initialSort}
        />
      </div>

      <Modal opened={open} onClose={() => setOpen(false)} title={t(createLabel)} size="lg" centered>
        <div className="admin-modal-form">
          {error && <div className="admin-alert danger">{error}</div>}
          <div className="admin-form-grid">
            {fields.map((field) => (
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
                    placeholder={field.placeholder ? t(field.placeholder) : undefined}
                    value={values[field.name] ?? ''}
                    onChange={(event) => setValues((current) => ({ ...current, [field.name]: event.target.value }))}
                  />
                ) : field.type === 'select' ? (
                  <select
                    className="admin-input"
                    value={values[field.name] ?? ''}
                    onChange={(event) => setValues((current) => ({ ...current, [field.name]: event.target.value }))}
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
                    placeholder={field.placeholder ? t(field.placeholder) : undefined}
                    min={field.min}
                    step={field.step}
                    value={values[field.name] ?? ''}
                    onChange={(event) => setValues((current) => ({ ...current, [field.name]: event.target.value }))}
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
            <button type="button" className="admin-button primary" onClick={submit} disabled={saving}>
              {saving ? t('Saving…') : t(createLabel)}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
