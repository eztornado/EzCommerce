import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import {
  IconArrowUpRight,
  IconCurrencyEuro,
  IconShoppingCart,
  IconUsers,
  IconChartHistogram,
  IconBox,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { formatDate, statusColor } from '../../lib/format';
import { AdminPageHeader } from '../../components/admin/ui';

interface SeriesPoint {
  label: string;
  orders: number;
  sales: number;
}

export default function DashboardPage() {
  const [stats, setStats] = useState<any>(null);
  const [sales, setSales] = useState<{ series: SeriesPoint[]; totals: any }>({ series: [], totals: null });
  const [products, setProducts] = useState<any[]>([]);
  const [orders, setOrders] = useState<any[]>([]);
  const [customers, setCustomers] = useState<any[]>([]);

  useEffect(() => {
    api.adminGet('admin/dashboard/stats').then(setStats).catch(() => undefined);
    api.adminGet('admin/reporting/sales').then(setSales).catch(() => undefined);
    api.adminGet('admin/reporting/products').then((data) => setProducts(Array.isArray(data) ? data.slice(0, 6) : [])).catch(() => undefined);
    api.adminGet('admin/reporting/customers').then((data) => setCustomers(Array.isArray(data) ? data.slice(0, 4) : [])).catch(() => undefined);
    api
      .adminPost('admin/sales/orders', {
        filters: {},
        sort: { column: 'created_at', order: 'desc' },
        pagination: { page: 1, per_page: 6 },
      })
      .then((data) => setOrders(data?.records ?? []))
      .catch(() => undefined);
  }, []);

  return (
    <div>
      <AdminPageHeader
        title="Hi! Administrator"
        subtitle="Quickly review what's going on in your store."
        actions={
          <Link to="/admin/reporting/sales" className="admin-button primary">
            <IconChartHistogram size={15} /> View reports
          </Link>
        }
      />

      <div className="admin-stat-grid">
        <Stat
          icon={<IconCurrencyEuro size={24} />}
          label="Total Sales"
          value={`€${Number(stats?.total_sales?.current ?? 0).toFixed(2)}`}
          progress={stats?.total_sales?.progress}
          tone="blue"
        />
        <Stat
          icon={<IconShoppingCart size={24} />}
          label="Total Orders"
          value={String(stats?.total_orders?.current ?? 0)}
          progress={stats?.total_orders?.progress}
          tone="violet"
        />
        <Stat
          icon={<IconUsers size={24} />}
          label="Total Customers"
          value={String(stats?.total_customers?.current ?? 0)}
          progress={stats?.total_customers?.progress}
          tone="emerald"
        />
        <Stat
          icon={<IconBox size={24} />}
          label="Average Order"
          value={`€${Number(stats?.avg_sales?.current ?? 0).toFixed(2)}`}
          progress={stats?.avg_sales?.progress}
          tone="amber"
        />
        <Stat
          icon={<IconArrowUpRight size={24} />}
          label="Orders Today"
          value={String(sales.series[sales.series.length - 1]?.orders ?? 0)}
          progress={100}
          tone="rose"
        />
      </div>

      <div className="admin-dashboard-grid">
        <section className="admin-card admin-chart-card">
          <div className="admin-card-head">
            <h2 className="admin-card-title">Sales overview</h2>
            <span className="admin-card-subtitle">Revenue over the reporting period</span>
          </div>
          <div className="admin-card-pad">
            <SalesChart series={sales.series} />
            <div className="admin-chart-legend">
              <span>
                <i style={{ background: '#598de6' }} /> Revenue
              </span>
              <span>
                <i style={{ background: '#c7d9f5' }} /> {Number(sales.totals?.total_orders ?? 0)} orders
              </span>
              <span style={{ marginLeft: 'auto' }}>
                Shipping collected: <strong>€{Number(sales.totals?.shipping_collected ?? 0).toFixed(2)}</strong>
              </span>
            </div>
          </div>
        </section>

        <section className="admin-card">
          <div className="admin-card-head">
            <h2 className="admin-card-title">Top selling products</h2>
            <Link to="/admin/reporting/products" className="admin-link">
              View all
            </Link>
          </div>
          <div className="admin-card-pad">
            {products.length === 0 ? (
              <p className="admin-muted">No sales recorded yet.</p>
            ) : (
              <div className="admin-top-products">
                {products.map((product, index) => {
                  const max = Number(products[0]?.total ?? 1);
                  return (
                    <div className="admin-top-product" key={product.product_id ?? product.sku ?? index}>
                      <span className="admin-rank">{index + 1}</span>
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <p className="admin-top-product-name">{product.name}</p>
                        <div className="admin-progress-track">
                          <span style={{ width: `${Math.max(6, (Number(product.total) / max) * 100)}%` }} />
                        </div>
                        <p className="admin-muted admin-small">
                          {product.sku} · {product.qty} sold
                        </p>
                      </div>
                      <strong className="admin-top-product-total">€{Number(product.total ?? 0).toFixed(2)}</strong>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </section>

        <section className="admin-card admin-span-2">
          <div className="admin-card-head">
            <h2 className="admin-card-title">Recent orders</h2>
            <Link to="/admin/sales/orders" className="admin-link">
              View all orders
            </Link>
          </div>
          <div className="admin-card-pad" style={{ paddingTop: 0 }}>
            <table className="admin-table" style={{ minWidth: 640 }}>
              <thead>
                <tr>
                  <th>Order</th>
                  <th>Customer</th>
                  <th>Status</th>
                  <th>Total</th>
                  <th>Date</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {orders.map((order) => (
                  <tr key={order.__primary}>
                    <td>
                      <Link to={`/admin/sales/orders/view/${order.__primary}`} className="admin-link" style={{ fontWeight: 600 }}>
                        #{order.increment_id}
                      </Link>
                    </td>
                    <td>{order.customer_email}</td>
                    <td>
                      <span className={`status-badge ${statusColor(order.status)}`} style={{ textTransform: 'capitalize' }}>
                        {order.status}
                      </span>
                    </td>
                    <td>{order.base_grand_total}</td>
                    <td>{formatDate(order.created_at)}</td>
                    <td className="actions">
                      <Link to={`/admin/sales/orders/view/${order.__primary}`} className="admin-link">
                        View
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="admin-card">
          <div className="admin-card-head">
            <h2 className="admin-card-title">Top customers</h2>
            <Link to="/admin/customers" className="admin-link">
              View all
            </Link>
          </div>
          <div className="admin-card-pad">
            {customers.length === 0 ? (
              <p className="admin-muted">No customers yet.</p>
            ) : (
              <div className="admin-customer-list">
                {customers.map((customer) => (
                  <div key={customer.id} className="admin-customer-row">
                    <span className="admin-avatar small">
                      {(customer.first_name?.[0] ?? '') + (customer.last_name?.[0] ?? '')}
                    </span>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p style={{ margin: 0, fontWeight: 600, color: '#111827' }}>
                        {customer.first_name} {customer.last_name}
                      </p>
                      <p className="admin-muted admin-small" style={{ margin: 0 }}>
                        {customer.email}
                      </p>
                    </div>
                    <div style={{ textAlign: 'right' }}>
                      <strong>€{Number(customer.total ?? 0).toFixed(2)}</strong>
                      <p className="admin-muted admin-small" style={{ margin: 0 }}>
                        {customer.orders} orders
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}

function Stat({ icon, label, value, progress, tone }: { icon: React.ReactNode; label: string; value: string; progress?: number; tone: string }) {
  return (
    <div className="admin-stat">
      <span className={`admin-stat-icon ${tone}`}>{icon}</span>
      <div style={{ minWidth: 0 }}>
        <div className="admin-stat-value">{value}</div>
        <div className="admin-stat-label">{label}</div>
        <div className="admin-stat-progress" style={{ color: '#047857' }}>
          <IconArrowUpRight size={11} /> {Number(progress ?? 0).toFixed(0)}% vs last period
        </div>
      </div>
    </div>
  );
}

function SalesChart({ series }: { series: SeriesPoint[] }) {
  if (!series.length) {
    return <div className="admin-chart-empty">No sales data available for the selected period.</div>;
  }

  const width = 720;
  const height = 280;
  const padding = { top: 16, right: 12, bottom: 34, left: 54 };
  const innerWidth = width - padding.left - padding.right;
  const innerHeight = height - padding.top - padding.bottom;
  const max = Math.max(...series.map((point) => point.sales), 1);
  const step = innerWidth / series.length;
  const barWidth = Math.min(30, step * 0.46);

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="admin-chart" role="img" aria-label="Revenue chart">
      {[0, 0.25, 0.5, 0.75, 1].map((fraction) => {
        const y = padding.top + innerHeight * fraction;
        return (
          <g key={fraction}>
            <line x1={padding.left} x2={width - padding.right} y1={y} y2={y} stroke="#f1f5f9" strokeWidth="1" />
            <text x={padding.left - 10} y={y + 4} textAnchor="end" fontSize="12" fill="#9ca3af">
              €{Math.round(max * (1 - fraction))}
            </text>
          </g>
        );
      })}

      {series.map((point, index) => {
        const barHeight = Math.max((point.sales / max) * innerHeight, point.sales > 0 ? 3 : 0);
        const x = padding.left + step * index + (step - barWidth) / 2;
        const y = padding.top + innerHeight - barHeight;
        return (
          <g key={point.label}>
            <rect x={x} y={y} width={barWidth} height={barHeight} rx="4" fill="#598de6">
              <title>{`€${point.sales.toFixed(2)} · ${point.orders} orders`}</title>
            </rect>
            <text x={x + barWidth / 2} y={height - 12} textAnchor="middle" fontSize="12" fill="#6b7280">
              {new Date(point.label).toLocaleDateString('en-GB', { day: '2-digit', month: 'short' })}
            </text>
          </g>
        );
      })}
    </svg>
  );
}
