import { useCallback, useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { api } from '../../api/client';
import { formatDate } from '../../lib/format';
import { AdminPageHeader, Badge } from '../../components/admin/ui';

const RMA_STATUSES = [
  { id: 1, title: 'Pending Review' },
  { id: 2, title: 'Approved' },
  { id: 3, title: 'Awaiting Return' },
  { id: 4, title: 'Return In Transit' },
  { id: 5, title: 'Refunded' },
  { id: 6, title: 'Solved' },
  { id: 7, title: 'Request Declined' },
  { id: 8, title: 'Item Canceled' },
  { id: 9, title: 'Request Canceled' },
];

export function RmaPage() {
  const [rows, setRows] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [page, setPage] = useState(1);
  const [meta, setMeta] = useState<any>({ current_page: 1, last_page: 1, total: 0 });
  const [search, setSearch] = useState('');

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const payload = await api.adminGet(`admin/rma?page=${page}&per_page=20`);
      setRows(Array.isArray(payload?.data) ? payload.data : []);
      setMeta(payload?.meta ?? { current_page: page, last_page: 1, total: 0 });
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to load return requests.');
    } finally {
      setLoading(false);
    }
  }, [page]);

  useEffect(() => {
    load();
  }, [load]);

  const updateStatus = async (id: number, statusId: number) => {
    try {
      await api.adminPost(`admin/rma/status/${id}`, { rma_status_id: statusId });
      setMessage('The RMA status has been updated.');
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to update the RMA.');
    }
  };

  const filtered = rows.filter((row) =>
    !search.trim() ? true : `${row.increment_id} ${row.status_title} ${row.package_condition}`.toLowerCase().includes(search.toLowerCase()),
  );

  return (
    <div>
      <AdminPageHeader
        title="Returns (RMA)"
        subtitle="Review return and refund requests raised by customers."
      />
      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}
      <div className="admin-card admin-card-pad">
        <div className="admin-resource-toolbar">
          <input
            className="admin-input"
            style={{ maxWidth: 280 }}
            placeholder="Search by order or status…"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
          <div style={{ flex: 1 }} />
          <button type="button" className="admin-toolbar-button" onClick={load}>
            Refresh
          </button>
        </div>
        <div className="admin-table-wrap" style={{ marginTop: 12, boxShadow: 'none' }}>
          <table className="admin-table">
            <thead>
              <tr>
                <th>RMA</th>
                <th>Order</th>
                <th>Status</th>
                <th>Package condition</th>
                <th>Information</th>
                <th>Created</th>
                <th className="actions">Update status</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                Array.from({ length: 4 }).map((_, index) => (
                  <tr key={index}>
                    <td colSpan={7}>
                      <div className="admin-skeleton-row" />
                    </td>
                  </tr>
                ))
              ) : filtered.length === 0 ? (
                <tr>
                  <td colSpan={7}>
                    <div className="admin-empty">No return requests found.</div>
                  </td>
                </tr>
              ) : (
                filtered.map((row) => (
                  <tr key={row.id}>
                    <td>
                      <span className="admin-code">#{row.id}</span>
                    </td>
                    <td>
                      <Link className="admin-link" to={`/admin/sales/orders/view/${row.order_id}`}>
                        #{row.increment_id}
                      </Link>
                    </td>
                    <td>
                      <Badge status={row.status_title} label={row.status_title ?? 'Pending'} />
                    </td>
                    <td>{row.package_condition || '—'}</td>
                    <td className="admin-table-clip">{row.information || '—'}</td>
                    <td>{formatDate(row.created_at, true)}</td>
                    <td className="actions">
                      <select
                        className="admin-input admin-input-inline"
                        value={row.rma_status_id ?? ''}
                        onChange={(event) => updateStatus(row.id, Number(event.target.value))}
                      >
                        {RMA_STATUSES.map((status) => (
                          <option key={status.id} value={status.id}>
                            {status.title}
                          </option>
                        ))}
                      </select>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
        {meta.last_page > 1 && (
          <div className="admin-pagination">
            <span>
              Page {meta.current_page} of {meta.last_page} · {meta.total} request(s)
            </span>
            <div style={{ display: 'flex', gap: 6 }}>
              <button type="button" className="admin-page-button" disabled={page <= 1} onClick={() => setPage((value) => value - 1)}>
                Previous
              </button>
              <button
                type="button"
                className="admin-page-button"
                disabled={page >= meta.last_page}
                onClick={() => setPage((value) => value + 1)}
              >
                Next
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export function GdprPage() {
  const [rows, setRows] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const payload = await api.adminGet('admin/gdpr');
      setRows(Array.isArray(payload) ? payload : []);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to load GDPR requests.');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const update = async (id: number, status: string) => {
    try {
      await api.adminPost(`admin/gdpr/status/${id}`, { status });
      setMessage(`The request has been marked as ${status}.`);
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to update the request.');
    }
  };

  return (
    <div>
      <AdminPageHeader
        title="GDPR Requests"
        subtitle="Handle customer data export and deletion requests."
      />
      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}
      <div className="admin-card admin-card-pad">
        <table className="admin-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Customer</th>
              <th>Type</th>
              <th>Status</th>
              <th>Message</th>
              <th>Requested</th>
              <th className="actions">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              Array.from({ length: 4 }).map((_, index) => (
                <tr key={index}>
                  <td colSpan={7}>
                    <div className="admin-skeleton-row" />
                  </td>
                </tr>
              ))
            ) : rows.length === 0 ? (
              <tr>
                <td colSpan={7}>
                  <div className="admin-empty">No GDPR requests found.</div>
                </td>
              </tr>
            ) : (
              rows.map((row) => (
                <tr key={row.id}>
                  <td>#{row.id}</td>
                  <td>{row.email ?? row.customer_id}</td>
                  <td>
                    <span className="admin-pill">{row.type}</span>
                  </td>
                  <td>
                    <Badge status={row.status} />
                  </td>
                  <td className="admin-table-clip">{row.message || '—'}</td>
                  <td>{formatDate(row.created_at, true)}</td>
                  <td className="actions">
                    <div className="admin-row-actions">
                      {row.type === 'export' && row.status === 'approved' && row.customer_id && (
                        <Link className="admin-link" to={`/admin/customers/view/${row.customer_id}`}>
                          View customer
                        </Link>
                      )}
                      {row.status !== 'approved' && (
                        <button type="button" className="admin-link" onClick={() => update(row.id, 'approved')}>
                          Approve
                        </button>
                      )}
                      {row.status !== 'declined' && (
                        <button type="button" className="admin-link danger" onClick={() => update(row.id, 'declined')}>
                          Decline
                        </button>
                      )}
                    </div>
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
