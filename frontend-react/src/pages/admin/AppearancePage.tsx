import { useEffect, useMemo, useState } from 'react';
import {
  IconArrowDown,
  IconArrowUp,
  IconPlus,
  IconTrash,
  IconPencil,
  IconLayoutGrid,
  IconPhoto,
  IconBox,
  IconTextCaption,
  IconTruck,
  IconLink,
  IconEye,
  IconX,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { AdminPageHeader } from '../../components/admin/ui';
import { useShop } from '../../store';

interface Section {
  id: number;
  type: string;
  name: string;
  sort_order: number;
  status: number;
  options: any;
}

const TYPES = [
  { value: 'image_carousel', label: 'Image carousel', icon: <IconPhoto size={16} />, hint: 'Hero slider with images and calls to action.' },
  { value: 'product_carousel', label: 'Product carousel', icon: <IconBox size={16} />, hint: 'Horizontal list of products from a filter.' },
  { value: 'category_carousel', label: 'Category carousel', icon: <IconLayoutGrid size={16} />, hint: 'Grid or carousel of catalog categories.' },
  { value: 'static_content', label: 'Static content', icon: <IconTextCaption size={16} />, hint: 'Promotional banner with title, subtitle and button.' },
  { value: 'services_content', label: 'Services content', icon: <IconTruck size={16} />, hint: 'Icons and text highlighting shipping, returns, support…' },
  { value: 'footer_links', label: 'Footer links', icon: <IconLink size={16} />, hint: 'Columns of links rendered in the storefront footer.' },
];

const OPTION_TEMPLATES: Record<string, any> = {
  image_carousel: {
    slides: [
      {
        image: '/storage/hero/hero-1.jpg',
        title: 'The Autumn Edit',
        subtitle: 'Considered essentials for the season ahead.',
        link: '/products',
        button: { title: 'Shop the collection', link: '/products' },
      },
    ],
  },
  product_carousel: { title: 'New Products', limit: 8, filters: { sort: 'newest' } },
  category_carousel: { title: 'Shop by Category', limit: 8, filters: { parent_id: 1 } },
  static_content: {
    title: 'Get ready for the new season',
    subtitle: 'Up to 30% off selected items',
    button: { title: 'Shop now', link: '/products' },
  },
  services_content: {
    services: [
      { icon: 'truck', title: 'Free shipping', description: 'On all orders over €100' },
      { icon: 'refresh', title: 'Easy returns', description: '30 days to change your mind' },
    ],
  },
  footer_links: {
    columns: [
      { title: 'Quick Links', links: [{ title: 'Home', url: '/' }, { title: 'Shop', url: '/products' }] },
    ],
  },
};

export default function AppearancePage() {
  const locales = useShop((state) => state.context?.locales ?? []);
  const [locale, setLocale] = useState('en');
  const [sections, setSections] = useState<Section[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [editing, setEditing] = useState<Section | null>(null);
  const [creating, setCreating] = useState(false);
  const [form, setForm] = useState({ type: 'image_carousel', name: '', status: 1, options: '{}' });

  const load = async (target = locale) => {
    setLoading(true);
    setError('');
    try {
      const data = await api.adminGet(`admin/appearance/sections/list/${target}`);
      setSections(Array.isArray(data) ? data : data?.data ?? []);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to load the appearance sections.');
      setSections([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load(locale);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [locale]);

  const openCreate = () => {
    setForm({ type: 'image_carousel', name: '', status: 1, options: JSON.stringify(OPTION_TEMPLATES.image_carousel, null, 2) });
    setCreating(true);
    setEditing(null);
    setError('');
  };

  const openEdit = (section: Section) => {
    setForm({
      type: section.type,
      name: section.name ?? '',
      status: section.status ? 1 : 0,
      options: JSON.stringify(section.options ?? {}, null, 2),
    });
    setEditing(section);
    setCreating(false);
    setError('');
  };

  const closeModal = () => {
    setCreating(false);
    setEditing(null);
  };

  const submit = async () => {
    if (!form.name.trim()) {
      setError('The section name is required.');
      return;
    }
    let options: any;
    try {
      options = JSON.parse(form.options || '{}');
    } catch {
      setError('The options must be valid JSON.');
      return;
    }
    setSaving(true);
    setError('');
    try {
      if (editing) {
        await api.adminPut(`admin/appearance/sections/edit/${editing.id}`, {
          type: form.type,
          name: form.name,
          status: form.status,
          locale,
          options,
        });
        setNotice('The section has been updated.');
      } else {
        await api.adminPost('admin/appearance/sections/store', {
          type: form.type,
          name: form.name,
          status: form.status,
          sort: sections.length + 1,
          locale,
          options,
        });
        setNotice('The section has been created.');
      }
      closeModal();
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to save the section.');
    } finally {
      setSaving(false);
    }
  };

  const toggle = async (section: Section) => {
    try {
      await api.adminPost(`admin/appearance/sections/status/${section.id}`, { status: section.status ? 0 : 1 });
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to update the section status.');
    }
  };

  const remove = async (section: Section) => {
    if (!window.confirm(`Delete the “${section.name}” section? This cannot be undone.`)) return;
    try {
      await api.adminDelete(`admin/appearance/sections/${section.id}`);
      setNotice('The section has been deleted.');
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to delete the section.');
    }
  };

  const move = async (index: number, direction: -1 | 1) => {
    const next = [...sections];
    const target = index + direction;
    if (target < 0 || target >= next.length) return;
    [next[index], next[target]] = [next[target], next[index]];
    setSections(next);
    try {
      await api.adminPost('admin/appearance/sections/reorder', { order: next.map((section) => section.id) });
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to reorder the sections.');
      await load();
    }
  };

  const typeMeta = useMemo(() => Object.fromEntries(TYPES.map((type) => [type.value, type])), []);

  const summary = (section: Section) => {
    const options = section.options ?? {};
    if (Array.isArray(options.slides)) return `${options.slides.length} slide${options.slides.length === 1 ? '' : 's'}`;
    if (options.title && section.type === 'static_content') return options.title;
    if (Array.isArray(options.services)) return `${options.services.length} services`;
    if (Array.isArray(options.columns)) return `${options.columns.length} columns`;
    if (options.limit) return `Limit ${options.limit}${options.filters?.sort ? ` · ${options.filters.sort}` : ''}`;
    return 'No options configured';
  };

  return (
    <div>
      <AdminPageHeader
        title="Appearance"
        subtitle="Compose the storefront home page from reusable sections, per locale."
        actions={
          <>
            <select className="admin-input admin-input-inline" value={locale} onChange={(event) => setLocale(event.target.value)}>
              {(locales.length ? locales : [{ code: 'en', name: 'English' }]).map((entry: any) => (
                <option key={entry.code} value={entry.code}>
                  {entry.name} ({entry.code})
                </option>
              ))}
            </select>
            <button type="button" className="admin-button primary" onClick={openCreate}>
              <IconPlus size={15} /> Create section
            </button>
          </>
        }
      />

      {error && <div className="admin-alert danger">{error}</div>}
      {notice && <div className="admin-alert success">{notice}</div>}

      <div className="admin-card">
        <div className="admin-card-head">
          <div>
            <h2 className="admin-card-title">Home page sections</h2>
            <p className="admin-card-subtitle">Drag-free ordering: use the arrows to change the position of each section.</p>
          </div>
          <span className="admin-pill">{sections.length} sections</span>
        </div>

        {loading ? (
          <div className="admin-card-pad">
            <div className="admin-skeleton-row" />
            <div className="admin-skeleton-row" />
            <div className="admin-skeleton-row" />
          </div>
        ) : sections.length === 0 ? (
          <div className="admin-empty">
            <h3>No sections for this locale</h3>
            <p>Create your first section to start composing the storefront home page.</p>
            <button type="button" className="admin-button primary" onClick={openCreate}>
              Create section
            </button>
          </div>
        ) : (
          <div className="appearance-list">
            {sections.map((section, index) => {
              const meta = typeMeta[section.type];
              return (
                <div className="appearance-item" key={section.id}>
                  <div className="appearance-order">
                    <button type="button" className="admin-icon-action" onClick={() => move(index, -1)} disabled={index === 0} title="Move up">
                      <IconArrowUp size={14} />
                    </button>
                    <span>{index + 1}</span>
                    <button type="button" className="admin-icon-action" onClick={() => move(index, 1)} disabled={index === sections.length - 1} title="Move down">
                      <IconArrowDown size={14} />
                    </button>
                  </div>
                  <span className="appearance-type">{meta?.icon ?? <IconLayoutGrid size={16} />}</span>
                  <div className="appearance-copy">
                    <strong>{section.name}</strong>
                    <span>
                      {meta?.label ?? section.type} · {summary(section)}
                    </span>
                  </div>
                  <span className={`status-badge ${section.status ? 'green' : 'gray'}`}>{section.status ? 'Active' : 'Inactive'}</span>
                  <div className="admin-row-actions">
                    <button type="button" className="admin-icon-action" onClick={() => toggle(section)} title={section.status ? 'Deactivate' : 'Activate'}>
                      {section.status ? <IconX size={15} /> : <IconEye size={15} />}
                    </button>
                    <button type="button" className="admin-icon-action" onClick={() => openEdit(section)} title="Edit">
                      <IconPencil size={15} />
                    </button>
                    <button type="button" className="admin-icon-action danger" onClick={() => remove(section)} title="Delete">
                      <IconTrash size={15} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <div className="admin-card admin-card-pad" style={{ marginTop: 16 }}>
        <h2 className="admin-card-title">Available section types</h2>
        <div className="appearance-types">
          {TYPES.map((type) => (
            <div className="appearance-type-card" key={type.value}>
              <span>{type.icon}</span>
              <strong>{type.label}</strong>
              <p>{type.hint}</p>
              <code>{type.value}</code>
            </div>
          ))}
        </div>
      </div>

      {(creating || editing) && (
        <div className="admin-modal-backdrop" onMouseDown={(event) => event.target === event.currentTarget && closeModal()}>
          <div className="admin-modal" style={{ maxWidth: 720 }}>
            <div className="admin-modal-head">
              <div>
                <h2>{editing ? 'Edit section' : 'Create section'}</h2>
                <p>{editing ? `Update “${editing.name}” for ${locale}.` : 'Add a new section to the home page.'}</p>
              </div>
              <button type="button" className="admin-icon-action" onClick={closeModal}>
                <IconX size={16} />
              </button>
            </div>
            <div className="admin-modal-form">
              <label className="admin-field">
                <span className="admin-field-label">Section type <em>*</em></span>
                <select
                  className="admin-input"
                  value={form.type}
                  onChange={(event) => {
                    const type = event.target.value;
                    setForm((current) => ({
                      ...current,
                      type,
                      options: JSON.stringify(OPTION_TEMPLATES[type] ?? {}, null, 2),
                    }));
                  }}
                >
                  {TYPES.map((type) => (
                    <option key={type.value} value={type.value}>
                      {type.label}
                    </option>
                  ))}
                </select>
              </label>
              <label className="admin-field">
                <span className="admin-field-label">Name <em>*</em></span>
                <input
                  className="admin-input"
                  value={form.name}
                  placeholder="Hero slider"
                  onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
                />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">Status</span>
                <select className="admin-input" value={form.status} onChange={(event) => setForm((current) => ({ ...current, status: Number(event.target.value) }))}>
                  <option value={1}>Active</option>
                  <option value={0}>Inactive</option>
                </select>
              </label>
              <label className="admin-field admin-span-2">
                <span className="admin-field-label">Options (JSON)</span>
                <textarea
                  className="admin-input admin-textarea data-transfer-code"
                  rows={16}
                  spellCheck={false}
                  value={form.options}
                  onChange={(event) => setForm((current) => ({ ...current, options: event.target.value }))}
                />
                <span className="admin-field-hint">Stored in <code>theme_section_translations.options</code> for the selected locale.</span>
              </label>
            </div>
            <div className="admin-modal-actions">
              <button type="button" className="admin-button" onClick={closeModal}>Cancel</button>
              <button type="button" className="admin-button primary" onClick={submit} disabled={saving}>
                {saving ? 'Saving…' : editing ? 'Save section' : 'Create section'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
