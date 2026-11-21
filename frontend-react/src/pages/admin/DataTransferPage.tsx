import { useRef, useState } from 'react';
import {
  IconUpload,
  IconDownload,
  IconFileTypeCsv,
  IconCheck,
  IconAlertTriangle,
  IconCloudUpload,
  IconX,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { AdminPageHeader } from '../../components/admin/ui';
import { t } from '../../i18n';

const SAMPLE = `sku,name,price,quantity,status
NEW-001,Canvas tote bag,39.90,25,1
NEW-002,Linen shirt,59.00,12,1`;

const EXPORTS = [
  {
    key: 'products',
    title: 'Products',
    description: 'Every catalog product with SKU, type, price, stock and URL key.',
    endpoint: 'admin/data-transfer/products/export',
    method: 'POST',
    filename: 'products.csv',
    columns: ['sku', 'name', 'type', 'price', 'quantity', 'status', 'url_key'],
  },
  {
    key: 'customers',
    title: 'Customers',
    description: 'Customer accounts with names, emails, status and group.',
    endpoint: 'admin/data-transfer/export-customers',
    method: 'GET',
    filename: 'customers.csv',
    columns: ['id', 'first_name', 'last_name', 'email', 'status', 'group_id'],
  },
  {
    key: 'orders',
    title: 'Orders',
    description: 'Order headers with totals, tax, shipping and creation date.',
    endpoint: 'admin/data-transfer/export-orders',
    method: 'GET',
    filename: 'orders.csv',
    columns: ['increment_id', 'status', 'customer_email', 'grand_total', 'tax', 'shipping', 'created_at'],
  },
];

export default function DataTransferPage() {
  const [tab, setTab] = useState<'import' | 'export'>('import');
  const [csv, setCsv] = useState('');
  const [fileName, setFileName] = useState('');
  const [importing, setImporting] = useState(false);
  const [downloading, setDownloading] = useState('');
  const [error, setError] = useState('');
  const [result, setResult] = useState<any>(null);
  const fileRef = useRef<HTMLInputElement | null>(null);

  const readFile = (file: File) => {
    const reader = new FileReader();
    reader.onload = () => {
      setCsv(String(reader.result ?? ''));
      setFileName(file.name);
      setResult(null);
      setError('');
    };
    reader.readAsText(file);
  };

  const download = async (entry: (typeof EXPORTS)[number]) => {
    setDownloading(entry.key);
    setError('');
    try {
      await api.downloadAdminFile(
        entry.endpoint,
        entry.filename,
        entry.method === 'POST' ? { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' } : undefined,
      );
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to export {item}.', { item: t(entry.title).toLowerCase() }));
    } finally {
      setDownloading('');
    }
  };

  const runImport = async () => {
    if (!csv.trim()) {
      setError(t('Paste a CSV or choose a file before starting the import.'));
      return;
    }
    setImporting(true);
    setError('');
    setResult(null);
    try {
      const data = await api.adminPost('admin/data-transfer/products/import', { csv });
      setResult(data);
    } catch (requestError: any) {
      setError(requestError?.message ?? t('The import could not be completed.'));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div>
      <AdminPageHeader
        title={t('Data Transfer')}
        subtitle={t('Import and export catalog, customer and order data as CSV files.')}
      />

      {error && <div className="admin-alert danger">{error}</div>}

      <div className="admin-tabs">
        <button type="button" className="admin-tab" data-active={tab === 'import'} onClick={() => setTab('import')}>
          {t('Import')}
        </button>
        <button type="button" className="admin-tab" data-active={tab === 'export'} onClick={() => setTab('export')}>
          {t('Export')}
        </button>
      </div>

      {tab === 'import' ? (
        <div className="data-transfer-layout">
          <section className="admin-card admin-card-pad">
            <div className="admin-card-head">
              <div>
                <h2 className="admin-card-title">{t('Import products')}</h2>
                <p className="admin-card-subtitle">
                  {t('Upload a CSV with')} <code>sku,name,price,quantity,status</code>. {t('Rows with an existing SKU are skipped and reported.')}
                </p>
              </div>
            </div>

            <div
              className="data-transfer-drop"
              onClick={() => fileRef.current?.click()}
              onDragOver={(event) => event.preventDefault()}
              onDrop={(event) => {
                event.preventDefault();
                const file = event.dataTransfer.files?.[0];
                if (file) readFile(file);
              }}
              role="button"
              tabIndex={0}
              onKeyDown={(event) => event.key === 'Enter' && fileRef.current?.click()}
            >
              <IconCloudUpload size={30} />
              <strong>{fileName || t('Drop your CSV here or click to browse')}</strong>
              <span>{t('Maximum recommended size: 10 MB')}</span>
              <input
                ref={fileRef}
                type="file"
                accept=".csv,text/csv"
                hidden
                onChange={(event) => {
                  const file = event.target.files?.[0];
                  if (file) readFile(file);
                }}
              />
            </div>

            <div className="data-transfer-or">
              <span>{t('or paste the CSV content')}</span>
            </div>

            <textarea
              className="admin-input admin-textarea data-transfer-code"
              rows={10}
              value={csv}
              spellCheck={false}
              placeholder={SAMPLE}
              onChange={(event) => {
                setCsv(event.target.value);
                setFileName('');
                setResult(null);
              }}
            />

            <div className="admin-modal-actions">
              <button type="button" className="admin-button" onClick={() => setCsv(SAMPLE)}>
                <IconFileTypeCsv size={15} /> {t('Use sample data')}
              </button>
              <button type="button" className="admin-button primary" onClick={runImport} disabled={importing}>
                <IconUpload size={15} /> {importing ? t('Importing…') : t('Start import')}
              </button>
            </div>
          </section>

          <aside className="admin-card admin-card-pad">
            <h2 className="admin-card-title">{t('Import summary')}</h2>
            {!result && !importing && (
              <p className="admin-muted" style={{ marginTop: 10 }}>
                {t('The result of your last import will appear here, including processed rows and any validation errors.')}
              </p>
            )}
            {importing && <div className="admin-skeleton-row" style={{ marginTop: 14 }} />}
            {result && (
              <>
                <div className={`admin-alert ${result.invalid_rows_count ? 'warning' : 'success'}`} style={{ marginTop: 12 }}>
                  {result.invalid_rows_count ? <IconAlertTriangle size={15} /> : <IconCheck size={15} />}
                  {t('{processed} rows imported, {invalid} invalid.', { processed: result.processed_rows_count ?? 0, invalid: result.invalid_rows_count ?? 0 })}
                </div>
                <dl className="data-transfer-stats">
                  <div>
                    <dt>{t('State')}</dt>
                    <dd style={{ textTransform: 'capitalize' }}>{t(result.state ?? 'completed').replace(/_/g, ' ')}</dd>
                  </div>
                  <div>
                    <dt>{t('Processed')}</dt>
                    <dd>{result.processed_rows_count ?? 0}</dd>
                  </div>
                  <div>
                    <dt>{t('Invalid')}</dt>
                    <dd>{result.invalid_rows_count ?? 0}</dd>
                  </div>
                </dl>
                {(result.errors ?? []).length > 0 && (
                  <div className="data-transfer-errors">
                    <p className="admin-small">{t('Errors')}</p>
                    <ul>
                      {(result.errors ?? []).slice(0, 20).map((entry: any, index: number) => (
                        <li key={index}>
                          <strong>{t('Row {number}', { number: entry.row })}</strong> {entry.error}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                <button type="button" className="admin-button" style={{ marginTop: 14 }} onClick={() => { setResult(null); setCsv(''); setFileName(''); }}>
                  <IconX size={15} /> {t('Clear')}
                </button>
              </>
            )}
          </aside>
        </div>
      ) : (
        <div className="data-transfer-export-grid">
          {EXPORTS.map((entry) => (
            <section className="admin-card admin-card-pad" key={entry.key}>
              <div className="data-transfer-export-head">
                <span className="data-transfer-export-icon">
                  <IconFileTypeCsv size={20} />
                </span>
                <div>
                  <h2 className="admin-card-title">{t(entry.title)}</h2>
                  <p className="admin-card-subtitle">{t(entry.description)}</p>
                </div>
              </div>
              <div className="data-transfer-columns">
                {entry.columns.map((column) => (
                  <code key={column}>{column}</code>
                ))}
              </div>
              <button
                type="button"
                className="admin-button primary"
                onClick={() => download(entry)}
                disabled={downloading === entry.key}
              >
                <IconDownload size={15} /> {downloading === entry.key ? t('Preparing…') : t('Export {item}', { item: t(entry.title).toLowerCase() })}
              </button>
            </section>
          ))}
        </div>
      )}
    </div>
  );
}
