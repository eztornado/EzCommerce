import { useCallback, useEffect, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import { Modal } from '@mantine/core';
import { IconEdit, IconMail, IconPlus, IconTrash } from '@tabler/icons-react';
import { api } from '../../api/client';
import { formatDate, money, statusColor } from '../../lib/format';
import { AdminPageHeader, Badge } from '../../components/admin/ui';

export default function CustomerViewPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [data, setData] = useState<any>(null);
  const [groups, setGroups] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [open, setOpen] = useState(false);
  const [values, setValues] = useState<any>({});
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const payload = await api.adminGet(`admin/customers/view/${id}`);
      setData(payload);
      setValues(payload.customer ?? {});
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to load the customer.');
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    load();
    api.adminGet('admin/customers/groups').then((rows) => setGroups(Array.isArray(rows) ? rows : [])).catch(() => undefined);
  }, [load]);

  const save = async () => {
    setSaving(true);
    setError('');
    try {
      await api.adminPut(`admin/customers/edit/${id}`, {
        first_name: values.first_name,
        last_name: values.last_name,
        email: values.email,
        phone: values.phone,
        gender: values.gender,
        date_of_birth: values.date_of_birth,
        status: Number(values.status) ? 1 : 0,
        customer_group_id: Number(values.customer_group_id || 1),
        subscribed_to_news_letter: Number(values.subscribed_to_news_letter) ? 1 : 0,
      });
      setMessage('The customer has been updated successfully.');
      setOpen(false);
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to update the customer.');
    } finally {
      setSaving(false);
    }
  };

  const remove = async () => {
    if (!window.confirm('Delete this customer account? This action cannot be undone.')) return;
    try {
      await api.adminDelete(`admin/customers/${id}`);
      navigate('/admin/customers');
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to delete the customer.');
    }
  };

  if (loading) {
    return (
      <div>
        <AdminPageHeader title="Loading customer…" backTo="/admin/customers" />
        <div className="admin-card admin-card-pad">
          {Array.from({ length: 6 }).map((_, index) => (
            <div className="admin-skeleton-row" key={index} />
          ))}
        </div>
      </div>
    );
  }

  if (!data?.customer) {
    return (
      <div>
        <AdminPageHeader title="Customer" backTo="/admin/customers" />
        <div className="admin-card admin-card-pad">
          <div className="admin-alert danger">{error || 'Customer not found.'}</div>
        </div>
      </div>
    );
  }

  const customer = data.customer;
  const orders = data.orders ?? [];
  const addresses = data.addresses ?? [];
  const reviews = data.reviews ?? [];
  const totalSpent = orders.reduce((sum: number, order: any) => sum + Number(order.base_grand_total ?? 0), 0);

  return (
    <div>
      <AdminPageHeader
        title={`${customer.first_name} ${customer.last_name}`}
        backTo="/admin/customers"
        subtitle={customer.email}
        actions={
          <div style={{ display: 'flex', gap: 8 }}>
            <button type="button" className="admin-button" onClick={() => setOpen(true)}>
              <IconEdit size={15} /> Edit customer
            </button>
            <button type="button" className="admin-button danger" onClick={remove}>
              <IconTrash size={15} /> Delete account
            </button>
          </div>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="customer-view-grid">
        <div className="customer-view-main">
          <section className="admin-card">
            <div className="customer-card-head">
              <h2>Orders ({orders.length})</h2>
              <span>{money(totalSpent)} lifetime value</span>
            </div>
            {orders.length === 0 ? (
              <div className="admin-empty">This customer has not placed any orders yet.</div>
            ) : (
              <table className="admin-table">
                <thead>
                  <tr>
                    <th>Order</th>
                    <th>Status</th>
                    <th>Total</th>
                    <th>Date</th>
                  </tr>
                </thead>
                <tbody>
                  {orders.map((order: any) => (
                    <tr key={order.id}>
                      <td>
                        <Link className="admin-link" to={`/admin/sales/orders/view/${order.id}`}>
                          #{order.increment_id}
                        </Link>
                      </td>
                      <td>
                        <Badge status={order.status} />
                      </td>
                      <td>{money(order.base_grand_total)}</td>
                      <td>{formatDate(order.created_at)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </section>

          <section className="admin-card">
            <div className="customer-card-head">
              <h2>Addresses ({addresses.length})</h2>
            </div>
            {addresses.length === 0 ? (
              <div className="admin-empty">No saved addresses.</div>
            ) : (
              <div className="customer-address-grid">
                {addresses.map((address: any) => (
                  <div className="customer-address" key={address.id}>
                    <p className="customer-address-name">
                      {address.first_name} {address.last_name}
                    </p>
                    <p>{address.address1}</p>
                    {address.address2 && <p>{address.address2}</p>}
                    <p>
                      {address.postcode} {address.city}
                    </p>
                    <p>{address.country}</p>
                    <p className="admin-muted">{address.phone}</p>
                  </div>
                ))}
              </div>
            )}
          </section>

          <section className="admin-card">
            <div className="customer-card-head">
              <h2>Reviews ({reviews.length})</h2>
            </div>
            {reviews.length === 0 ? (
              <div className="admin-empty">No reviews written by this customer.</div>
            ) : (
              <div className="customer-review-list">
                {reviews.map((review: any) => (
                  <div className="customer-review" key={review.id}>
                    <div className="customer-review-head">
                      <strong>{review.title}</strong>
                      <Badge status={review.status} />
                    </div>
                    <div className="admin-muted admin-small">
                      {'★'.repeat(review.rating ?? 0)}
                      {'☆'.repeat(Math.max(0, 5 - (review.rating ?? 0)))} · {formatDate(review.created_at)}
                    </div>
                    <p>{review.comment}</p>
                  </div>
                ))}
              </div>
            )}
          </section>
        </div>

        <aside className="customer-view-side">
          <section className="admin-card admin-card-pad">
            <div className="customer-side-head">
              <span className="customer-avatar">
                {customer.first_name?.[0] ?? '?'}
                {customer.last_name?.[0] ?? ''}
              </span>
              <div>
                <h2 className="customer-side-name">
                  {customer.first_name} {customer.last_name}
                </h2>
                <Badge status={customer.status} label={customer.status ? 'Active' : 'Inactive'} />
              </div>
            </div>
            <dl className="customer-details">
              <div>
                <dt>Email</dt>
                <dd>{customer.email}</dd>
              </div>
              <div>
                <dt>Phone</dt>
                <dd>{customer.phone || '—'}</dd>
              </div>
              <div>
                <dt>Group</dt>
                <dd>{groups.find((group) => group.id === customer.customer_group_id)?.name ?? `#${customer.customer_group_id}`}</dd>
              </div>
              <div>
                <dt>Gender</dt>
                <dd>{customer.gender || '—'}</dd>
              </div>
              <div>
                <dt>Date of birth</dt>
                <dd>{customer.date_of_birth || '—'}</dd>
              </div>
              <div>
                <dt>Newsletter</dt>
                <dd>{customer.subscribed_to_news_letter ? 'Subscribed' : 'Not subscribed'}</dd>
              </div>
              <div>
                <dt>Wishlist</dt>
                <dd>{data.wishlist_count ?? 0} item(s)</dd>
              </div>
              <div>
                <dt>Customer since</dt>
                <dd>{formatDate(customer.created_at)}</dd>
              </div>
            </dl>
            <a className="admin-button" href={`mailto:${customer.email}`} style={{ width: '100%', justifyContent: 'center', marginTop: 14 }}>
              <IconMail size={15} /> Email customer
            </a>
          </section>
        </aside>
      </div>

      <Modal opened={open} onClose={() => setOpen(false)} title="Edit customer" size="lg" centered>
        <div className="admin-modal-form">
          <div className="admin-form-grid">
            {[
              { key: 'first_name', label: 'First name' },
              { key: 'last_name', label: 'Last name' },
              { key: 'email', label: 'Email', type: 'email' },
              { key: 'phone', label: 'Phone' },
              { key: 'date_of_birth', label: 'Date of birth', type: 'date' },
            ].map((field) => (
              <label className="admin-field" key={field.key}>
                <span className="admin-field-label">{field.label}</span>
                <input
                  className="admin-input"
                  type={field.type ?? 'text'}
                  value={values[field.key] ?? ''}
                  onChange={(event) => setValues((current: any) => ({ ...current, [field.key]: event.target.value }))}
                />
              </label>
            ))}
            <label className="admin-field">
              <span className="admin-field-label">Gender</span>
              <select
                className="admin-input"
                value={values.gender ?? ''}
                onChange={(event) => setValues((current: any) => ({ ...current, gender: event.target.value }))}
              >
                <option value="">Select…</option>
                <option value="Male">Male</option>
                <option value="Female">Female</option>
                <option value="Other">Other</option>
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Customer group</span>
              <select
                className="admin-input"
                value={values.customer_group_id ?? 1}
                onChange={(event) => setValues((current: any) => ({ ...current, customer_group_id: event.target.value }))}
              >
                {groups.map((group) => (
                  <option key={group.id} value={group.id}>
                    {group.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Status</span>
              <select
                className="admin-input"
                value={values.status ? '1' : '0'}
                onChange={(event) => setValues((current: any) => ({ ...current, status: event.target.value }))}
              >
                <option value="1">Active</option>
                <option value="0">Inactive</option>
              </select>
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Newsletter</span>
              <select
                className="admin-input"
                value={values.subscribed_to_news_letter ? '1' : '0'}
                onChange={(event) => setValues((current: any) => ({ ...current, subscribed_to_news_letter: event.target.value }))}
              >
                <option value="1">Subscribed</option>
                <option value="0">Not subscribed</option>
              </select>
            </label>
          </div>
          <div className="admin-modal-actions">
            <button type="button" className="admin-button" onClick={() => setOpen(false)}>
              Cancel
            </button>
            <button type="button" className="admin-button primary" disabled={saving} onClick={save}>
              {saving ? 'Saving…' : 'Save customer'}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
