import { useEffect, useState } from 'react';
import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { Menu } from '@mantine/core';
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
import type { Category } from '../../types';

export default function ShopLayout() {
  const { context, changeLocale, changeCurrency } = useShop();
  const cartCount = useCart((state) => state.count);
  const refreshCart = useCart((state) => state.refresh);
  const loggedIn = useAuth((state) => state.loggedIn);
  const customer = useAuth((state) => state.customer);
  const reload = useShopReload();
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

  const activeCurrency = context?.currencies.find((currency) => currency.code === context?.currency);
  const activeLocale = context?.locales.find((locale) => locale.code === context?.locale);

  return (
    <div className="shop-shell">
      <div className="shop-topbar">
        <div className="shop-container shop-topbar-inner">
          <Menu shadow="md" width={200} position="bottom-start">
            <Menu.Target>
              <button type="button" className="topbar-select">
                {activeCurrency ? `${activeCurrency.symbol} ${activeCurrency.code}` : 'EUR €'} <IconChevronDown size={14} />
              </button>
            </Menu.Target>
            <Menu.Dropdown>
              {context?.currencies.map((currency) => (
                <Menu.Item key={currency.code} onClick={() => changeCurrency(currency.code)}>
                  {currency.symbol} {currency.code} — {currency.name}
                </Menu.Item>
              ))}
            </Menu.Dropdown>
          </Menu>

          <p className="topbar-offer">
            {context?.header_offer ?? 'Free shipping on all orders over €100'}
            <Link to="/products"> Shop now</Link>
          </p>

          <Menu shadow="md" width={180} position="bottom-end">
            <Menu.Target>
              <button type="button" className="topbar-select">
                {activeLocale?.name ?? 'English'} <IconChevronDown size={14} />
              </button>
            </Menu.Target>
            <Menu.Dropdown>
              {context?.locales.map((locale) => (
                <Menu.Item key={locale.code} onClick={() => changeLocale(locale.code)}>
                  {locale.name}
                </Menu.Item>
              ))}
            </Menu.Dropdown>
          </Menu>
        </div>
      </div>

      <header className="shop-header">
        <div className="shop-container shop-header-main">
          <button type="button" className="mobile-menu-toggle" onClick={() => setMobileOpen((open) => !open)} aria-label="Menu">
            {mobileOpen ? <IconX size={22} /> : <IconMenu2 size={22} />}
          </button>

          <Link to="/" className="shop-logo">
            {context?.channel?.logo ? (
              <img src={mediaUrl(context.channel.logo)} alt={context.channel.name} style={{ height: 30 }} />
            ) : (
              <>
                Ez<span>Commerce</span>
              </>
            )}
          </Link>

          <nav className="shop-nav" aria-label="Main navigation">
            <NavLink to="/" end className="shop-nav-link" data-active={location.pathname === '/'}>
              Home
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
              New Arrivals
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
                placeholder="Search products..."
                aria-label="Search products"
              />
            </form>

            <div className="shop-icon-row">
              <Link to="/account/wishlist" className="shop-icon-button" aria-label="Wishlist">
                <IconHeart size={21} />
              </Link>

              <Menu shadow="md" width={230} position="bottom-end">
                <Menu.Target>
                  <button type="button" className="shop-icon-button" aria-label="Account">
                    <IconUser size={21} />
                  </button>
                </Menu.Target>
                <Menu.Dropdown>
                  {loggedIn ? (
                    <>
                      <Menu.Label>Welcome, {customer?.first_name ?? 'Customer'}</Menu.Label>
                      <Menu.Item component={Link} to="/account">
                        My profile
                      </Menu.Item>
                      <Menu.Item component={Link} to="/account/orders">
                        My orders
                      </Menu.Item>
                      <Menu.Item component={Link} to="/account/wishlist">
                        Wishlist
                      </Menu.Item>
                      <Menu.Item
                        onClick={async () => {
                          await useAuth.getState().logout();
                          navigate('/');
                        }}
                      >
                        Sign out
                      </Menu.Item>
                    </>
                  ) : (
                    <>
                      <Menu.Label>Welcome to EzCommerce</Menu.Label>
                      <Menu.Item component={Link} to="/login">
                        Sign in
                      </Menu.Item>
                      <Menu.Item component={Link} to="/register">
                        Create an account
                      </Menu.Item>
                    </>
                  )}
                </Menu.Dropdown>
              </Menu>

              <Link to="/cart" className="shop-icon-button" aria-label="Shopping cart">
                <IconShoppingBag size={21} />
                {cartCount > 0 && <span className="shop-cart-count">{cartCount}</span>}
              </Link>
            </div>
          </div>
        </div>

        {mobileOpen && (
          <div className="shop-mobile-nav">
            <Link to="/">Home</Link>
            {categories.map((category) => (
              <Link key={category.id} to={`/category/${category.id}`}>
                {category.name}
              </Link>
            ))}
            <Link to="/search">Search</Link>
            <Link to="/account">My account</Link>
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
              <p className="shop-footer-column-title">Quick Links</p>
              <ul>
                <li>
                  <Link to="/">Home</Link>
                </li>
                <li>
                  <Link to="/search?sort=newest">New arrivals</Link>
                </li>
                <li>
                  <Link to="/search">All products</Link>
                </li>
                <li>
                  <Link to="/cart">Shopping cart</Link>
                </li>
              </ul>
            </div>
            <div>
              <p className="shop-footer-column-title">Customer Service</p>
              <ul>
                <li>
                  <Link to="/account">My account</Link>
                </li>
                <li>
                  <Link to="/account/orders">Order tracking</Link>
                </li>
                <li>
                  <Link to="/account/wishlist">Wishlist</Link>
                </li>
                <li>
                  <Link to="/login">Sign in</Link>
                </li>
              </ul>
            </div>
            <div>
              <p className="shop-footer-column-title">Categories</p>
              <ul>
                {categories.slice(0, 4).map((category) => (
                  <li key={category.id}>
                    <Link to={`/category/${category.id}`}>{category.name}</Link>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <p className="shop-footer-column-title">Follow Us</p>
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
            <p className="shop-footer-title">
              Subscribe to our newsletter and get <em>10% off</em> your first order.
            </p>
            <p className="shop-muted" style={{ fontSize: 12, marginTop: 8 }}>
              Stay up to date with new collections, offers and stories from EzCommerce.
            </p>
            <form className="shop-footer-input" onSubmit={subscribe}>
              <input
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder="email@example.com"
                aria-label="Email address"
                required
              />
              <button type="submit">Subscribe</button>
            </form>
            {subscribed && (
              <p style={{ fontSize: 12, color: '#047857', marginTop: 8 }}>Thanks for subscribing!</p>
            )}
          </div>
        </div>
        <div className="shop-footer-bottom">
          <div className="shop-container" style={{ display: 'flex', justifyContent: 'space-between', flexWrap: 'wrap', gap: 8 }}>
            <p style={{ margin: 0 }}>© {new Date().getFullYear()} EzCommerce. All rights reserved.</p>
            <p style={{ margin: 0 }}>Secure payments · Privacy policy · Terms &amp; conditions</p>
          </div>
        </div>
      </footer>
    </div>
  );
}
