import { useCallback, useEffect, useMemo, useState } from 'react';
import { Modal } from '@mantine/core';
import { IconPlus, IconRefresh, IconSearch, IconSend, IconTrash, IconX } from '@tabler/icons-react';
import { api } from '../../api/client';
import { formatDate } from '../../lib/format';
import ResourcePage from '../../components/admin/ResourcePage';
import { AdminPageHeader, Badge } from '../../components/admin/ui';

export function TemplatesPage() {
  return (
    <ResourcePage
      title="Email Templates"
      subtitle="Reusable email bodies used by campaigns and transactional messages."
      endpoint="admin/marketing/communications/templates"
      createLabel="Create template"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'name', label: 'Name', required: true, placeholder: 'Welcome email' },
        { name: 'status', label: 'Status', type: 'checkbox', defaultValue: true },
        {
          name: 'content',
          label: 'Content',
          type: 'textarea',
          colSpan: 2,
          placeholder: 'Hello {{first_name}}, …',
          hint: 'Available placeholders: {{first_name}}, {{last_name}}, {{name}}, {{email}}.',
        },
        { name: 'updated_at', label: 'Updated', formHidden: true, render: (value) => formatDate(value, true) },
      ]}
      columns={['id', 'name', 'status', 'updated_at']}
      createEndpoint="admin/marketing/communications/templates/store"
      updateEndpoint="admin/marketing/communications/templates/edit/{id}"
      deleteEndpoint="admin/marketing/communications/templates/{id}"
      emptyLabel="No email templates found."
    />
  );
}

export function EventsPage() {
  return (
    <ResourcePage
      title="Events"
      subtitle="Dates and occasions that campaigns can be attached to."
      endpoint="admin/marketing/communications/events"
      createLabel="Create event"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'name', label: 'Name', required: true, placeholder: 'Black Friday' },
        { name: 'date', label: 'Date', type: 'date' },
        { name: 'description', label: 'Description', type: 'textarea', colSpan: 2 },
        { name: 'created_at', label: 'Created', formHidden: true, render: (value) => formatDate(value, true) },
      ]}
      columns={['id', 'name', 'date', 'created_at']}
      createEndpoint="admin/marketing/communications/events/store"
      updateEndpoint="admin/marketing/communications/events/edit/{id}"
      deleteEndpoint="admin/marketing/communications/events/{id}"
      emptyLabel="No events found."
    />
  );
}

