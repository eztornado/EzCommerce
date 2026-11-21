import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { IconLock, IconMail, IconUser, IconShoppingBag } from '@tabler/icons-react';
import { api } from '../../api/client';
import { t } from '../../i18n';

export default function RegisterPage() {
  const navigate = useNavigate();
  const [form, setForm] = useState({
    first_name: '',
    last_name: '',
    email: '',
    password: '',
    password_confirmation: '',
    subscribed_to_news_letter: false,
  });
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  const set = (field: string, value: string | boolean) => setForm((current) => ({ ...current, [field]: value }));

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    setBusy(true);
    try {
      const data = await api.post('customer/register', form);
      if (data?.token) {
        localStorage.setItem('customer_token', data.token);
        window.location.assign('/account');
      } else {
        navigate('/login');
      }
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to create your account.'));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="auth-shell">
      <div className="auth-card" style={{ maxWidth: 560 }}>
        <div className="auth-logo">
          <IconShoppingBag size={22} />
        </div>
        <h1 className="auth-title">{t('Create an account')}</h1>
        <p className="shop-muted" style={{ textAlign: 'center', fontSize: 13, marginBottom: 26 }}>
          {t('Join EzCommerce for a faster checkout, order tracking and exclusive offers.')}
        </p>

        <form onSubmit={submit}>
          <div className="two-col">
            <div>
              <label className="field-label" htmlFor="first_name">
                {t('First name')}
              </label>
              <div className="input-icon">
                <IconUser size={16} />
                <input id="first_name" className="form-input" value={form.first_name} onChange={(event) => set('first_name', event.target.value)} required />
              </div>
            </div>
            <div>
              <label className="field-label" htmlFor="last_name">
                {t('Last name')}
              </label>
              <input id="last_name" className="form-input" value={form.last_name} onChange={(event) => set('last_name', event.target.value)} required />
            </div>
          </div>

          <label className="field-label" htmlFor="email">
            {t('Email address')}
          </label>
          <div className="input-icon">
            <IconMail size={16} />
            <input id="email" type="email" className="form-input" value={form.email} onChange={(event) => set('email', event.target.value)} required />
          </div>

          <div className="two-col">
            <div>
              <label className="field-label" htmlFor="password">
                {t('Password')}
              </label>
              <div className="input-icon">
                <IconLock size={16} />
                <input id="password" type="password" className="form-input" value={form.password} onChange={(event) => set('password', event.target.value)} required minLength={6} />
              </div>
            </div>
            <div>
              <label className="field-label" htmlFor="password_confirmation">
                {t('Confirm password')}
              </label>
              <input
                id="password_confirmation"
                type="password"
                className="form-input"
                value={form.password_confirmation}
                onChange={(event) => set('password_confirmation', event.target.value)}
                required
                minLength={6}
              />
            </div>
          </div>

          <label className="checkout-check" style={{ marginTop: 10 }}>
            <input
              type="checkbox"
              checked={form.subscribed_to_news_letter}
              onChange={(event) => set('subscribed_to_news_letter', event.target.checked)}
            />
            {t('Subscribe to the EzCommerce newsletter')}
          </label>

          {error && <p className="form-error">{error}</p>}

          <button type="submit" className="shop-primary-button" style={{ width: '100%', marginTop: 16 }} disabled={busy}>
            {busy ? t('Creating account…') : t('Create account')}
          </button>
        </form>

        <p className="auth-switch">
          {t('Already have an account?')}{' '}
          <Link to="/login" className="shop-link">
            {t('Sign in')}
          </Link>
        </p>
      </div>
    </div>
  );
}
