import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Modal } from '@mantine/core';
import {
  IconArrowLeft,
  IconCalendar,
  IconCheck,
  IconFileInvoice,
  IconMail,
  IconPackage,
  IconPrinter,
  IconReceiptRefund,
  IconTruckDelivery,
  IconX,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { formatDate, formatLabel, money, mediaUrl, statusColor } from '../../lib/format';
import { AdminPageHeader, AdminTabs } from '../../components/admin/ui';

const TABS = [
  { key: 'items', label: 'Order items' },
  { key: 'invoices', label: 'Invoices' },
  { key: 'shipments', label: 'Shipments' },
  { key: 'comments', label: 'Comments' },
];

function Field({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="order-field">
      <span className="order-field-label">{label}</span>
      <span className="order-field-value">{value || '—'}</span>
    </div>
  );
}

function AddressCard({ title, address, email }: { title: string; address: any; email?: string }) {
  if (!address) {
    return (
      <div className="order-card">
        <h3 className="order-card-title">{title}</h3>
        <p className="admin-muted">No address on file.</p>
      </div>
    );
  }
  return (
    <div className="order-card">
      <h3 className="order-card-title">{title}</h3>
      <p className="order-address-name">
        {address.first_name} {address.last_name}
      </p>
      <p>{address.company_name}</p>
      <p>{address.address1}</p>
      {address.address2 && <p>{address.address2}</p>}
      <p>
        {address.postcode} {address.city}
      </p>
      <p>{address.state}</p>
      <p>{address.country}</p>
      {address.phone && <p className="admin-muted">{address.phone}</p>}
      {(address.email || email) && <p className="admin-muted">{address.email || email}</p>}
    </div>
  );
}

export default function OrderViewPage() {
  const { id } = useParams();
  const [order, setOrder] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [tab, setTab] = useState('items');
  const [sources, setSources] = useState<any[]>([]);
  const [busy, setBusy] = useState('');

  const [shipOpen, setShipOpen] = useState(false);
  const [shipSource, setShipSource] = useState('1');
  const [shipItems, setShipItems] = useState<Record<string, number>>({});

  const [refundOpen, setRefundOpen] = useState(false);
  const [refundItems, setRefundItems] = useState<Record<string, number>>({});
  const [adjustmentRefund, setAdjustmentRefund] = useState('');
  const [adjustmentFee, setAdjustmentFee] = useState('');

  const [comment, setComment] = useState('');
  const [notify, setNotify] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      setOrder(await api.adminGet(`admin/sales/orders/view/${id}`));
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to load the order.');
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    load();
    api
      .adminGet('admin/settings/inventory-sources')
      .then((data) => setSources(Array.isArray(data) ? data.filter((source) => source.status === 1) : []))
      .catch(() => undefined);
  }, [load]);

  const run = async (label: string, action: () => Promise<any>, success: string) => {
    setBusy(label);
    setError('');
    setMessage('');
    try {
      await action();
      setMessage(success);
      await load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to complete the action.');
    } finally {
      setBusy('');
    }
  };

  const createInvoice = () => {
    if (!window.confirm('Create an invoice for all invoiceable items?')) return;
    run('invoice', () => api.adminPost(`admin/sales/invoices/create/${order.id}`, {}), 'The invoice has been created successfully.');
  };

  const openShipment = () => {
    const defaults: Record<string, number> = {};
    (order?.items ?? []).forEach((item: any) => {
      const remaining = Math.max(0, (item.qty_ordered ?? 0) - (item.qty_shipped ?? 0) - (item.qty_canceled ?? 0));
      if (remaining > 0) defaults[String(item.id)] = remaining;
    });
    setShipItems(defaults);
    setShipSource(String(sources[0]?.id ?? 1));
    setShipOpen(true);
  };

  const createShipment = () => {
    const items: Record<string, number> = {};
    Object.entries(shipItems).forEach(([key, qty]) => {
      if (qty > 0) items[key] = qty;
    });
    setShipOpen(false);
    run(
      'shipment',
      () => api.adminPost(`admin/sales/shipments/create/${order.id}`, { source_id: Number(shipSource), items }),
      'The shipment has been created successfully.',
    );
  };

  const openRefund = () => {
    const defaults: Record<string, number> = {};
    (order?.items ?? []).forEach((item: any) => {
      const remaining = Math.max(0, (item.qty_invoiced || item.qty_ordered || 0) - (item.qty_refunded ?? 0));
      if (remaining > 0) defaults[String(item.id)] = remaining;
    });
    setRefundItems(defaults);
    setAdjustmentRefund('');
    setAdjustmentFee('');
    setRefundOpen(true);
  };

  const createRefund = () => {
    const items: Record<string, number> = {};
    Object.entries(refundItems).forEach(([key, qty]) => {
      if (qty > 0) items[key] = qty;
    });
    setRefundOpen(false);
    run(
      'refund',
      () =>
        api.adminPost(`admin/sales/refunds/create/${order.id}`, {
          items,
          adjustment_refund: Number(adjustmentRefund || 0),
          adjustment_fee: Number(adjustmentFee || 0),
        }),
      'The refund has been created successfully.',
    );
  };

  const cancelOrder = () => {
    if (!window.confirm('Cancel this order? This action cannot be undone.')) return;
    run('cancel', () => api.adminPost(`admin/sales/orders/${order.id}/cancel`, {}), 'The order has been canceled.');
  };

  const addComment = () => {
    if (!comment.trim()) return;
    run(
      'comment',
      async () => {
        await api.adminPost(`admin/sales/orders/comment/${order.id}`, {
          comment: comment.trim(),
          customer_notified: notify ? 1 : 0,
        });
        setComment('');
        setNotify(false);
      },
      'The comment has been added.',
    );
  };

  const totalRemaining = useMemo(
    () =>
      (order?.items ?? []).reduce(
        (sum: number, item: any) => sum + Math.max(0, (item.qty_ordered ?? 0) - (item.qty_shipped ?? 0) - (item.qty_canceled ?? 0)),
        0,
      ),
    [order],
  );

  if (loading) {
    return (
      <div>
        <AdminPageHeader title="Loading order…" backTo="/admin/sales/orders" />
        <div className="admin-card admin-card-pad">
          {Array.from({ length: 6 }).map((_, index) => (
            <div className="admin-skeleton-row" key={index} />
          ))}
        </div>
      </div>
    );
  }

  if (!order) {
    return (
      <div>
        <AdminPageHeader title="Order" backTo="/admin/sales/orders" />
        <div className="admin-card admin-card-pad">
          <div className="admin-alert danger">{error || 'Order not found.'}</div>
        </div>
      </div>
    );
  }

  const invoices = order.invoices ?? [];
  const shipments = order.shipments ?? [];
  const comments = order.comments ?? [];

  return (
    <div>
      <AdminPageHeader
        title={`Order #${order.increment_id}`}
        backTo="/admin/sales/orders"
        subtitle={`Placed ${formatDate(order.created_at, true)} · ${order.customer_email}`}
        actions={
          <div className="order-actions">
            {order.can_invoice && (
              <button type="button" className="admin-button" onClick={createInvoice} disabled={!!busy}>
                <IconFileInvoice size={15} /> {busy === 'invoice' ? 'Invoicing…' : 'Invoice'}
              </button>
            )}
            {order.can_ship && (
              <button type="button" className="admin-button" onClick={openShipment} disabled={!!busy}>
                <IconTruckDelivery size={15} /> Ship
              </button>
            )}
            {order.can_refund && (
              <button type="button" className="admin-button" onClick={openRefund} disabled={!!busy}>
                <IconReceiptRefund size={15} /> Refund
              </button>
            )}
            {order.can_cancel && (
              <button type="button" className="admin-button danger" onClick={cancelOrder} disabled={!!busy}>
                <IconX size={15} /> {busy === 'cancel' ? 'Cancelling…' : 'Cancel'}
              </button>
            )}
          </div>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="order-meta">
        <span className={`status-badge ${statusColor(order.status)}`}>{order.status}</span>
        <span className="order-meta-item">
          <IconCalendar size={14} /> {formatDate(order.created_at, true)}
        </span>
        <span className="order-meta-item">
          <IconMail size={14} /> {order.customer_email}
        </span>
        <span className="order-meta-item">
          <IconPackage size={14} /> {totalRemaining} item(s) to fulfil
        </span>
        <span className="order-meta-item">{order.is_guest ? 'Guest checkout' : 'Registered customer'}</span>
      </div>

      <div className="order-layout">
        <div className="order-main">
          <div className="admin-card">
            <AdminTabs tabs={TABS.map((item) => ({
              ...item,
              count: item.key === 'invoices' ? invoices.length : item.key === 'shipments' ? shipments.length : item.key === 'comments' ? comments.length : order.items?.length,
            }))} active={tab} onChange={setTab} />

            <div className="admin-card-pad">
              {tab === 'items' && (
                <div className="order-items">
                  {(order.items ?? []).map((item: any) => (
                    <div className="order-item" key={item.id}>
                      <div className="order-item-media">
                        {item.base_image?.url ? (
                          <img src={mediaUrl(item.base_image.url)} alt={item.name} />
                        ) : (
                          <span>{item.name?.slice(0, 1)}</span>
                        )}
                      </div>
                      <div className="order-item-info">
                        <p className="order-item-name">{item.name}</p>
                        <p className="order-item-meta">
                          {item.sku} · {money(item.base_price, order.base_currency_code ?? 'EUR')} per unit
                        </p>
                        <p className="order-item-meta">
                          Ordered {item.qty_ordered ?? 0}
                          {item.qty_invoiced ? ` · Invoiced ${item.qty_invoiced}` : ''}
                          {item.qty_shipped ? ` · Shipped ${item.qty_shipped}` : ''}
                          {item.qty_refunded ? ` · Refunded ${item.qty_refunded}` : ''}
                          {item.qty_canceled ? ` · Canceled ${item.qty_canceled}` : ''}
                        </p>
                      </div>
                      <div className="order-item-total">
                        {money(item.base_total, order.base_currency_code ?? 'EUR')}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {tab === 'invoices' && (
                invoices.length === 0 ? (
                  <div className="admin-empty">No invoices have been raised for this order.</div>
                ) : (
                  <table className="admin-table order-subtable">
                    <thead>
                      <tr>
                        <th>Invoice</th>
                        <th>State</th>
                        <th>Total</th>
                        <th>Date</th>
                        <th className="actions">Action</th>
                      </tr>
                    </thead>
                    <tbody>
                      {invoices.map((invoice: any) => (
                        <tr key={invoice.id}>
                          <td>{invoice.increment_id}</td>
                          <td>
                            <span className={`status-badge ${statusColor(invoice.state)}`}>{invoice.state}</span>
                          </td>
                          <td>{money(invoice.base_grand_total, invoice.base_currency_code ?? 'EUR')}</td>
                          <td>{formatDate(invoice.created_at, true)}</td>
                          <td className="actions">
                            <button
                              type="button"
                              className="admin-link"
                              onClick={() => api.openAdminDocument(`admin/sales/invoices/print/${invoice.id}`)}
                            >
                              <IconPrinter size={14} /> Print
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )
              )}

              {tab === 'shipments' && (
                shipments.length === 0 ? (
                  <div className="admin-empty">No shipments have been created for this order.</div>
                ) : (
                  <table className="admin-table order-subtable">
                    <thead>
                      <tr>
                        <th>Shipment</th>
                        <th>Carrier</th>
                        <th>Tracking</th>
                        <th>Qty</th>
                        <th>Date</th>
                      </tr>
                    </thead>
                    <tbody>
                      {shipments.map((shipment: any) => (
                        <tr key={shipment.id}>
                          <td>#{shipment.id}</td>
                          <td>{shipment.carrier_title || '—'}</td>
                          <td>{shipment.track_number || '—'}</td>
                          <td>{shipment.total_qty ?? 0}</td>
                          <td>{formatDate(shipment.created_at, true)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )
              )}

              {tab === 'comments' && (
                <div className="order-comments">
                  <div className="order-comment-form">
                    <textarea
                      className="admin-input"
                      rows={3}
                      placeholder="Add an internal note or notify the customer about this order…"
                      value={comment}
                      onChange={(event) => setComment(event.target.value)}
                    />
                    <label className="order-checkbox">
                      <input type="checkbox" checked={notify} onChange={(event) => setNotify(event.target.checked)} />
                      Notify customer by email
                    </label>
                    <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                      <button type="button" className="admin-button primary" onClick={addComment} disabled={busy === 'comment' || !comment.trim()}>
                        <IconCheck size={15} /> {busy === 'comment' ? 'Adding…' : 'Add comment'}
                      </button>
                    </div>
                  </div>
                  {comments.length === 0 ? (
                    <div className="admin-empty">No comments yet.</div>
                  ) : (
                    <div className="order-comment-list">
                      {comments.map((entry: any) => (
                        <div className="order-comment" key={entry.id}>
                          <div className="order-comment-head">
                            <span>{formatDate(entry.created_at, true)}</span>
                            {!!entry.customer_notified && <span className="status-badge blue">Customer notified</span>}
                          </div>
                          <p>{entry.comment}</p>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>

        <aside className="order-side">
          <div className="order-card">
            <h3 className="order-card-title">Order summary</h3>
            <div className="order-total-row">
              <span>Subtotal</span>
              <span>{money(order.base_sub_total, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>Shipping</span>
              <span>{money(order.base_shipping_amount, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>Tax</span>
              <span>{money(order.base_tax_amount, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>Discount</span>
              <span>−{money(order.base_discount_amount, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row grand">
              <span>Grand total</span>
              <span>{money(order.base_grand_total, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>Total paid</span>
              <span>{money(order.base_grand_total_invoiced, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>Refunded</span>
              <span>{money(order.base_grand_total_refunded, order.base_currency_code ?? 'EUR')}</span>
            </div>
          </div>

          <div className="order-card">
            <h3 className="order-card-title">Customer</h3>
            <Field label="Name" value={`${order.customer_first_name ?? ''} ${order.customer_last_name ?? ''}`.trim()} />
            <Field label="Email" value={order.customer_email} />
            <Field label="Customer ID" value={order.customer_id ? <Link className="admin-link" to={`/admin/customers/view/${order.customer_id}`}>#{order.customer_id}</Link> : 'Guest'} />
            <Field label="Channel" value={order.channel_name} />
          </div>

          <AddressCard title="Shipping address" address={order.shipping_address} email={order.customer_email} />
          <AddressCard title="Billing address" address={order.billing_address} email={order.customer_email} />

          <div className="order-card">
            <h3 className="order-card-title">Payment &amp; shipping</h3>
            <Field label="Payment method" value={order.payment?.method_title ?? order.payment?.method} />
            <Field label="Shipping method" value={order.shipping_title ?? (formatLabel(order.shipping_method) || '—')} />
            <Field label="Currency" value={order.order_currency_code ?? order.currency_code ?? 'EUR'} />
          </div>
        </aside>
      </div>

      <Modal opened={shipOpen} onClose={() => setShipOpen(false)} title={`Create shipment · Order #${order.increment_id}`} size="lg" centered>
        <div className="admin-modal-form">
          <label className="admin-field">
            <span className="admin-field-label">Inventory source</span>
            <select className="admin-input" value={shipSource} onChange={(event) => setShipSource(event.target.value)}>
              {sources.length === 0 && <option value="1">Default</option>}
              {sources.map((source) => (
                <option key={source.id} value={source.id}>
                  {source.name} ({source.code})
                </option>
              ))}
            </select>
          </label>
          <table className="admin-table">
            <thead>
              <tr>
                <th>Item</th>
                <th>Ordered</th>
                <th>Qty to ship</th>
              </tr>
            </thead>
            <tbody>
              {(order.items ?? []).map((item: any) => {
                const remaining = Math.max(0, (item.qty_ordered ?? 0) - (item.qty_shipped ?? 0) - (item.qty_canceled ?? 0));
                return (
                  <tr key={item.id}>
                    <td>
                      {item.name}
                      <div className="admin-muted admin-small">{item.sku}</div>
                    </td>
                    <td>{item.qty_ordered}</td>
                    <td>
                      <input
                        className="admin-input"
                        type="number"
                        min={0}
                        max={remaining}
                        disabled={remaining === 0}
                        value={shipItems[String(item.id)] ?? 0}
                        onChange={(event) =>
                          setShipItems((current) => ({
                            ...current,
                            [String(item.id)]: Math.max(0, Math.min(remaining, Number(event.target.value))),
                          }))
                        }
                      />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          <div className="admin-modal-actions">
            <button type="button" className="admin-button" onClick={() => setShipOpen(false)}>
              Cancel
            </button>
            <button type="button" className="admin-button primary" onClick={createShipment}>
              Create shipment
            </button>
          </div>
        </div>
      </Modal>

      <Modal opened={refundOpen} onClose={() => setRefundOpen(false)} title={`Create refund · Order #${order.increment_id}`} size="lg" centered>
        <div className="admin-modal-form">
          <table className="admin-table">
            <thead>
              <tr>
                <th>Item</th>
                <th>Invoiced</th>
                <th>Qty to refund</th>
              </tr>
            </thead>
            <tbody>
              {(order.items ?? []).map((item: any) => {
                const remaining = Math.max(0, (item.qty_invoiced || item.qty_ordered || 0) - (item.qty_refunded ?? 0));
                return (
                  <tr key={item.id}>
                    <td>
                      {item.name}
                      <div className="admin-muted admin-small">{item.sku}</div>
                    </td>
                    <td>{item.qty_invoiced || item.qty_ordered}</td>
                    <td>
                      <input
                        className="admin-input"
                        type="number"
                        min={0}
                        max={remaining}
                        disabled={remaining === 0}
                        value={refundItems[String(item.id)] ?? 0}
                        onChange={(event) =>
                          setRefundItems((current) => ({
                            ...current,
                            [String(item.id)]: Math.max(0, Math.min(remaining, Number(event.target.value))),
                          }))
                        }
                      />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          <div className="admin-form-grid">
            <label className="admin-field">
              <span className="admin-field-label">Adjustment refund</span>
              <input
                className="admin-input"
                type="number"
                step="0.01"
                value={adjustmentRefund}
                onChange={(event) => setAdjustmentRefund(event.target.value)}
                placeholder="0.00"
              />
            </label>
            <label className="admin-field">
              <span className="admin-field-label">Adjustment fee</span>
              <input
                className="admin-input"
                type="number"
                step="0.01"
                value={adjustmentFee}
                onChange={(event) => setAdjustmentFee(event.target.value)}
                placeholder="0.00"
              />
            </label>
          </div>
          <div className="admin-modal-actions">
            <button type="button" className="admin-button" onClick={() => setRefundOpen(false)}>
              Cancel
            </button>
            <button type="button" className="admin-button primary" onClick={createRefund}>
              Create refund
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
