import { useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router-dom';
import { IconLock, IconMail, IconShoppingBag } from '@tabler/icons-react';
import { useAuth } from '../../store';
import { t } from '../../i18n';

export default function SignInPage() {
  const login = useAuth((state) => state.login);
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    setBusy(true);
    try {
      await login(email.trim(), password);
      navigate(params.get('redirect') ?? '/account');
    } catch (requestError: any) {
      setError(requestError?.message ?? t('The provided credentials are incorrect.'));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="auth-shell">
      <div className="auth-card">
        <div className="auth-logo">
          <IconShoppingBag size={22} />
        </div>
        <h1 className="auth-title">{t('Welcome back')}</h1>
        <p className="shop-muted" style={{ textAlign: 'center', fontSize: 13, marginBottom: 26 }}>
          {t('Sign in to your EzCommerce account to continue.')}
        </p>

        <form onSubmit={submit}>
          <label className="field-label" htmlFor="email">
            {t('Email address')}
          </label>
          <div className="input-icon">
            <IconMail size={16} />
            <input
              id="email"
              type="email"
              className="form-input"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder={t('you@example.com')}
              required
            />
          </div>

          <label className="field-label" htmlFor="password">
            {t('Password')}
          </label>
          <div className="input-icon">
            <IconLock size={16} />
            <input
              id="password"
              type="password"
              className="form-input"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder={t('Your password')}
              required
            />
          </div>

          <div style={{ display: 'flex', justifyContent: 'flex-end', margin: '4px 0 18px' }}>
            <Link to="/forgot-password" className="shop-link" style={{ fontSize: 12 }}>
              {t('Forgot password?')}
            </Link>
          </div>

          {error && <p className="form-error">{error}</p>}

          <button type="submit" className="shop-primary-button" style={{ width: '100%' }} disabled={busy}>
            {busy ? t('Signing in…') : t('Sign in')}
          </button>
        </form>

        <p className="auth-switch">
          {t('New to EzCommerce?')}{' '}
          <Link to="/register" className="shop-link">
            {t('Create an account')}
          </Link>
        </p>

        <div className="auth-demo">
          <strong>{t('Demo account')}</strong>
          <span>customer@example.com · customer123</span>
        </div>
      </div>
    </div>
  );
}
