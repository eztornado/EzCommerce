import { useEffect, useMemo, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import {
  IconChartBar,
  IconDownload,
  IconTrendingUp,
  IconShoppingBag,
  IconReceipt,
  IconTruck,
  IconArrowBackUp,
  IconTicket,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { AdminPageHeader } from '../../components/admin/ui';
import { t } from '../../i18n';
import { money } from '../../lib/format';

type Module = 'sales' | 'products' | 'customers';

const TABS: { key: Module; label: string }[] = [
  { key: 'sales', label: 'Sales' },
  { key: 'products', label: 'Products' },
  { key: 'customers', label: 'Customers' },
];

const RANGES = [
  { key: '7', label: 'Last 7 days', days: 7 },
  { key: '30', label: 'Last 30 days', days: 30 },
  { key: '90', label: 'Last 90 days', days: 90 },
  { key: '365', label: 'Last 12 months', days: 365 },
];

function dateRange(days: number) {
  const end = new Date();
  const start = new Date();
  start.setDate(end.getDate() - days);
  return { start: start.toISOString().slice(0, 10), end: end.toISOString().slice(0, 10) };
}

export default function ReportingPage() {
  const params = useParams();
  const module = (['sales', 'products', 'customers'].includes(String(params.module)) ? params.module : 'sales') as Module;
  const [range, setRange] = useState('30');
  const [granularity, setGranularity] = useState<'day' | 'week' | 'month'>('day');
  const [sales, setSales] = useState<any>(null);
  const [rows, setRows] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const { start, end } = useMemo(() => dateRange(Number(range)), [range]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError('');
    const query = `?start=${start}&end=${end}${module === 'sales' ? `&granularity=${granularity}` : ''}`;
    api
      .adminGet(`admin/reporting/${module}${query}`)
      .then((data) => {
        if (cancelled) return;
        if (module === 'sales') {
          setSales(data ?? null);
          setRows([]);
        } else {
          setRows(Array.isArray(data) ? data : data?.data ?? []);
          setSales(null);
        }
      })
      .catch((requestError) => {
        if (!cancelled) {
          setError(requestError?.message ?? t('Unable to load the report.'));
          setRows([]);
          setSales(null);
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [module, start, end, granularity]);

  const exportCsv = async () => {
    try {
      await api.downloadAdminFile(
        `admin/reporting/${module}?format=csv&start=${start}&end=${end}${module === 'sales' ? `&granularity=${granularity}` : ''}`,
        `${module}-report.csv`,
      );
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to export the report.'));
    }
  };

  const series: any[] = sales?.series ?? [];
  const maxSales = Math.max(...series.map((point) => Number(point.sales ?? 0)), 1);
  const totals = sales?.totals ?? {};
  const totalRevenue = module === 'products' || module === 'customers'
    ? rows.reduce((sum, row) => sum + Number(row.total ?? 0), 0)
    : 0;

  const metrics = [
    { key: 'total_sales', label: 'Total sales', value: money(totals.total_sales ?? 0), icon: <IconTrendingUp size={17} /> },
    { key: 'total_orders', label: 'Orders', value: totals.total_orders ?? 0, icon: <IconShoppingBag size={17} /> },
    { key: 'tax_collected', label: 'Tax collected', value: money(totals.tax_collected ?? 0), icon: <IconReceipt size={17} /> },
    { key: 'shipping_collected', label: 'Shipping collected', value: money(totals.shipping_collected ?? 0), icon: <IconTruck size={17} /> },
    { key: 'refunds', label: 'Refunds', value: money(totals.refunds ?? 0), icon: <IconArrowBackUp size={17} /> },
    { key: 'coupon_discount', label: 'Coupon discount', value: money(totals.coupon_discount ?? 0), icon: <IconTicket size={17} /> },
  ];

  return (
    <div>
      <AdminPageHeader
        title={t('Reporting')}
        subtitle={t('Analyse sales performance, best selling products and top customers.')}
        actions={
          <button type="button" className="admin-button" onClick={exportCsv}>
            <IconDownload size={15} /> {t('Export CSV')}
          </button>
        }
      />

      <div className="report-toolbar">
        <div className="admin-tabs" style={{ marginBottom: 0 }}>
          {TABS.map((tab) => (
            <Link
              key={tab.key}
              to={`/admin/reporting/${tab.key}`}
              className="admin-tab"
              data-active={module === tab.key}
            >
              {t(tab.label)}
            </Link>
          ))}
        </div>
        <div className="report-range">
          <select className="admin-input admin-input-inline" value={range} onChange={(event) => setRange(event.target.value)}>
            {RANGES.map((entry) => (
              <option key={entry.key} value={entry.key}>
                {t(entry.label)}
              </option>
            ))}
          </select>
          {module === 'sales' && (
            <select className="admin-input admin-input-inline" value={granularity} onChange={(event) => setGranularity(event.target.value as any)}>
              <option value="day">{t('Daily')}</option>
              <option value="week">{t('Weekly')}</option>
              <option value="month">{t('Monthly')}</option>
            </select>
          )}
        </div>
      </div>

      <p className="admin-muted report-period">
        {t('Showing data from')} <strong>{start}</strong> {t('to')} <strong>{end}</strong>.
      </p>

      {error && <div className="admin-alert danger">{error}</div>}

      {module === 'sales' ? (
        <>
          <div className="report-metrics">
            {metrics.map((metric) => (
              <div className="report-metric" key={metric.key}>
                <span className="report-metric-icon">{metric.icon}</span>
                <div>
                  <p>{t(metric.label)}</p>
                  <strong>{loading ? '—' : metric.value}</strong>
                </div>
              </div>
            ))}
          </div>

          <div className="admin-card admin-card-pad" style={{ marginTop: 16 }}>
            <div className="admin-card-head" style={{ padding: 0, marginBottom: 18 }}>
              <div>
                <h2 className="admin-card-title">{t('Sales over time')}</h2>
                <p className="admin-card-subtitle">{t('Net invoiced sales grouped by {granularity}.', { granularity: t(granularity === 'day' ? 'day' : granularity === 'week' ? 'week' : 'month') })}</p>
              </div>
              <span className="admin-pill">{t('{count} buckets', { count: series.length })}</span>
            </div>
            {loading ? (
              <div className="admin-skeleton-row" style={{ height: 220 }} />
            ) : series.length === 0 ? (
              <div className="admin-empty">
                <h3>{t('No sales in this period')}</h3>
                <p>{t('Try widening the date range or selecting a different granularity.')}</p>
              </div>
            ) : (
              <>
                <div className="report-chart">
                  {series.map((point, index) => {
                    const height = Math.max((Number(point.sales ?? 0) / maxSales) * 100, 1.5);
                    return (
                      <div className="report-bar-wrap" key={`${point.label}-${index}`}>
                        <div className="report-bar" style={{ height: `${height}%` }}>
                          <span className="report-bar-tooltip">
                            {point.label}: {money(point.sales)} · {t('{count} orders', { count: point.orders })}
                          </span>
                        </div>
                        <span className="report-bar-label">
                          {String(point.label).length > 7 ? String(point.label).slice(5) : point.label}
                        </span>
                      </div>
                    );
                  })}
                </div>
                <div className="report-chart-axis">
                  <span>0</span>
                  <span>{money(maxSales)}</span>
                </div>
              </>
            )}
          </div>

          <div className="admin-card admin-card-pad" style={{ marginTop: 16 }}>
            <h2 className="admin-card-title">{t('Breakdown')}</h2>
            <div className="report-table">
              <table className="admin-table">
                <thead>
                  <tr>
                    <th>{t('Period')}</th>
                    <th>{t('Orders')}</th>
                    <th>{t('Sales')}</th>
                  </tr>
                </thead>
                <tbody>
                  {series.length === 0 ? (
                    <tr>
                      <td colSpan={3} className="admin-muted">{t('No data available for the selected period.')}</td>
                    </tr>
                  ) : (
                    [...series].reverse().map((point, index) => (
                      <tr key={`${point.label}-${index}`}>
                        <td>{point.label}</td>
                        <td>{point.orders}</td>
                        <td>{money(point.sales)}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </>
      ) : (
        <>
          <div className="report-metrics report-metrics-3">
            <div className="report-metric">
              <span className="report-metric-icon"><IconChartBar size={17} /></span>
              <div>
                <p>{module === 'products' ? t('Products sold') : t('Top customers')}</p>
                <strong>{loading ? '—' : rows.length}</strong>
              </div>
            </div>
            <div className="report-metric">
              <span className="report-metric-icon"><IconTrendingUp size={17} /></span>
              <div>
                <p>{t('Revenue in report')}</p>
                <strong>{loading ? '—' : money(totalRevenue)}</strong>
              </div>
            </div>
            <div className="report-metric">
              <span className="report-metric-icon"><IconShoppingBag size={17} /></span>
              <div>
                <p>{module === 'products' ? t('Units sold') : t('Orders placed')}</p>
                <strong>
                  {loading ? '—' : rows.reduce((sum, row) => sum + Number(row.qty ?? row.orders ?? 0), 0)}
                </strong>
              </div>
            </div>
          </div>

          <div className="admin-card admin-card-pad" style={{ marginTop: 16 }}>
            <div className="admin-card-head" style={{ padding: 0, marginBottom: 14 }}>
              <div>
                <h2 className="admin-card-title">
                  {module === 'products' ? t('Top selling products') : t('Top customers')}
                </h2>
                <p className="admin-card-subtitle">{t('Ordered by revenue within the selected period.')}</p>
              </div>
            </div>
            <div className="report-table">
              <table className="admin-table">
                <thead>
                  <tr>
                    <th style={{ width: 60 }}>#</th>
                    <th>{module === 'products' ? t('Product') : t('Customer')}</th>
                    <th>{module === 'products' ? t('SKU') : t('Email')}</th>
                    <th>{module === 'products' ? t('Units') : t('Orders')}</th>
                    <th>{t('Revenue')}</th>
                  </tr>
                </thead>
                <tbody>
                  {loading ? (
                    Array.from({ length: 4 }).map((_, index) => (
                      <tr key={index}>
                        <td colSpan={5}><div className="admin-skeleton-row" /></td>
                      </tr>
                    ))
                  ) : rows.length === 0 ? (
                    <tr>
                      <td colSpan={5} className="admin-muted">{t('No records for the selected period.')}</td>
                    </tr>
                  ) : (
                    rows.map((row, index) => (
                      <tr key={`${row.id ?? row.product_id}-${index}`}>
                        <td>
                          <span className="admin-rank">{index + 1}</span>
                        </td>
                        <td>
                          {module === 'products'
                            ? row.name
                            : `${row.first_name ?? ''} ${row.last_name ?? ''}`.trim() || t('Guest')}
                        </td>
                        <td>{module === 'products' ? row.sku : row.email}</td>
                        <td>{row.qty ?? row.orders ?? 0}</td>
                        <td>{money(row.total)}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
