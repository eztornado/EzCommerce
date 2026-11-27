import { useEffect, useState } from 'react';
import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import {
  IconChevronDown,
  IconHeart,
  IconSearch,
  IconShoppingBag,
  IconUser,
  IconMenu2,
  IconX,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { useAuth, useCart, useShop, shopQuery } from '../../store';
import { mediaUrl } from '../../lib/format';
import { useShopReload } from '../../lib/hooks';
import { t, setLocale, uiLocaleName, useLocale, type UiLocale } from '../../i18n';
import type { Category } from '../../types';

export default function ShopLayout() {
  const { context, brand, changeLocale, changeCurrency } = useShop();
  const brandLogo = context?.channel?.logo || brand?.logo || null;
  const brandName = context?.channel?.name || brand?.name || 'EzCommerce';
  const cartCount = useCart((state) => state.count);
  const refreshCart = useCart((state) => state.refresh);
  const loggedIn = useAuth((state) => state.loggedIn);
  const customer = useAuth((state) => state.customer);
  const reload = useShopReload();
  const locale = useLocale();
  const [categories, setCategories] = useState<Category[]>([]);
  const [query, setQuery] = useState('');
  const [mobileOpen, setMobileOpen] = useState(false);
  const [email, setEmail] = useState('');
  const [subscribed, setSubscribed] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    api
      .get(`categories/tree${shopQuery()}`)
      .then((data) => setCategories(Array.isArray(data) ? data : []))
      .catch(() => setCategories([]));
  }, [reload]);

  useEffect(() => {
    refreshCart();
  }, [refreshCart, reload, location.pathname]);

  useEffect(() => {
    const handler = () => refreshCart();
    window.addEventListener('cart-updated', handler);
    return () => window.removeEventListener('cart-updated', handler);
  }, [refreshCart]);

  useEffect(() => {
    window.scrollTo({ top: 0, behavior: 'instant' as ScrollBehavior });
    setMobileOpen(false);
  }, [location.pathname, location.search]);

  const submitSearch = (event: React.FormEvent) => {
    event.preventDefault();
    if (query.trim()) navigate(`/search?q=${encodeURIComponent(query.trim())}`);
  };

  const subscribe = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!email.trim()) return;
    try {
      await api.post('newsletter/subscribe', { email: email.trim() });
      setSubscribed(true);
      setEmail('');
    } catch {
      setSubscribed(true);
    }
  };

  // El idioma de la interfaz se puede previsualizar desde aquí; el idioma de
  // contenido (en/es) se mantiene separado porque el catálogo puede no estar
  // traducido en todos los locales.
  const chooseLanguage = async (code: UiLocale) => {
    setLocale(code);
    if (code !== 'ja' && code !== context?.locale) await changeLocale(code);
  };

  const activeCurrency = context?.currencies.find((currency) => currency.code === context?.currency);

  return (
    <div className="shop-shell">
      <div className="shop-topbar">
        <div className="shop-container shop-topbar-inner">
          <details className="shop-select">
            <summary className="topbar-select">
              {activeCurrency ? `${activeCurrency.symbol} ${activeCurrency.code}` : 'EUR €'} <IconChevronDown size={14} />
            </summary>
            <div className="shop-select-panel">
              {context?.currencies.map((currency) => (
                <button
                  type="button"
                  className="shop-select-option"
                  key={currency.code}
                  onClick={(event) => {
                    void changeCurrency(currency.code);
                    event.currentTarget.closest('details')?.removeAttribute('open');
                  }}
                >
                  {currency.symbol} {currency.code} — {currency.name}
                </button>
              ))}
            </div>
          </details>

          <p className="topbar-offer">
            {context?.header_offer ?? t('Free shipping on all orders over €100')}
            <Link to="/products"> {t('Shop now')}</Link>
          </p>

          <details className="shop-select" data-align="end">
            <summary className="topbar-select">
              {uiLocaleName(locale)} <IconChevronDown size={14} />
            </summary>
            <div className="shop-select-panel">
              {(['en', 'es', 'ja'] as UiLocale[]).map((code) => (
                <button
                  type="button"
                  className="shop-select-option"
                  key={code}
                  onClick={(event) => {
                    void chooseLanguage(code);
                    event.currentTarget.closest('details')?.removeAttribute('open');
                  }}
                >
                  {uiLocaleName(code)}
                </button>
              ))}
            </div>
          </details>
        </div>
      </div>

      <header className="shop-header">
        <div className="shop-container shop-header-main">
          <button
            type="button"
            className="mobile-menu-toggle"
            onClick={() => setMobileOpen((open) => !open)}
            aria-label={t('Menu')}
          >
            {mobileOpen ? <IconX size={22} /> : <IconMenu2 size={22} />}
          </button>

          <Link to="/" className="shop-logo">
            {brandLogo ? (
              <img src={mediaUrl(brandLogo)} alt={brandName} style={{ height: 30 }} />
            ) : brand ? (
              <>
                {brandName.slice(0, 2)}
                <span>{brandName.slice(2)}</span>
              </>
            ) : (
              // Primer arranque sin caché: evita mostrar la marca por defecto
              // mientras llega /core/context (y el posible logo configurado).
              <span aria-hidden="true" style={{ display: 'inline-block', width: 120, height: 30 }} />
            )}
          </Link>

          <nav className="shop-nav" aria-label={t('Main navigation')}>
            <NavLink to="/" end className="shop-nav-link" data-active={location.pathname === '/'}>
              {t('Home')}
            </NavLink>
            {categories.map((category) => (
              <NavLink
                key={category.id}
                to={`/category/${category.id}`}
                className="shop-nav-link"
                data-active={location.pathname === `/category/${category.id}`}
              >
                {category.name}
              </NavLink>
            ))}
            <NavLink to="/search?sort=newest" className="shop-nav-link">
              {t('New Arrivals')}
            </NavLink>
          </nav>

          <div className="shop-header-actions">
            <form className="shop-search" onSubmit={submitSearch} role="search">
              <span className="shop-search-icon">
                <IconSearch size={17} />
              </span>
              <input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={t('Search products...')}
                aria-label={t('Search products')}
              />
            </form>

            <div className="shop-icon-row">
              <Link to="/account/wishlist" className="shop-icon-button" aria-label={t('Wishlist')}>
                <IconHeart size={21} />
              </Link>

              <details className="shop-select shop-account-select">
                <summary className="shop-icon-button" aria-label={t('Account')}>
                  <IconUser size={21} />
                </summary>
                <div className="shop-select-panel">
                  {loggedIn ? (
                    <>
                      <div className="shop-select-label">
                        {t('Welcome, {name}', { name: customer?.first_name ?? t('Customer') })}
                      </div>
                      <Link
                        className="shop-select-option"
                        to="/account"
                        onClick={(event) => event.currentTarget.closest('details')?.removeAttribute('open')}
                      >
                        {t('My profile')}
                      </Link>
                      <Link
                        className="shop-select-option"
                        to="/account/orders"
                        onClick={(event) => event.currentTarget.closest('details')?.removeAttribute('open')}
                      >
                        {t('My orders')}
                      </Link>
                      <Link
                        className="shop-select-option"
                        to="/account/wishlist"
                        onClick={(event) => event.currentTarget.closest('details')?.removeAttribute('open')}
                      >
                        {t('Wishlist')}
                      </Link>
                      <button
                        type="button"
                        className="shop-select-option"
                        onClick={async () => {
                          await useAuth.getState().logout();
                          navigate('/');
                        }}
                      >
                        {t('Sign out')}
                      </button>
                    </>
                  ) : (
                    <>
                      <div className="shop-select-label">{t('Welcome to EzCommerce')}</div>
                      <Link
                        className="shop-select-option"
                        to="/login"
                        onClick={(event) => event.currentTarget.closest('details')?.removeAttribute('open')}
                      >
                        {t('Sign in')}
                      </Link>
                      <Link
                        className="shop-select-option"
                        to="/register"
                        onClick={(event) => event.currentTarget.closest('details')?.removeAttribute('open')}
                      >
                        {t('Create an account')}
                      </Link>
                    </>
                  )}
                </div>
              </details>

              <Link to="/cart" className="shop-icon-button" aria-label={t('Shopping cart')}>
                <IconShoppingBag size={21} />
                {cartCount > 0 && <span className="shop-cart-count">{cartCount}</span>}
              </Link>
            </div>
          </div>
        </div>

        {mobileOpen && (
          <div className="shop-mobile-nav">
            <Link to="/">{t('Home')}</Link>
            {categories.map((category) => (
              <Link key={category.id} to={`/category/${category.id}`}>
                {category.name}
              </Link>
            ))}
            <Link to="/search">{t('Search')}</Link>
            <Link to="/account">{t('My account')}</Link>
          </div>
        )}
      </header>

      <main>
        <Outlet />
      </main>

      <footer className="shop-footer">
        <div className="shop-container shop-footer-main">
          <div className="shop-footer-links">
            <div>
              <p className="shop-footer-column-title">{t('Quick Links')}</p>
              <ul>
                <li>
                  <Link to="/">{t('Home')}</Link>
                </li>
                <li>
                  <Link to="/search?sort=newest">{t('New arrivals')}</Link>
                </li>
                <li>
                  <Link to="/search">{t('All products')}</Link>
                </li>
                <li>
                  <Link to="/cart">{t('Shopping cart')}</Link>
                </li>
              </ul>
            </div>
            <div>
              <p className="shop-footer-column-title">{t('Customer Service')}</p>
              <ul>
                <li>
                  <Link to="/account">{t('My account')}</Link>
                </li>
                <li>
                  <Link to="/account/orders">{t('Order tracking')}</Link>
                </li>
                <li>
                  <Link to="/account/wishlist">{t('Wishlist')}</Link>
                </li>
                <li>
                  <Link to="/login">{t('Sign in')}</Link>
                </li>
              </ul>
            </div>
            <div>
              <p className="shop-footer-column-title">{t('Categories')}</p>
              <ul>
                {categories.slice(0, 4).map((category) => (
                  <li key={category.id}>
                    <Link to={`/category/${category.id}`}>{category.name}</Link>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <p className="shop-footer-column-title">{t('Follow Us')}</p>
              <ul>
                <li>
                  <a href="https://instagram.com" target="_blank" rel="noreferrer">
                    Instagram
                  </a>
                </li>
                <li>
                  <a href="https://facebook.com" target="_blank" rel="noreferrer">
                    Facebook
                  </a>
                </li>
                <li>
                  <a href="https://x.com" target="_blank" rel="noreferrer">
                    X / Twitter
                  </a>
                </li>
                <li>
                  <a href="https://pinterest.com" target="_blank" rel="noreferrer">
                    Pinterest
                  </a>
                </li>
              </ul>
            </div>
          </div>

          <div className="shop-footer-newsletter">
            <p className="shop-footer-title">{t('Subscribe to our newsletter and get 10% off your first order.')}</p>
            <p className="shop-muted" style={{ fontSize: 12, marginTop: 8 }}>
              {t('Stay up to date with new collections, offers and stories from EzCommerce.')}
            </p>
            <form className="shop-footer-input" onSubmit={subscribe}>
              <input
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder="email@example.com"
                aria-label={t('Email address')}
                required
              />
              <button type="submit">{t('Subscribe')}</button>
            </form>
            {subscribed && (
              <p style={{ fontSize: 12, color: '#047857', marginTop: 8 }}>{t('Thanks for subscribing!')}</p>
            )}
          </div>
        </div>
        <div className="shop-footer-bottom">
          <div className="shop-container" style={{ display: 'flex', justifyContent: 'space-between', flexWrap: 'wrap', gap: 8 }}>
            <p style={{ margin: 0 }}>{t('© {year} EzCommerce. All rights reserved.', { year: new Date().getFullYear() })}</p>
            <p style={{ margin: 0 }}>{t('Secure payments · Privacy policy · Terms & conditions')}</p>
          </div>
        </div>
      </footer>
    </div>
  );
}
