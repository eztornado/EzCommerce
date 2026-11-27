import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import '../../styles/admin.css';
import { IconLock, IconMail, IconShoppingBag } from '@tabler/icons-react';
import { useAdminAuth } from '../../store';
import { t } from '../../i18n';

export default function AdminLoginPage() {
  const login = useAdminAuth((state) => state.login);
  const navigate = useNavigate();
  const [email, setEmail] = useState('admin@example.com');
  const [password, setPassword] = useState('admin123');
  const [remember, setRemember] = useState(true);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    setBusy(true);
    try {
      await login(email.trim(), password);
      navigate('/admin');
    } catch (requestError: any) {
      setError(requestError?.message ?? t('The provided credentials are incorrect.'));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="admin-login-page">
      <div className="admin-login-card">
        <div className="admin-login-brand">
          <span className="admin-logo-mark" style={{ width: 40, height: 40, fontSize: 22 }}>
            E
          </span>
          <div>
            <strong>EzCommerce</strong>
            <span>{t('Administration panel')}</span>
          </div>
        </div>

        <h1 className="admin-login-title">{t('Sign in to your account')}</h1>
        <p className="admin-login-subtitle">{t('Use your administrator credentials to access the dashboard.')}</p>

        <form onSubmit={submit}>
          <label className="admin-field-label" htmlFor="admin-email">
            {t('Email address')}
          </label>
          <div className="input-icon">
            <IconMail size={16} />
            <input
              id="admin-email"
              type="email"
              className="form-input"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="admin@example.com"
              required
            />
          </div>

          <label className="admin-field-label" htmlFor="admin-password">
            {t('Password')}
          </label>
          <div className="input-icon">
            <IconLock size={16} />
            <input
              id="admin-password"
              type="password"
              className="form-input"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="••••••••"
              required
            />
          </div>

          <label className="checkout-check" style={{ marginTop: 4 }}>
            <input type="checkbox" checked={remember} onChange={(event) => setRemember(event.target.checked)} />
            {t('Remember me')}
          </label>

          {error && <p className="form-error">{error}</p>}

          <button type="submit" className="admin-button primary full" style={{ marginTop: 16 }} disabled={busy}>
            {busy ? t('Signing in…') : t('Sign in')}
          </button>
        </form>

        <div className="admin-login-demo">
          <IconShoppingBag size={15} />
          <span>
            {t('Demo credentials:')} <strong>admin@example.com</strong> / <strong>admin123</strong>
          </span>
        </div>
      </div>
    </div>
  );
}
