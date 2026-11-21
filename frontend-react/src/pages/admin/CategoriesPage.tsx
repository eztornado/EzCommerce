import { useEffect, useState } from 'react';
import { Modal } from '@mantine/core';
import { IconChevronRight, IconPencil, IconPlus, IconTrash } from '@tabler/icons-react';
import { api } from '../../api/client';
import { AdminPageHeader } from '../../components/admin/ui';
import { t } from '../../i18n';

interface CategoryNode {
  id: number;
  name: string;
  slug?: string;
  status?: number;
  position?: number;
  children?: CategoryNode[];
}

const emptyForm = {
  parent_id: 1,
  name: '',
  position: 0,
  status: 1,
  description: '',
  display_mode: 'products_and_description',
};

export default function CategoriesPage() {
  const [tree, setTree] = useState<CategoryNode[]>([]);
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<number | null>(null);
  const [form, setForm] = useState<any>(emptyForm);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  const load = () => {
    api
      .adminGet('admin/catalog/categories/tree')
      .then((data) => setTree(Array.isArray(data) ? data : []))
      .catch(() => setTree([]));
  };

  useEffect(load, []);

  const startCreate = (parentId = 1) => {
    setEditing(null);
    setError('');
    setForm({ ...emptyForm, parent_id: parentId });
    setOpen(true);
  };

  const startEdit = async (category: CategoryNode) => {
    setEditing(category.id);
    setError('');
    try {
      const detail = await api.get(`categories/${category.id}`);
      setForm({
        parent_id: detail.parent_id ?? 1,
        name: detail.name ?? category.name,
        position: detail.position ?? category.position ?? 0,
        status: detail.status ?? 1,
        description: detail.description ?? '',
        display_mode: detail.display_mode ?? 'products_and_description',
      });
    } catch {
      setForm({ ...emptyForm, name: category.name, position: category.position ?? 0, status: category.status ?? 1 });
    }
    setOpen(true);
  };

  const save = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    try {
      if (editing) await api.adminPut(`admin/catalog/categories/edit/${editing}?locale=en`, form);
      else await api.adminPost('admin/catalog/categories/store', { ...form, locale: 'en' });
      setOpen(false);
      setMessage(editing ? t('Category updated.') : t('Category created.'));
      load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to save the category.'));
    }
  };

  const remove = async (category: CategoryNode) => {
    if (!window.confirm(t('Delete “{name}”? Products in this category will lose the assignment.', { name: category.name }))) return;
    try {
      await api.adminDelete(`admin/catalog/categories/${category.id}`);
      setMessage(t('Category deleted.'));
      load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to delete the category.'));
    }
  };

  const renderNode = (node: CategoryNode, depth = 0) => (
    <div key={node.id}>
      <div className="admin-tree-row" style={{ paddingLeft: 12 + depth * 24 }}>
        <IconChevronRight size={14} style={{ color: '#9ca3af' }} />
        <span style={{ flex: 1, fontWeight: depth === 0 ? 600 : 500, color: '#111827' }}>{node.name}</span>
        <span className={`status-badge ${node.status === 0 ? 'red' : 'green'}`}>{node.status === 0 ? t('Inactive') : t('Active')}</span>
        <span className="admin-muted admin-small">ID {node.id}</span>
        <button type="button" className="admin-icon-button" onClick={() => startCreate(node.id)} title={t('Add child')}>
          <IconPlus size={15} />
        </button>
        <button type="button" className="admin-icon-button" onClick={() => startEdit(node)} title={t('Edit')}>
          <IconPencil size={15} />
        </button>
        <button type="button" className="admin-icon-button danger" onClick={() => remove(node)} title={t('Delete')}>
          <IconTrash size={15} />
        </button>
      </div>
      {(node.children ?? []).map((child) => renderNode(child, depth + 1))}
    </div>
  );

  return (
    <div>
      <AdminPageHeader
        title={t('Categories')}
        subtitle={t('Organise the storefront navigation and product assignments.')}
        actions={
          <button type="button" className="admin-button primary" onClick={() => startCreate(1)}>
            <IconPlus size={15} /> {t('Create category')}
          </button>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && !open && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card">
        <div className="admin-card-head">
          <h2 className="admin-card-title">{t('Category tree')}</h2>
          <span className="admin-card-subtitle">{t('Root category #1 is the store root')}</span>
        </div>
        <div style={{ padding: '0 8px 8px' }}>
          {tree.length === 0 ? (
            <div className="admin-empty">{t('No categories yet.')}</div>
          ) : (
            <div className="admin-tree">{tree.map((node) => renderNode(node))}</div>
          )}
        </div>
      </div>

      <Modal opened={open} onClose={() => setOpen(false)} title={editing ? t('Edit category') : t('New category')} centered size="lg">
        <form onSubmit={save}>
          <div className="admin-form-grid">
            <label className="admin-field span-2">
              <span className="admin-field-label">{t('Name')} *</span>
              <input className="admin-input" value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} required />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Parent category')}</span>
              <select className="admin-input" value={form.parent_id} onChange={(event) => setForm({ ...form, parent_id: Number(event.target.value) })}>
                <option value={1}>{t('Root')}</option>
                {tree.map((node) => (
                  <option key={node.id} value={node.id}>
                    {node.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Position')}</span>
              <input
                type="number"
                className="admin-input"
                value={form.position}
                onChange={(event) => setForm({ ...form, position: Number(event.target.value) })}
              />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Status')}</span>
              <select className="admin-input" value={form.status} onChange={(event) => setForm({ ...form, status: Number(event.target.value) })}>
                <option value={1}>{t('Active')}</option>
                <option value={0}>{t('Inactive')}</option>
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Display mode')}</span>
              <select className="admin-input" value={form.display_mode} onChange={(event) => setForm({ ...form, display_mode: event.target.value })}>
                <option value="products_and_description">{t('Products and description')}</option>
                <option value="products_only">{t('Products only')}</option>
                <option value="description_only">{t('Description only')}</option>
              </select>
            </label>
            <label className="admin-field span-2">
              <span className="admin-field-label">{t('Description')}</span>
              <textarea
                className="admin-input"
                rows={4}
                value={form.description ?? ''}
                onChange={(event) => setForm({ ...form, description: event.target.value })}
              />
            </label>
          </div>

          {error && <p className="form-error">{error}</p>}

          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 18 }}>
            <button type="button" className="admin-button ghost" onClick={() => setOpen(false)}>
              {t('Cancel')}
            </button>
            <button type="submit" className="admin-button primary">
              {editing ? t('Save changes') : t('Create category')}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
