import { useCallback, useEffect, useMemo, useState } from 'react';
import { Modal } from '@mantine/core';
import { IconPlus, IconRefresh, IconSearch, IconSend, IconTrash, IconX } from '@tabler/icons-react';
import { api } from '../../api/client';
import { formatDate } from '../../lib/format';
import ResourcePage from '../../components/admin/ResourcePage';
import { AdminPageHeader, Badge } from '../../components/admin/ui';
import { t } from '../../i18n';

export function TemplatesPage() {
  return (
    <ResourcePage
      title={t('Email Templates')}
      subtitle={t('Reusable email bodies used by campaigns and transactional messages.')}
      endpoint="admin/marketing/communications/templates"
      createLabel={t('Create template')}
      fields={[
        { name: 'id', label: t('ID'), formHidden: true },
        { name: 'name', label: t('Name'), required: true, placeholder: 'Welcome email' },
        { name: 'status', label: t('Status'), type: 'checkbox', defaultValue: true },
        {
          name: 'content',
          label: t('Content'),
          type: 'textarea',
          colSpan: 2,
          placeholder: t('Hello {{first_name}}, …'),
          hint: t('Available placeholders: {{first_name}}, {{last_name}}, {{name}}, {{email}}.'),
        },
        { name: 'updated_at', label: t('Updated'), formHidden: true, render: (value) => formatDate(value, true) },
      ]}
      columns={['id', 'name', 'status', 'updated_at']}
      createEndpoint="admin/marketing/communications/templates/store"
      updateEndpoint="admin/marketing/communications/templates/edit/{id}"
      deleteEndpoint="admin/marketing/communications/templates/{id}"
      emptyLabel={t('No email templates found.')}
    />
  );
}

