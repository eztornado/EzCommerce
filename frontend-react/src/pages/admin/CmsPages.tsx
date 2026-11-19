import { useEffect, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import { IconCheck, IconExternalLink } from '@tabler/icons-react';
import { api } from '../../api/client';
import DataGrid from '../../components/admin/DataGrid';
import { AdminPageHeader, AdminTabs } from '../../components/admin/ui';

export function CmsPagesPage() {
  return (
    <div>
      <AdminPageHeader
        title="CMS Pages"
        subtitle="Publish static content pages such as About, Shipping or Returns."
        actions={
          <Link to="/admin/cms/pages/create" className="admin-button primary">
            Create page
          </Link>
        }
      />
      <div className="admin-card admin-card-pad">
        <DataGrid
          path="admin/cms/pages"
          hideColumn={['id']}
          emptyLabel="No CMS pages found."
          actions={(row) => (
            <Link className="admin-link" to={`/admin/cms/pages/edit/${row.__primary ?? row.id}`}>
              Edit
            </Link>
          )}
        />
      </div>
    </div>
  );
}

export function CmsEditorPage() {
  const { id } = useParams();
  const isCreate = !id || id === 'create';
  const navigate = useNavigate();
  const [tab, setTab] = useState('general');
  const [values, setValues] = useState<any>({
    locale: 'en',
    page_title: '',
    url_key: '',
    html_content: '',
    meta_title: '',
    meta_description: '',
  });
  const [loading, setLoading] = useState(!isCreate);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    if (isCreate) return;
    (async () => {
      setLoading(true);
      try {
        const data = await api.adminGet(`admin/cms/pages/view/${id}`);
        setValues({ ...values, ...(data.translation ?? {}) });
      } catch (requestError: any) {
        setError(requestError?.message ?? 'Unable to load the page.');
      } finally {
        setLoading(false);
      }
    })();
  }, [id, isCreate]);

  const set = (key: string, value: any) => setValues((current: any) => ({ ...current, [key]: value }));

  const save = async () => {
    setError('');
    setMessage('');
    if (!String(values.page_title ?? '').trim() || !String(values.url_key ?? '').trim()) {
      setError('The page title and URL key are required.');
      setTab('general');
      return;
    }
    setSaving(true);
    try {
      if (isCreate) {
        const created = await api.adminPost('admin/cms/pages/store', values);
        if (created?.id) navigate(`/admin/cms/pages/edit/${created.id}`);
      } else {
        await api.adminPut(`admin/cms/pages/edit/${id}`, values);
        setMessage('The page has been saved successfully.');
      }
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to save the page.');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div>
        <AdminPageHeader title="Loading page…" backTo="/admin/cms/pages" />
        <div className="admin-card admin-card-pad">
          {Array.from({ length: 5 }).map((_, index) => (
            <div className="admin-skeleton-row" key={index} />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div>
      <AdminPageHeader
        title={isCreate ? 'Create CMS page' : `Edit page · ${values.page_title || id}`}
        subtitle="Rich HTML content is rendered by the storefront CMS page."
        backTo="/admin/cms/pages"
        actions={
          <>
            {!isCreate && values.url_key && (
              <a className="admin-button" href={`/page/${values.url_key}`} target="_blank" rel="noreferrer">
                <IconExternalLink size={15} /> Preview page
              </a>
            )}
            <button type="button" className="admin-button primary" onClick={save} disabled={saving}>
              <IconCheck size={15} /> {saving ? 'Saving…' : 'Save page'}
            </button>
          </>
        }
      />
      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card">
        <AdminTabs
          tabs={[
            { key: 'general', label: 'General' },
            { key: 'seo', label: 'SEO' },
          ]}
          active={tab}
          onChange={setTab}
        />
        <div className="admin-card-pad">
          {tab === 'general' && (
            <div className="admin-form-grid">
              <label className="admin-field">
                <span className="admin-field-label">
                  Page title <em>*</em>
                </span>
                <input className="admin-input" value={values.page_title ?? ''} onChange={(event) => set('page_title', event.target.value)} />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">
                  URL key <em>*</em>
                </span>
                <input className="admin-input" value={values.url_key ?? ''} onChange={(event) => set('url_key', event.target.value)} placeholder="about-us" />
              </label>
              <label className="admin-field">
                <span className="admin-field-label">Locale</span>
                <select className="admin-input" value={values.locale ?? 'en'} onChange={(event) => set('locale', event.target.value)}>
                  <option value="en">English</option>
                  <option value="es">Spanish</option>
                  <option value="fr">French</option>
                  <option value="de">German</option>
                  <option value="ar">Arabic</option>
                </select>
              </label>
              <label className="admin-field" style={{ gridColumn: '1 / -1' }}>
                <span className="admin-field-label">HTML content</span>
                <textarea
                  className="admin-input admin-code-input"
                  rows={16}
                  value={values.html_content ?? ''}
                  onChange={(event) => set('html_content', event.target.value)}
                  placeholder="<h1>About us</h1><p>…</p>"
                />
                <span className="admin-field-hint">Raw HTML is stored and rendered on the storefront.</span>
              </label>
            </div>
          )}

          {tab === 'seo' && (
            <div className="admin-form-grid">
              <label className="admin-field" style={{ gridColumn: '1 / -1' }}>
                <span className="admin-field-label">Meta title</span>
                <input className="admin-input" value={values.meta_title ?? ''} onChange={(event) => set('meta_title', event.target.value)} />
              </label>
              <label className="admin-field" style={{ gridColumn: '1 / -1' }}>
                <span className="admin-field-label">Meta description</span>
                <textarea className="admin-input" rows={3} value={values.meta_description ?? ''} onChange={(event) => set('meta_description', event.target.value)} />
              </label>
            </div>
          )}
        </div>
      </div>

      {tab === 'general' && values.html_content && (
        <div className="admin-card" style={{ marginTop: 16 }}>
          <div className="admin-card-head">
            <h2 className="admin-card-title">Preview</h2>
          </div>
          <div className="admin-card-pad cms-preview" dangerouslySetInnerHTML={{ __html: values.html_content }} />
        </div>
      )}
    </div>
  );
}
