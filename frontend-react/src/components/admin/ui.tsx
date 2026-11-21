import { Link } from 'react-router-dom';
import { statusColor, statusLabel } from '../../lib/format';
import { t } from '../../i18n';

export function AdminPageHeader({
  title,
  subtitle,
  actions,
  backTo,
}: {
  title: string;
  subtitle?: string;
  actions?: React.ReactNode;
  backTo?: string;
}) {
  return (
    <div className="admin-page-head">
      <div>
        {backTo && (
          <Link to={backTo} className="admin-back-link">
            ← {t('Back')}
          </Link>
        )}
        <h1 className="admin-page-title">{t(title)}</h1>
        {subtitle && <p className="admin-page-subtitle">{t(subtitle)}</p>}
      </div>
      {actions && <div className="admin-page-actions">{actions}</div>}
    </div>
  );
}

export function AdminCard({ title, actions, children, className = '', noPad = false }: { title?: string; actions?: React.ReactNode; children: React.ReactNode; className?: string; noPad?: boolean }) {
  return (
    <section className={`admin-card ${className}`}>
      {(title || actions) && (
        <div className="admin-card-head">
          {title && <h2 className="admin-card-title">{t(title)}</h2>}
          {actions && <div className="admin-card-actions">{actions}</div>}
        </div>
      )}
      <div className={noPad ? '' : 'admin-card-pad'}>{children}</div>
    </section>
  );
}

export function Badge({ status, label }: { status?: string | number | null; label?: string }) {
  return (
    <span className={`status-badge ${statusColor(status)}`} style={{ textTransform: 'capitalize' }}>
      {label ? t(label) : statusLabel(status)}
    </span>
  );
}

export function AdminField({ label, hint, children, required }: { label: string; hint?: string; children: React.ReactNode; required?: boolean }) {
  return (
    <label className="admin-field">
      <span className="admin-field-label">
        {t(label)} {required && <em>*</em>}
      </span>
      {children}
      {hint && <span className="admin-field-hint">{t(hint)}</span>}
    </label>
  );
}

export function AdminEmpty({ title, text, action }: { title: string; text?: string; action?: React.ReactNode }) {
  return (
    <div className="admin-empty">
      <h3>{t(title)}</h3>
      {text && <p>{t(text)}</p>}
      {action}
    </div>
  );
}

export function AdminTabs({ tabs, active, onChange }: { tabs: { key: string; label: string; count?: number }[]; active: string; onChange: (key: string) => void }) {
  return (
    <div className="admin-tabs">
      {tabs.map((tab) => (
        <button key={tab.key} type="button" className="admin-tab" data-active={active === tab.key} onClick={() => onChange(tab.key)}>
          {t(tab.label)}
          {tab.count !== undefined && <span className="admin-tab-count">{tab.count}</span>}
        </button>
      ))}
    </div>
  );
}