export function CampaignsPage() {
  const [rows, setRows] = useState<any[]>([]);
  const [templates, setTemplates] = useState<any[]>([]);
  const [events, setEvents] = useState<any[]>([]);
  const [groups, setGroups] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<any | null>(null);
  const [values, setValues] = useState<any>({ name: '', subject: '', marketing_template_id: '', customer_group_id: '', marketing_event_id: '' });
  const [saving, setSaving] = useState(false);
  const [search, setSearch] = useState('');

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [campaignRows, templateRows, eventRows, groupRows] = await Promise.all([
        api.adminGet('admin/marketing/communications/campaigns'),
        api.adminGet('admin/marketing/communications/templates'),
        api.adminGet('admin/marketing/communications/events'),
        api.adminGet('admin/customers/groups'),
      ]);
      setRows(Array.isArray(campaignRows) ? campaignRows : []);
      setTemplates(Array.isArray(templateRows) ? templateRows : []);
      setEvents(Array.isArray(eventRows) ? eventRows : []);
      setGroups(Array.isArray(groupRows) ? groupRows : []);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to load campaigns.');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const visibleRows = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return rows;
    return rows.filter((row) =>
      [row.name, row.subject, row.template_name, row.event_name, row.group_name]
        .some((value) => String(value ?? '').toLowerCase().includes(term)),
    );
  }, [rows, search]);

  const startCreate = () => {
    setEditing(null);
    setValues({ name: '', subject: '', marketing_template_id: templates[0]?.id ?? '', customer_group_id: '', marketing_event_id: '' });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setValues({
      name: row.name ?? '',
      subject: row.subject ?? '',
      marketing_template_id: row.marketing_template_id ?? '',
      customer_group_id: row.customer_group_id ?? '',
      marketing_event_id: row.marketing_event_id ?? '',
    });
    setOpen(true);
  };

  const save = async () => {
    setSaving(true);
    setError('');
    try {
      const payload = {
        name: values.name,
        subject: values.subject,
        marketing_template_id: Number(values.marketing_template_id) || null,
        customer_group_id: Number(values.customer_group_id) || null,
        marketing_event_id: Number(values.marketing_event_id) || null,
      };
      if (editing) await api.adminPut(`admin/marketing/communications/campaigns/edit/${editing.id}`, payload);
      else await api.adminPost('admin/marketing/communications/campaigns/store', payload);
      setOpen(false);
      setMessage(editing ? 'The campaign has been updated.' : 'The campaign has been created.');
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to save the campaign.');
    } finally {
      setSaving(false);
    }
  };

  const send = async (row: any) => {
    if (!window.confirm(`Send “${row.name}” to subscribed customers?`)) return;
    try {
      const result = await api.adminPost(`admin/marketing/communications/campaigns/send/${row.id}`, {});
      setMessage(`Campaign queued for ${result?.recipients ?? 0} recipients.`);
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to send the campaign.');
    }
  };

  const remove = async (row: any) => {
    if (!window.confirm(`Delete campaign “${row.name}”?`)) return;
    try {
      await api.adminDelete(`admin/marketing/communications/campaigns/${row.id}`);
      setMessage('The campaign has been deleted.');
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to delete the campaign.');
    }
  };

  return (
    <div>
      <AdminPageHeader
        title="Campaigns"
        subtitle="Send newsletter campaigns to customers by group and event."
        actions={
          <button type="button" className="admin-button primary" onClick={startCreate}>
            <IconPlus size={15} /> Create campaign
          </button>
        }
      />
      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card admin-card-pad">
        <div className="admin-datagrid-toolbar">
          <div className="admin-datagrid-search">
            <span style={{ position: 'absolute', left: 11, display: 'flex', color: '#9ca3af' }}>
              <IconSearch size={15} />
            </span>
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search..."
              aria-label="Search campaigns"
            />
            {search && (
              <button type="button" className="admin-input-clear" onClick={() => setSearch('')} aria-label="Clear search">
                <IconX size={13} />
              </button>
            )}
          </div>
          <div style={{ flex: 1 }} />
          <span className="admin-muted admin-small">{visibleRows.length} record(s)</span>
          <button type="button" className="admin-toolbar-button" onClick={() => load()}>
            <IconRefresh size={15} /> Refresh
          </button>
        </div>
        <table className="admin-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Subject</th>
              <th>Template</th>
              <th>Event</th>
              <th>Customer group</th>
              <th>Status</th>
              <th>Created</th>
              <th className="actions">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              Array.from({ length: 4 }).map((_, index) => (
                <tr key={index}>
                  <td colSpan={9}>
                    <div className="admin-skeleton-row" />
                  </td>
                </tr>
              ))
            ) : visibleRows.length === 0 ? (
              <tr>
                <td colSpan={9}>
                  <div className="admin-empty">{rows.length ? 'No campaigns match your search.' : 'No campaigns found.'}</div>
                </td>
              </tr>
            ) : (
              visibleRows.map((row) => (
                <tr key={row.id}>
                  <td>{row.id}</td>
                  <td>{row.name}</td>
                  <td>{row.subject}</td>
                  <td>{row.template_name || '—'}</td>
                  <td>{row.event_name || '—'}</td>
                  <td>{row.group_name || 'All groups'}</td>
                  <td>
                    <Badge status={row.status ? 'active' : 'inactive'} label={row.status ? 'Sent' : 'Draft'} />
                  </td>
                  <td>{formatDate(row.created_at)}</td>
                  <td className="actions">
                    <div className="admin-row-actions">
                      <button type="button" className="admin-link" onClick={() => startEdit(row)}>
                        Edit
                      </button>
                      <button type="button" className="admin-link" onClick={() => send(row)}>
                        <IconSend size={14} /> Send
                      </button>
                      <button type="button" className="admin-link danger" onClick={() => remove(row)}>
                        <IconTrash size={14} />
                      </button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      <Modal opened={open} onClose={() => setOpen(false)} title={editing ? 'Edit campaign' : 'Create campaign'} size="lg" centered>
        <div className="admin-modal-form">
          <div className="admin-form-grid">
            <label className="admin-field">
              <span className="admin-field-label">Name</span>
              <input className="admin-input" value={values.name} onChange={(event) => setValues((current: any) => ({ ...current, name: event.target.value }))} />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Subject</span>
              <input className="admin-input" value={values.subject} onChange={(event) => setValues((current: any) => ({ ...current, subject: event.target.value }))} />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Template</span>
              <select
                className="admin-input"
                value={values.marketing_template_id}
                onChange={(event) => setValues((current: any) => ({ ...current, marketing_template_id: event.target.value }))}
              >
                <option value="">Select a template…</option>
                {templates.map((template) => (
                  <option key={template.id} value={template.id}>
                    {template.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Customer group</span>
              <select
                className="admin-input"
                value={values.customer_group_id}
                onChange={(event) => setValues((current: any) => ({ ...current, customer_group_id: event.target.value }))}
              >
                <option value="">All groups</option>
                {groups.map((group) => (
                  <option key={group.id} value={group.id}>
                    {group.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Event</span>
              <select
                className="admin-input"
                value={values.marketing_event_id}
                onChange={(event) => setValues((current: any) => ({ ...current, marketing_event_id: event.target.value }))}
              >
                <option value="">No event</option>
                {events.map((event) => (
                  <option key={event.id} value={event.id}>
                    {event.name}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="admin-modal-actions">
            <button type="button" className="admin-button" onClick={() => setOpen(false)}>
              Cancel
            </button>
            <button type="button" className="admin-button primary" onClick={save} disabled={saving}>
              {saving ? 'Saving…' : 'Save campaign'}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

export function SubscribersPage() {
  const [rows, setRows] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [search, setSearch] = useState('');

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.adminGet('admin/seo/subscribers');
      setRows(Array.isArray(data) ? data : []);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to load subscribers.');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const visibleRows = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return rows;
    return rows.filter((row) =>
      [row.email, row.first_name, row.last_name].some((value) => String(value ?? '').toLowerCase().includes(term)),
    );
  }, [rows, search]);

  const unsubscribe = async (row: any) => {
    try {
      await api.adminPut(`admin/seo/subscribers/${row.id}`, { subscribed: 0 });
      setMessage(`${row.email} has been unsubscribed.`);
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to update the subscription.');
    }
  };

  return (
    <div>
      <AdminPageHeader title="Newsletter Subscribers" subtitle="Customers who opted in to marketing emails." />
      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}
      <div className="admin-card admin-card-pad">
        <div className="admin-datagrid-toolbar">
          <div className="admin-datagrid-search">
            <span style={{ position: 'absolute', left: 11, display: 'flex', color: '#9ca3af' }}>
              <IconSearch size={15} />
            </span>
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search..."
              aria-label="Search subscribers"
            />
            {search && (
              <button type="button" className="admin-input-clear" onClick={() => setSearch('')} aria-label="Clear search">
                <IconX size={13} />
              </button>
            )}
          </div>
          <div style={{ flex: 1 }} />
          <span className="admin-muted admin-small">{visibleRows.length} record(s)</span>
          <button type="button" className="admin-toolbar-button" onClick={() => load()}>
            <IconRefresh size={15} /> Refresh
          </button>
        </div>
        <table className="admin-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Email</th>
              <th>Subscribed</th>
              <th>Since</th>
              <th className="actions">Action</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              Array.from({ length: 4 }).map((_, index) => (
                <tr key={index}>
                  <td colSpan={6}>
                    <div className="admin-skeleton-row" />
                  </td>
                </tr>
              ))
            ) : visibleRows.length === 0 ? (
              <tr>
                <td colSpan={6}>
                  <div className="admin-empty">{rows.length ? 'No subscribers match your search.' : 'No newsletter subscribers yet.'}</div>
                </td>
              </tr>
            ) : (
              visibleRows.map((row) => (
                <tr key={row.id}>
                  <td>{row.id}</td>
                  <td>
                    {row.first_name} {row.last_name}
                  </td>
                  <td>{row.email}</td>
                  <td>
                    <Badge status={row.subscribed_to_news_letter ? 'active' : 'inactive'} label={row.subscribed_to_news_letter ? 'Yes' : 'No'} />
                  </td>
                  <td>{formatDate(row.created_at)}</td>
                  <td className="actions">
                    <button type="button" className="admin-link danger" onClick={() => unsubscribe(row)}>
                      Unsubscribe
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
