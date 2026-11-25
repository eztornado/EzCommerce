import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Modal } from '@mantine/core';
import {
  IconArrowLeft,
  IconCalendar,
  IconCheck,
  IconCreditCard,
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
import { t } from '../../i18n';

const TABS = [
  { key: 'items', label: 'Order items' },
  { key: 'invoices', label: 'Invoices' },
  { key: 'refunds', label: 'Refunds' },
  { key: 'transactions', label: 'Transactions' },
  { key: 'shipments', label: 'Shipments' },
  { key: 'comments', label: 'Comments' },
];

function Field({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="order-field">
      <span className="order-field-label">{t(label)}</span>
      <span className="order-field-value">{value || '—'}</span>
    </div>
  );
}

function AddressCard({ title, address, email }: { title: string; address: any; email?: string }) {
  if (!address) {
    return (
      <div className="order-card">
        <h3 className="order-card-title">{t(title)}</h3>
        <p className="admin-muted">{t('No address on file.')}</p>
      </div>
    );
  }
  return (
    <div className="order-card">
      <h3 className="order-card-title">{t(title)}</h3>
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
      setError(requestError?.message ?? t('Unable to load the order.'));
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
      setError(requestError?.message ?? t('Unable to complete the action.'));
    } finally {
      setBusy('');
    }
  };

  const createInvoice = () => {
    if (!window.confirm(t('Create an invoice for all invoiceable items?'))) return;
    run('invoice', () => api.adminPost(`admin/sales/invoices/create/${order.id}`, {}), t('The invoice has been created successfully.'));
  };

  const capturePayment = () => {
    if (!window.confirm(t('Capture this authorized Stripe payment?'))) return;
    run('capture', () => api.adminPost(`admin/sales/orders/${order.id}/capture`, {}), t('The payment has been captured successfully.'));
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
      t('The shipment has been created successfully.'),
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
      t('The refund has been created successfully.'),
    );
  };

  const cancelOrder = () => {
    if (!window.confirm(t('Cancel this order? This action cannot be undone.'))) return;
    run('cancel', () => api.adminPost(`admin/sales/orders/${order.id}/cancel`, {}), t('The order has been canceled.'));
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
      t('The comment has been added.'),
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
        <AdminPageHeader title={t('Loading order…')} backTo="/admin/sales/orders" />
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
        <AdminPageHeader title={t('Order')} backTo="/admin/sales/orders" />
        <div className="admin-card admin-card-pad">
          <div className="admin-alert danger">{error || t('Order not found.')}</div>
        </div>
      </div>
    );
  }

  const invoices = order.invoices ?? [];
  const refunds = order.refunds ?? [];
  const shipments = order.shipments ?? [];
  const comments = order.comments ?? [];
  const transactions = order.transactions ?? [];
  const payment = order.payment ?? {};
  const paymentDetails = payment.additional ?? {};
  const latestTransaction = transactions[0];
  const transactionId =
    paymentDetails.payment_intent_id ??
    paymentDetails.transaction_id ??
    latestTransaction?.transaction_id;
  const cardBrand = paymentDetails.brand;
  const cardLast4 = paymentDetails.last4;
  const receiptUrl = paymentDetails.receipt_url;
  const paymentStatus = order.payment_status ?? (payment.method ? 'pending' : null);

  return (
    <div>
      <AdminPageHeader
        title={`Order #${order.increment_id}`}
        backTo="/admin/sales/orders"
        subtitle={`${t('Placed')} ${formatDate(order.created_at, true)} · ${order.customer_email}`}
        actions={
          <div className="order-actions">
            {order.can_invoice && (
              <button type="button" className="admin-button" onClick={createInvoice} disabled={!!busy}>
                <IconFileInvoice size={15} /> {busy === 'invoice' ? t('Invoicing…') : t('Invoice')}
              </button>
            )}
            {order.can_capture && (
              <button type="button" className="admin-button" onClick={capturePayment} disabled={!!busy}>
                <IconCreditCard size={15} /> {busy === 'capture' ? t('Capturing…') : t('Capture payment')}
              </button>
            )}
            {order.can_ship && (
              <button type="button" className="admin-button" onClick={openShipment} disabled={!!busy}>
                <IconTruckDelivery size={15} /> {t('Ship')}
              </button>
            )}
            {order.can_refund && (
              <button type="button" className="admin-button" onClick={openRefund} disabled={!!busy}>
                <IconReceiptRefund size={15} /> {t('Refund')}
              </button>
            )}
            {order.can_cancel && (
              <button type="button" className="admin-button danger" onClick={cancelOrder} disabled={!!busy}>
                <IconX size={15} /> {busy === 'cancel' ? t('Cancelling…') : t('Cancel')}
              </button>
            )}
          </div>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="order-meta">
        <span className={`status-badge ${statusColor(order.status)}`}>{t(formatLabel(order.status))}</span>
        {paymentStatus && (
          <span className={`status-badge ${statusColor(paymentStatus)}`}>
            {t('Payment')}: {t(formatLabel(paymentStatus))}
          </span>
        )}
        <span className="order-meta-item">
          <IconCalendar size={14} /> {formatDate(order.created_at, true)}
        </span>
        <span className="order-meta-item">
          <IconMail size={14} /> {order.customer_email}
        </span>
        <span className="order-meta-item">
          <IconPackage size={14} /> {t('{count} item(s) to fulfil', { count: totalRemaining })}
        </span>
        <span className="order-meta-item">{order.is_guest ? t('Guest checkout') : t('Registered customer')}</span>
      </div>

      <div className="order-layout">
        <div className="order-main">
          <div className="admin-card">
            <AdminTabs tabs={TABS.map((item) => ({
              ...item,
              count:
                item.key === 'invoices'
                  ? invoices.length
                  : item.key === 'refunds'
                    ? refunds.length
                    : item.key === 'transactions'
                      ? transactions.length
                      : item.key === 'shipments'
                        ? shipments.length
                        : item.key === 'comments'
                          ? comments.length
                          : order.items?.length,
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
                          {item.sku} · {money(item.base_price, order.base_currency_code ?? 'EUR')} {t('per unit')}
                        </p>
                        <p className="order-item-meta">
                          {t('Ordered')} {item.qty_ordered ?? 0}
                          {item.qty_invoiced ? ` · ${t('Invoiced')} ${item.qty_invoiced}` : ''}
                          {item.qty_shipped ? ` · ${t('Shipped')} ${item.qty_shipped}` : ''}
                          {item.qty_refunded ? ` · ${t('Refunded')} ${item.qty_refunded}` : ''}
                          {item.qty_canceled ? ` · ${t('Canceled')} ${item.qty_canceled}` : ''}
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
                  <div className="admin-empty">{t('No invoices have been raised for this order.')}</div>
                ) : (
                  <table className="admin-table order-subtable">
                    <thead>
                      <tr>
                        <th>{t('Invoice')}</th>
                        <th>{t('State')}</th>
                        <th>{t('Total')}</th>
                        <th>{t('Date')}</th>
                        <th className="actions">{t('Action')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {invoices.map((invoice: any) => (
                        <tr key={invoice.id}>
                          <td>{invoice.increment_id}</td>
                          <td>
                            <span className={`status-badge ${statusColor(invoice.state)}`}>{t(formatLabel(invoice.state))}</span>
                          </td>
                          <td>{money(invoice.base_grand_total, invoice.base_currency_code ?? 'EUR')}</td>
                          <td>{formatDate(invoice.created_at, true)}</td>
                          <td className="actions">
                            <button
                              type="button"
                              className="admin-link"
                              onClick={() => api.openAdminDocument(`admin/sales/invoices/print/${invoice.id}`)}
                            >
                              <IconPrinter size={14} /> {t('Print')}
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )
              )}

              {tab === 'refunds' && (
                refunds.length === 0 ? (
                  <div className="admin-empty">{t('No refunds have been issued for this order.')}</div>
                ) : (
                  <table className="admin-table order-subtable">
                    <thead>
                      <tr>
                        <th>{t('Refund')}</th>
                        <th>{t('State')}</th>
                        <th>{t('Total')}</th>
                        <th>{t('Transaction ID')}</th>
                        <th>{t('Date')}</th>
                        <th className="actions">{t('Action')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {refunds.map((refund: any) => (
                        <tr key={refund.id}>
                          <td>{refund.increment_id}</td>
                          <td>
                            <span className={`status-badge ${statusColor(refund.state)}`}>
                              {t(formatLabel(refund.state))}
                            </span>
                          </td>
                          <td>{money(refund.base_grand_total, refund.base_currency_code ?? order.base_currency_code ?? 'EUR')}</td>
                          <td>{refund.transaction_id || '—'}</td>
                          <td>{formatDate(refund.created_at, true)}</td>
                          <td className="actions">
                            {refund.transaction_id?.startsWith('re_') && (
                              <a
                                className="admin-link"
                                href={`https://dashboard.stripe.com/refunds/${encodeURIComponent(refund.transaction_id)}`}
                                target="_blank"
                                rel="noreferrer"
                              >
                                {t('Open in Stripe')}
                              </a>
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )
              )}

              {tab === 'transactions' && (
                transactions.length === 0 ? (
                  <div className="admin-empty">{t('No payment transactions have been recorded for this order.')}</div>
                ) : (
                  <table className="admin-table order-subtable">
                    <thead>
                      <tr>
                        <th>{t('Date')}</th>
                        <th>{t('Type')}</th>
                        <th>{t('Status')}</th>
                        <th>{t('Transaction ID')}</th>
                        <th>{t('Method')}</th>
                        <th style={{ textAlign: 'right' }}>{t('Amount')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {transactions.map((transaction: any) => (
                        <tr key={transaction.id}>
                          <td>{formatDate(transaction.created_at, true)}</td>
                          <td>{t(formatLabel(transaction.type) || '—')}</td>
                          <td>
                            <span className={`status-badge ${statusColor(transaction.status)}`}>
                              {t(formatLabel(transaction.status) || '—')}
                            </span>
                          </td>
                          <td>
                            <code>{transaction.transaction_id || '—'}</code>
                          </td>
                          <td>{transaction.payment_method || '—'}</td>
                          <td style={{ textAlign: 'right' }}>
                            {money(
                              transaction.base_amount ?? transaction.amount,
                              transaction.currency_code ?? order.base_currency_code ?? 'EUR',
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )
              )}

              {tab === 'shipments' && (
                shipments.length === 0 ? (
                  <div className="admin-empty">{t('No shipments have been created for this order.')}</div>
                ) : (
                  <table className="admin-table order-subtable">
                    <thead>
                      <tr>
                        <th>{t('Shipment')}</th>
                        <th>{t('Carrier')}</th>
                        <th>{t('Tracking')}</th>
                        <th>{t('Qty')}</th>
                        <th>{t('Date')}</th>
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
                      placeholder={t('Add an internal note or notify the customer about this order…')}
                      value={comment}
                      onChange={(event) => setComment(event.target.value)}
                    />
                    <label className="order-checkbox">
                      <input type="checkbox" checked={notify} onChange={(event) => setNotify(event.target.checked)} />
                      {t('Notify customer by email')}
                    </label>
                    <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                      <button type="button" className="admin-button primary" onClick={addComment} disabled={busy === 'comment' || !comment.trim()}>
                        <IconCheck size={15} /> {busy === 'comment' ? t('Adding…') : t('Add comment')}
                      </button>
                    </div>
                  </div>
                  {comments.length === 0 ? (
                    <div className="admin-empty">{t('No comments yet.')}</div>
                  ) : (
                    <div className="order-comment-list">
                      {comments.map((entry: any) => (
                        <div className="order-comment" key={entry.id}>
                          <div className="order-comment-head">
                            <span>{formatDate(entry.created_at, true)}</span>
                            {!!entry.customer_notified && <span className="status-badge blue">{t('Customer notified')}</span>}
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
            <h3 className="order-card-title">{t('Order summary')}</h3>
            <div className="order-total-row">
              <span>{t('Subtotal')}</span>
              <span>{money(order.base_sub_total, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>{t('Shipping')}</span>
              <span>{money(order.base_shipping_amount, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>{t('Tax')}</span>
              <span>{money(order.base_tax_amount, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>{t('Discount')}</span>
              <span>−{money(order.base_discount_amount, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row grand">
              <span>{t('Grand total')}</span>
              <span>{money(order.base_grand_total, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>{t('Total paid')}</span>
              <span>{money(order.base_grand_total_invoiced, order.base_currency_code ?? 'EUR')}</span>
            </div>
            <div className="order-total-row">
              <span>{t('Refunded')}</span>
              <span>{money(order.base_grand_total_refunded, order.base_currency_code ?? 'EUR')}</span>
            </div>
          </div>

          <div className="order-card">
            <h3 className="order-card-title">{t('Customer')}</h3>
            <Field label={t('Name')} value={`${order.customer_first_name ?? ''} ${order.customer_last_name ?? ''}`.trim()} />
            <Field label={t('Email')} value={order.customer_email} />
            <Field label={t('Customer ID')} value={order.customer_id ? <Link className="admin-link" to={`/admin/customers/view/${order.customer_id}`}>#{order.customer_id}</Link>  : t('Guest')} />
            <Field label={t('Channel')} value={order.channel_name} />
          </div>

          <AddressCard title={t('Shipping address')} address={order.shipping_address} email={order.customer_email} />
          <AddressCard title={t('Billing address')} address={order.billing_address} email={order.customer_email} />

          <div className="order-card">
            <h3 className="order-card-title">{t('Payment & shipping')}</h3>
            <Field label={t('Payment method')} value={payment.method_title ?? payment.method ?? '—'} />
            <Field
              label={t('Payment status')}
              value={
                paymentStatus ? (
                  <span className={`status-badge ${statusColor(paymentStatus)}`}>{t(formatLabel(paymentStatus))}</span>
                ) : (
                  '—'
                )
              }
            />
            {(cardBrand || cardLast4) && (
              <Field
                label={t('Card')}
                value={[cardBrand ? formatLabel(cardBrand) : '', cardLast4 ? `•••• ${cardLast4}` : ''].filter(Boolean).join(' ')}
              />
            )}
            {transactionId && (
              <Field label={t('Transaction ID')} value={<code>{transactionId}</code>} />
            )}
            {receiptUrl && (
              <Field
                label={t('Receipt')}
                value={
                  <a className="admin-link" href={receiptUrl} target="_blank" rel="noreferrer">
                    {t('View receipt')}
                  </a>
                }
              />
            )}
            {transactionId && (
              <Field
                label={t('Stripe')}
                value={
                  <a
                    className="admin-link"
                    href={`https://dashboard.stripe.com/payments/${encodeURIComponent(transactionId)}`}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {t('Open in Stripe')}
                  </a>
                }
              />
            )}
            {paymentDetails.dispute_status && (
              <Field
                label={t('Dispute status')}
                value={
                  <span className={`status-badge ${statusColor(paymentDetails.dispute_status)}`}>
                    {t(formatLabel(paymentDetails.dispute_status))}
                  </span>
                }
              />
            )}
            {paymentDetails.dispute_reason && (
              <Field label={t('Dispute reason')} value={formatLabel(paymentDetails.dispute_reason)} />
            )}
            {paymentDetails.dispute_id && (
              <Field label={t('Dispute ID')} value={<code>{paymentDetails.dispute_id}</code>} />
            )}
            <Field label={t('Shipping method')} value={order.shipping_title ?? (formatLabel(order.shipping_method) || '—')} />
            <Field label={t('Currency')} value={order.order_currency_code ?? order.currency_code ?? 'EUR'} />
          </div>
        </aside>
      </div>

      <Modal opened={shipOpen} onClose={() => setShipOpen(false)} title={t('Create shipment · Order #{id}', { id: order.increment_id })} size="lg" centered>
        <div className="admin-modal-form">
          <label className="admin-field">
            <span className="admin-field-label">{t('Inventory source')}</span>
            <select className="admin-input" value={shipSource} onChange={(event) => setShipSource(event.target.value)}>
              {sources.length === 0 && <option value="1">{t('Default')}</option>}
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
                <th>{t('Item')}</th>
                <th>{t('Ordered')}</th>
                <th>{t('Qty to ship')}</th>
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
              {t('Cancel')}
            </button>
            <button type="button" className="admin-button primary" onClick={createShipment}>
              {t('Create shipment')}
            </button>
          </div>
        </div>
      </Modal>

      <Modal opened={refundOpen} onClose={() => setRefundOpen(false)} title={t('Create refund · Order #{id}', { id: order.increment_id })} size="lg" centered>
        <div className="admin-modal-form">
          <table className="admin-table">
            <thead>
              <tr>
                <th>{t('Item')}</th>
                <th>{t('Invoiced')}</th>
                <th>{t('Qty to refund')}</th>
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
              <span className="admin-field-label">{t('Adjustment refund')}</span>
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
              <span className="admin-field-label">{t('Adjustment fee')}</span>
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
              {t('Cancel')}
            </button>
            <button type="button" className="admin-button primary" onClick={createRefund}>
              {t('Create refund')}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