export function EventsPage() {
  return (
    <ResourcePage
      title={t('Events')}
      subtitle={t('Dates and occasions that campaigns can be attached to.')}
      endpoint="admin/marketing/communications/events"
      createLabel={t('Create event')}
      fields={[
        { name: 'id', label: t('ID'), formHidden: true },
        { name: 'name', label: t('Name'), required: true, placeholder: t('Black Friday') },
        { name: 'date', label: t('Date'), type: 'date' },
        { name: 'description', label: t('Description'), type: 'textarea', colSpan: 2 },
        { name: 'created_at', label: t('Created'), formHidden: true, render: (value) => formatDate(value, true) },
      ]}
      columns={['id', 'name', 'date', 'created_at']}
      createEndpoint="admin/marketing/communications/events/store"
      updateEndpoint="admin/marketing/communications/events/edit/{id}"
      deleteEndpoint="admin/marketing/communications/events/{id}"
      emptyLabel={t('No events found.')}
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
      setError(requestError?.message ?? t('Unable to load campaigns.'));
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
      setMessage(editing ? t('The campaign has been updated.') : t('The campaign has been created.'));
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to save the campaign.'));
    } finally {
      setSaving(false);
    }
  };

  const send = async (row: any) => {
    if (!window.confirm(t('Send “{name}” to subscribed customers?', { name: row.name }))) return;
    try {
      const result = await api.adminPost(`admin/marketing/communications/campaigns/send/${row.id}`, {});
      setMessage(t('Campaign queued for {count} recipients.', { count: result?.recipients ?? 0 }));
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to send the campaign.'));
    }
  };

  const remove = async (row: any) => {
    if (!window.confirm(t('Delete campaign “{name}”?', { name: row.name }))) return;
    try {
      await api.adminDelete(`admin/marketing/communications/campaigns/${row.id}`);
      setMessage(t('The campaign has been deleted.'));
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to delete the campaign.'));
    }
  };

  return (
    <div>
      <AdminPageHeader
        title={t('Campaigns')}
        subtitle={t('Send newsletter campaigns to customers by group and event.')}
        actions={
          <button type="button" className="admin-button primary" onClick={startCreate}>
            <IconPlus size={15} /> {t('Create campaign')}
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
              placeholder={t('Search...')}
              aria-label={t('Search campaigns')}
            />
            {search && (
              <button type="button" className="admin-input-clear" onClick={() => setSearch('')} aria-label={t('Clear search')}>
                <IconX size={13} />
              </button>
            )}
          </div>
          <div style={{ flex: 1 }} />
          <span className="admin-muted admin-small">{t('{count} record(s)', { count: visibleRows.length })}</span>
          <button type="button" className="admin-toolbar-button" onClick={() => load()}>
            <IconRefresh size={15} /> {t('Refresh')}
          </button>
        </div>
        <table className="admin-table">
          <thead>
            <tr>
              <th>{t('ID')}</th>
              <th>{t('Name')}</th>
              <th>{t('Subject')}</th>
              <th>{t('Template')}</th>
              <th>{t('Event')}</th>
              <th>{t('Customer group')}</th>
              <th>{t('Status')}</th>
              <th>{t('Created')}</th>
              <th className="actions">{t('Actions')}</th>
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
                  <div className="admin-empty">{rows.length ? t('No campaigns match your search.') : t('No campaigns found.')}</div>
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
                  <td>{row.group_name || t('All groups')}</td>
                  <td>
                    <Badge status={row.status ? 'active' : 'inactive'} label={row.status ? t('Sent') : t('Draft')} />
                  </td>
                  <td>{formatDate(row.created_at)}</td>
                  <td className="actions">
                    <div className="admin-row-actions">
                      <button type="button" className="admin-link" onClick={() => startEdit(row)}>
                        {t('Edit')}
                      </button>
                      <button type="button" className="admin-link" onClick={() => send(row)}>
                        <IconSend size={14} /> {t('Send')}
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

      <Modal opened={open} onClose={() => setOpen(false)} title={editing ? t('Edit campaign') : t('Create campaign')} size="lg" centered>
        <div className="admin-modal-form">
          <div className="admin-form-grid">
            <label className="admin-field">
              <span className="admin-field-label">{t('Name')}</span>
              <input className="admin-input" value={values.name} onChange={(event) => setValues((current: any) => ({ ...current, name: event.target.value }))} />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Subject')}</span>
              <input className="admin-input" value={values.subject} onChange={(event) => setValues((current: any) => ({ ...current, subject: event.target.value }))} />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Template')}</span>
              <select
                className="admin-input"
                value={values.marketing_template_id}
                onChange={(event) => setValues((current: any) => ({ ...current, marketing_template_id: event.target.value }))}
              >
                <option value="">{t('Select a template…')}</option>
                {templates.map((template) => (
                  <option key={template.id} value={template.id}>
                    {template.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Customer group')}</span>
              <select
                className="admin-input"
                value={values.customer_group_id}
                onChange={(event) => setValues((current: any) => ({ ...current, customer_group_id: event.target.value }))}
              >
                <option value="">{t('All groups')}</option>
                {groups.map((group) => (
                  <option key={group.id} value={group.id}>
                    {group.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Event')}</span>
              <select
                className="admin-input"
                value={values.marketing_event_id}
                onChange={(event) => setValues((current: any) => ({ ...current, marketing_event_id: event.target.value }))}
              >
                <option value="">{t('No event')}</option>
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
              {t('Cancel')}
            </button>
            <button type="button" className="admin-button primary" onClick={save} disabled={saving}>
              {saving ? t('Saving…') : t('Save campaign')}
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
      setError(requestError?.message ?? t('Unable to load subscribers.'));
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
      setMessage(t('{email} has been unsubscribed.', { email: row.email }));
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to update the subscription.'));
    }
  };

  return (
    <div>
      <AdminPageHeader title={t('Newsletter Subscribers')} subtitle={t('Customers who opted in to marketing emails.')} />
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
              placeholder={t('Search...')}
              aria-label={t('Search subscribers')}
            />
            {search && (
              <button type="button" className="admin-input-clear" onClick={() => setSearch('')} aria-label={t('Clear search')}>
                <IconX size={13} />
              </button>
            )}
          </div>
          <div style={{ flex: 1 }} />
          <span className="admin-muted admin-small">{t('{count} record(s)', { count: visibleRows.length })}</span>
          <button type="button" className="admin-toolbar-button" onClick={() => load()}>
            <IconRefresh size={15} /> {t('Refresh')}
          </button>
        </div>
        <table className="admin-table">
          <thead>
            <tr>
              <th>{t('ID')}</th>
              <th>{t('Name')}</th>
              <th>{t('Email')}</th>
              <th>{t('Subscribed')}</th>
              <th>{t('Since')}</th>
              <th className="actions">{t('Action')}</th>
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
                  <div className="admin-empty">{rows.length ? t('No subscribers match your search.') : t('No newsletter subscribers yet.')}</div>
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
                    <Badge status={row.subscribed_to_news_letter ? 'active' : 'inactive'} label={row.subscribed_to_news_letter ? t('Yes') : t('No')} />
                  </td>
                  <td>{formatDate(row.created_at)}</td>
                  <td className="actions">
                    <button type="button" className="admin-link danger" onClick={() => unsubscribe(row)}>
                      {t('Unsubscribe')}
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
