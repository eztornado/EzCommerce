import { useEffect, useState } from 'react';
import { Modal } from '@mantine/core';
import { IconPlus, IconRefresh, IconSearch, IconTrash, IconX } from '@tabler/icons-react';
import { api } from '../../api/client';
import DataGrid from '../../components/admin/DataGrid';
import { AdminPageHeader } from '../../components/admin/ui';
import { t } from '../../i18n';

interface OptionRow {
  admin_name: string;
  swatch_value?: string;
  sort_order?: number;
}

const emptyAttribute = {
  code: '',
  admin_name: '',
  type: 'text',
  is_required: 0,
  is_unique: 0,
  is_filterable: 0,
  is_comparable: 0,
  is_configurable: 0,
  is_visible_on_front: 1,
  value_per_locale: 0,
  value_per_channel: 0,
  options: [] as OptionRow[],
};

const ATTRIBUTE_TYPES = ['text', 'textarea', 'price', 'boolean', 'select', 'multiselect', 'date', 'datetime', 'image', 'file'];

export function AttributesPage() {
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<number | null>(null);
  const [form, setForm] = useState<any>(emptyAttribute);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [gridKey, setGridKey] = useState(0);

  const startCreate = () => {
    setEditing(null);
    setForm({ ...emptyAttribute, options: [] });
    setError('');
    setOpen(true);
  };

  const startEdit = async (record: any) => {
    setError('');
    try {
      const data = await api.adminGet(`admin/catalog/attributes/edit/${record.__primary}`);
      const attribute = data?.attribute ?? {};
      setEditing(record.__primary);
      setForm({
        code: attribute.code ?? record.code,
        admin_name: attribute.admin_name ?? record.admin_name,
        type: attribute.type ?? record.type ?? 'text',
        is_required: Number(attribute.is_required ?? 0),
        is_unique: Number(attribute.is_unique ?? 0),
        is_filterable: Number(attribute.is_filterable ?? 0),
        is_comparable: Number(attribute.is_comparable ?? 0),
        is_configurable: Number(attribute.is_configurable ?? 0),
        is_visible_on_front: Number(attribute.is_visible_on_front ?? 1),
        value_per_locale: Number(attribute.value_per_locale ?? 0),
        value_per_channel: Number(attribute.value_per_channel ?? 0),
        options: (data?.options ?? []).map((option: any) => ({
          admin_name: option.admin_name,
          swatch_value: option.swatch_value ?? '',
          sort_order: option.sort_order ?? 0,
        })),
      });
      setOpen(true);
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to load the attribute.'));
    }
  };

  const save = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    try {
      if (editing) await api.adminPut(`admin/catalog/attributes/edit/${editing}`, form);
      else await api.adminPost('admin/catalog/attributes/store', form);
      setOpen(false);
      setMessage(editing ? t('Attribute updated.') : t('Attribute created.'));
      setGridKey((key) => key + 1);
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to save the attribute.'));
    }
  };

  const remove = async (record: any) => {
    if (!window.confirm(`Delete attribute “${record.admin_name}”?`)) return;
    try {
      await api.adminDelete(`admin/catalog/attributes/${record.__primary}`);
      setMessage(t('Attribute deleted.'));
      setGridKey((key) => key + 1);
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to delete the attribute.'));
    }
  };

  const toggleFlag = (field: string) => setForm((current: any) => ({ ...current, [field]: current[field] ? 0 : 1 }));

  return (
    <div>
      <AdminPageHeader
        title={t('Attributes')}
        subtitle={t('Define the product properties used by attribute families, filters and variants.')}
        actions={
          <button type="button" className="admin-button primary" onClick={startCreate}>
            <IconPlus size={15} /> {t('Create attribute')}
          </button>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && !open && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card admin-card-pad">
        <DataGrid
          key={gridKey}
          path="admin/catalog/attributes"
          emptyLabel={t('No attributes found.')}
          actions={(record) => (
            <>
              <button type="button" className="admin-link" onClick={() => startEdit(record)}>
                {t('Edit')}
              </button>
              <button type="button" className="admin-link danger" onClick={() => remove(record)}>
                {t('Delete')}
              </button>
            </>
          )}
        />
      </div>

      <Modal opened={open} onClose={() => setOpen(false)} title={editing ? t('Edit attribute') : t('New attribute')} centered size="xl">
        <form onSubmit={save}>
          <div className="admin-form-grid">
            <label className="admin-field">
              <span className="admin-field-label">{t('Code')} *</span>
              <input
                className="admin-input"
                value={form.code}
                onChange={(event) => setForm({ ...form, code: event.target.value })}
                disabled={!!editing}
                required
              />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Admin name')} *</span>
              <input className="admin-input" value={form.admin_name} onChange={(event) => setForm({ ...form, admin_name: event.target.value })} required />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Type')}</span>
              <select className="admin-input" value={form.type} onChange={(event) => setForm({ ...form, type: event.target.value })}>
                {ATTRIBUTE_TYPES.map((type) => (
                  <option key={type} value={type}>
                    {type}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="admin-flag-grid">
            {[
              ['is_required', 'Required'],
              ['is_unique', 'Unique'],
              ['is_filterable', 'Filterable'],
              ['is_comparable', 'Comparable'],
              ['is_configurable', 'Configurable'],
              ['is_visible_on_front', 'Visible on front'],
              ['value_per_locale', 'Value per locale'],
              ['value_per_channel', 'Value per channel'],
            ].map(([field, label]) => (
              <label className="filter-check" key={field}>
                <input type="checkbox" checked={!!form[field]} onChange={() => toggleFlag(field)} />
                <span>{t(label)}</span>
              </label>
            ))}
          </div>

          {form.type === 'select' && (
            <div style={{ marginTop: 18 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h3 className="admin-section-title" style={{ margin: 0 }}>
                  {t('Options')}
                </h3>
                <button
                  type="button"
                  className="admin-button ghost"
                  onClick={() => setForm({ ...form, options: [...form.options, { admin_name: '', swatch_value: '', sort_order: form.options.length }] })}
                >
                  <IconPlus size={14} /> {t('Add option')}
                </button>
              </div>
              <div style={{ display: 'grid', gap: 8, marginTop: 10 }}>
                {form.options.map((option: OptionRow, index: number) => (
                  <div key={index} style={{ display: 'grid', gridTemplateColumns: '1fr 130px 80px 36px', gap: 8 }}>
                    <input
                      className="admin-input"
                      value={option.admin_name}
                      onChange={(event) => {
                        const options = [...form.options];
                        options[index] = { ...option, admin_name: event.target.value };
                        setForm({ ...form, options });
                      }}
                      placeholder={t('Option name')}
                    />
                    <input
                      type="color"
                      className="admin-input"
                      value={option.swatch_value || '#111827'}
                      onChange={(event) => {
                        const options = [...form.options];
                        options[index] = { ...option, swatch_value: event.target.value };
                        setForm({ ...form, options });
                      }}
                    />
                    <input
                      type="number"
                      className="admin-input"
                      value={option.sort_order ?? 0}
                      onChange={(event) => {
                        const options = [...form.options];
                        options[index] = { ...option, sort_order: Number(event.target.value) };
                        setForm({ ...form, options });
                      }}
                    />
                    <button
                      type="button"
                      className="admin-icon-button danger"
                      onClick={() => setForm({ ...form, options: form.options.filter((_: any, position: number) => position !== index) })}
                    >
                      <IconTrash size={15} />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {error && <p className="form-error">{error}</p>}

          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 20 }}>
            <button type="button" className="admin-button ghost" onClick={() => setOpen(false)}>
              {t('Cancel')}
            </button>
            <button type="submit" className="admin-button primary">
              {editing ? t('Save changes') : t('Create attribute')}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}

export function FamiliesPage() {
  const [families, setFamilies] = useState<any[]>([]);
  const [search, setSearch] = useState('');
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<number | null>(null);
  const [form, setForm] = useState({ code: '', name: '' });
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  const load = () => {
    api
      .adminGet('admin/catalog/families')
      .then((data) => setFamilies(Array.isArray(data) ? data : []))
      .catch(() => setFamilies([]));
  };

  useEffect(load, []);

  const save = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    try {
      if (editing) await api.adminPut(`admin/catalog/families/edit/${editing}`, form);
      else await api.adminPost('admin/catalog/families/store', form);
      setOpen(false);
      setMessage(editing ? t('Attribute family updated.') : t('Attribute family created.'));
      load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to save the attribute family.'));
    }
  };

  const remove = async (family: any) => {
    if (!window.confirm(t('Delete family “{name}”?', { name: family.name }))) return;
    try {
      await api.adminDelete(`admin/catalog/families/${family.id}`);
      setMessage(t('Attribute family deleted.'));
      load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to delete the attribute family.'));
    }
  };

  const visibleFamilies = search.trim()
    ? families.filter((family) =>
        `${family.code ?? ''} ${family.name ?? ''}`.toLowerCase().includes(search.trim().toLowerCase()),
      )
    : families;

  return (
    <div>
      <AdminPageHeader
        title={t('Attribute Families')}
        subtitle={t('Group attributes into families that can be assigned to products.')}
        actions={
          <button
            type="button"
            className="admin-button primary"
            onClick={() => {
              setEditing(null);
              setForm({ code: '', name: '' });
              setOpen(true);
            }}
          >
            <IconPlus size={15} /> {t('Create family')}
          </button>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && !open && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card admin-card-pad">
        <div className="admin-datagrid-toolbar">
          <div className="admin-datagrid-search">
            <span style={{ position: 'absolute', left: 11, display: 'flex', color: '#9ca3af' }}>
              <IconSearch size={15} />
            </span>
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder={t('Search...')}
              aria-label={t('Search attribute families')}
            />
            {search && (
              <button type="button" className="admin-input-clear" onClick={() => setSearch('')} aria-label={t('Clear search')}>
                <IconX size={13} />
              </button>
            )}
          </div>
          <div style={{ flex: 1 }} />
          <span className="admin-muted admin-small">{t('{count} record(s)', { count: visibleFamilies.length })}</span>
          <button type="button" className="admin-toolbar-button" onClick={load}>
            <IconRefresh size={15} /> {t('Refresh')}
          </button>
        </div>
        <table className="admin-table">
          <thead>
            <tr>
              <th>{t('ID')}</th>
              <th>{t('Code')}</th>
              <th>{t('Name')}</th>
              <th>{t('User defined')}</th>
              <th className="actions">{t('Action')}</th>
            </tr>
          </thead>
          <tbody>
            {visibleFamilies.length === 0 ? (
              <tr>
                <td colSpan={5}>
                  <div className="admin-empty">
                    {families.length ? t('No families match your search.') : t('No attribute families found.')}
                  </div>
                </td>
              </tr>
            ) : (
              visibleFamilies.map((family) => (
                <tr key={family.id}>
                  <td>{family.id}</td>
                  <td>
                    <span className="admin-code">{family.code}</span>
                  </td>
                  <td style={{ color: '#111827', fontWeight: 500 }}>{family.name}</td>
                  <td>{family.is_user_defined ? t('Yes') : t('No')}</td>
                  <td className="actions">
                    <button
                      type="button"
                      className="admin-link"
                      onClick={() => {
                        setEditing(family.id);
                        setForm({ code: family.code, name: family.name });
                        setOpen(true);
                      }}
                    >
                      {t('Edit')}
                    </button>
                    <button type="button" className="admin-link danger" onClick={() => remove(family)}>
                      {t('Delete')}
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      <Modal opened={open} onClose={() => setOpen(false)} title={editing ? t('Edit attribute family') : t('New attribute family')} centered>
        <form onSubmit={save}>
          <label className="admin-field">
            <span className="admin-field-label">{t('Code')} *</span>
            <input className="admin-input" value={form.code} onChange={(event) => setForm({ ...form, code: event.target.value })} required />
          </label>
          <label className="admin-field" style={{ marginTop: 14 }}>
            <span className="admin-field-label">{t('Name')} *</span>
            <input className="admin-input" value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} required />
          </label>
          {error && <p className="form-error">{error}</p>}
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 18 }}>
            <button type="button" className="admin-button ghost" onClick={() => setOpen(false)}>
              {t('Cancel')}
            </button>
            <button type="submit" className="admin-button primary">
              {t('Save')}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
