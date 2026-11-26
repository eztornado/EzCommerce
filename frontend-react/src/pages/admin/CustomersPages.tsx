import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Modal } from '@mantine/core';
import { IconCheck, IconPlus, IconX } from '@tabler/icons-react';
import DataGrid from '../../components/admin/DataGrid';
import ResourcePage from '../../components/admin/ResourcePage';
import { AdminPageHeader } from '../../components/admin/ui';
import { api } from '../../api/client';
import { t } from '../../i18n';

export function CustomersPage() {
  const navigate = useNavigate();
  const [version, setVersion] = useState(0);
  const [open, setOpen] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [values, setValues] = useState({
    first_name: '',
    last_name: '',
    email: '',
    password: '',
    phone: '',
  });

  const startCreate = () => {
    setError('');
    setMessage('');
    setValues({ first_name: '', last_name: '', email: '', password: '', phone: '' });
    setOpen(true);
  };

  const save = async () => {
    setSaving(true);
    setError('');
    try {
      await api.adminPost('admin/customers/store', values);
      setOpen(false);
      setMessage(t('The customer has been created.'));
      setVersion((current) => current + 1);
    } catch (requestError: any) {
      const detail = requestError?.errors
        ? Object.values(requestError.errors).flat().join(' ')
        : requestError?.message;
      setError(detail || t('Unable to create the customer.'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div>
      <AdminPageHeader
        title="Customers"
        subtitle="Manage customer accounts, groups and their order history."
        actions={
          <button type="button" className="admin-button primary" onClick={startCreate}>
            <IconPlus size={15} /> {t('Create customer')}
          </button>
        }
      />
      {message && <div className="admin-alert success">{message}</div>}
      <div className="admin-card admin-card-pad">
        <DataGrid
          key={version}
          path="admin/customers"
          hideColumn={['id']}
          emptyLabel="No customers found."
          onRowClick={(record) => navigate(`/admin/customers/view/${record.__primary ?? record.id}`)}
        />
      </div>

      <Modal opened={open} onClose={() => setOpen(false)} title={t('Create customer')} centered>
        <div className="admin-modal-form">
          {error && <div className="admin-alert danger">{error}</div>}
          <div className="admin-form-grid">
            <label className="admin-field">
              <span className="admin-field-label">{t('First name')}</span>
              <input
                className="admin-input"
                value={values.first_name}
                onChange={(event) => setValues((current) => ({ ...current, first_name: event.target.value }))}
              />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Last name')}</span>
              <input
                className="admin-input"
                value={values.last_name}
                onChange={(event) => setValues((current) => ({ ...current, last_name: event.target.value }))}
              />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Email')}</span>
              <input
                type="email"
                className="admin-input"
                value={values.email}
                onChange={(event) => setValues((current) => ({ ...current, email: event.target.value }))}
              />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Phone')}</span>
              <input
                className="admin-input"
                value={values.phone}
                onChange={(event) => setValues((current) => ({ ...current, phone: event.target.value }))}
              />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">{t('Password')}</span>
              <input
                type="password"
                className="admin-input"
                value={values.password}
                onChange={(event) => setValues((current) => ({ ...current, password: event.target.value }))}
              />
            </label>
          </div>
          <div className="admin-modal-actions">
            <button type="button" className="admin-button" onClick={() => setOpen(false)} disabled={saving}>
              {t('Cancel')}
            </button>
            <button type="button" className="admin-button primary" onClick={save} disabled={saving}>
              {saving ? t('Saving…') : t('Save')}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

export function GroupsPage() {
  return (
    <ResourcePage
      title="Customer Groups"
      subtitle="Group customers to target promotions, pricing and campaigns."
      endpoint="admin/customers/groups"
      createLabel="Create group"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'code', label: 'Code', required: true, placeholder: 'vip' },
        { name: 'name', label: 'Name', required: true, placeholder: 'VIP customers' },
        {
          name: 'is_user_defined',
          label: 'System',
          type: 'checkbox',
          hideInTable: false,
          render: (value) => (
            <span className={`status-badge ${value ? 'gray' : 'blue'}`}>{value ? 'User defined' : 'System'}</span>
          ),
        },
      ]}
      columns={['id', 'code', 'name', 'is_user_defined']}
      createEndpoint="admin/customers/groups/store"
      updateEndpoint="admin/customers/groups/edit/{id}"
      deleteEndpoint="admin/customers/groups/{id}"
      transform={(values, editing) => ({
        code: values.code,
        name: values.name,
        ...(editing ? {} : { is_user_defined: 1 }),
      })}
      emptyLabel="No customer groups found."
    />
  );
}

export function ReviewsPage() {
  const [version, setVersion] = useState(0);

  const moderate = async (id: number, status: string) => {
    try {
      await api.adminPut(`admin/customers/reviews/${id}/status`, { status });
      setVersion((value) => value + 1);
    } catch (error: any) {
      window.alert(error?.message ?? 'Unable to update the review.');
    }
  };

  return (
    <div>
      <AdminPageHeader
        title="Reviews"
        subtitle="Moderate product reviews before they appear in the storefront."
      />
      <div className="admin-card admin-card-pad">
        <DataGrid
          key={version}
          path="admin/customers/reviews"
          emptyLabel="No reviews found."
          actions={(record) => (
            <>
              <button type="button" className="admin-icon-action" title="Approve" onClick={() => moderate(record.__primary ?? record.id, 'approved')}>
                <IconCheck size={15} />
              </button>
              <button type="button" className="admin-icon-action danger" title="Reject" onClick={() => moderate(record.__primary ?? record.id, 'rejected')}>
                <IconX size={15} />
              </button>
            </>
          )}
        />
      </div>
    </div>
  );
}
