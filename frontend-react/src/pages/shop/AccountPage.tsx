import { useEffect, useMemo, useState } from 'react';
import { Link, Navigate, useLocation, useNavigate } from 'react-router-dom';
import { Modal } from '@mantine/core';
import {
  IconChevronRight,
  IconHeart,
  IconHome,
  IconLogout,
  IconMapPin,
  IconPackage,
  IconShoppingBag,
  IconTrash,
  IconUser,
  IconPlus,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { useAuth, shopQuery } from '../../store';
import { formatDate, initials, mediaUrl, statusColor } from '../../lib/format';
import { addToCart } from '../../lib/cart';
import Stars from '../../components/shop/Stars';

const NAV = [
  { path: '/account', label: 'My profile', icon: <IconUser size={16} /> },
  { path: '/account/orders', label: 'My orders', icon: <IconPackage size={16} /> },
  { path: '/account/addresses', label: 'Address book', icon: <IconMapPin size={16} /> },
  { path: '/account/wishlist', label: 'My wishlist', icon: <IconHeart size={16} /> },
];

export default function AccountPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const loggedIn = useAuth((state) => state.loggedIn);
  const customer = useAuth((state) => state.customer);
  const restore = useAuth((state) => state.restore);
  const active = NAV.find((item) => item.path === location.pathname)?.path ?? '/account';
  const [message, setMessage] = useState('');

  useEffect(() => {
    if (loggedIn && !customer) restore();
  }, [loggedIn, customer, restore]);

  useEffect(() => {
    setMessage('');
  }, [location.pathname]);

  if (!loggedIn) {
    return <Navigate to="/login?redirect=/account" replace />;
  }

  return (
    <div className="shop-container" style={{ paddingTop: 32 }}>
      <div className="shop-breadcrumbs">
        <Link to="/">Home</Link>
        <span>/</span>
        <span>My account</span>
      </div>

      <div className="listing-head" style={{ marginBottom: 24 }}>
        <div>
          <span className="listing-eyebrow">My account</span>
          <h1 className="shop-heading" style={{ fontSize: 36 }}>
            Hello, {customer?.first_name ?? 'there'}
          </h1>
        </div>
      </div>

      <div className="account-layout">
        <aside>
          <div className="account-profile-card">
            <span className="account-avatar">{initials(`${customer?.first_name ?? ''} ${customer?.last_name ?? ''}`)}</span>
            <div>
              <strong>
                {customer?.first_name} {customer?.last_name}
              </strong>
              <span>{customer?.email}</span>
            </div>
          </div>
          <nav className="account-nav">
            {NAV.map((item) => (
              <Link key={item.path} to={item.path} data-active={active === item.path}>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 8 }}>
                  {item.icon} {item.label}
                </span>
                <IconChevronRight size={14} />
              </Link>
            ))}
            <button
              type="button"
              onClick={async () => {
                await useAuth.getState().logout();
                navigate('/');
              }}
            >
              <span style={{ display: 'inline-flex', alignItems: 'center', gap: 8 }}>
                <IconLogout size={16} /> Sign out
              </span>
            </button>
          </nav>
        </aside>

        <div>
          {active === '/account' && <ProfilePanel onSaved={setMessage} />}
          {active === '/account/orders' && <OrdersPanel />}
          {active === '/account/addresses' && <AddressesPanel />}
          {active === '/account/wishlist' && <WishlistPanel />}
          {message && <p className="form-success">{message}</p>}
        </div>
      </div>
    </div>
  );
}

function ProfilePanel({ onSaved }: { onSaved: (message: string) => void }) {
  const customer = useAuth((state) => state.customer);
  const restore = useAuth((state) => state.restore);
  const [form, setForm] = useState({
    first_name: customer?.first_name ?? '',
    last_name: customer?.last_name ?? '',
    phone: (customer as any)?.phone ?? '',
    gender: (customer as any)?.gender ?? '',
    date_of_birth: (customer as any)?.date_of_birth ?? '',
    current_password: '',
    new_password: '',
    confirm_password: '',
  });
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (customer) {
      setForm((current) => ({
        ...current,
        first_name: customer.first_name ?? '',
        last_name: customer.last_name ?? '',
        phone: (customer as any).phone ?? '',
        gender: (customer as any).gender ?? '',
        date_of_birth: (customer as any).date_of_birth ?? '',
      }));
    }
  }, [customer]);

  const set = (field: string, value: string) => setForm((current) => ({ ...current, [field]: value }));

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    setBusy(true);
    try {
      await api.put('customer/profile', {
        first_name: form.first_name,
        last_name: form.last_name,
        phone: form.phone,
        gender: form.gender,
        date_of_birth: form.date_of_birth,
        current_password: form.current_password,
        new_password: form.new_password,
      });
      await restore();
      onSaved('Your profile has been updated.');
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to update your profile.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="account-card">
      <h2 className="account-card-title">Profile information</h2>
      <form onSubmit={submit}>
        <div className="checkout-form-grid">
          <div>
            <label className="field-label">First name</label>
            <input className="form-input" value={form.first_name} onChange={(event) => set('first_name', event.target.value)} required />
          </div>
          <div>
            <label className="field-label">Last name</label>
            <input className="form-input" value={form.last_name} onChange={(event) => set('last_name', event.target.value)} required />
          </div>
          <div>
            <label className="field-label">Email</label>
            <input className="form-input" value={customer?.email ?? ''} readOnly />
          </div>
          <div>
            <label className="field-label">Phone</label>
            <input className="form-input" value={form.phone} onChange={(event) => set('phone', event.target.value)} />
          </div>
          <div>
            <label className="field-label">Gender</label>
            <select className="form-input" value={form.gender} onChange={(event) => set('gender', event.target.value)}>
              <option value="">Select</option>
              <option value="Male">Male</option>
              <option value="Female">Female</option>
              <option value="Other">Other</option>
            </select>
          </div>
          <div>
            <label className="field-label">Date of birth</label>
            <input type="date" className="form-input" value={form.date_of_birth ?? ''} onChange={(event) => set('date_of_birth', event.target.value)} />
          </div>
        </div>

        <h3 className="account-subtitle">Change password</h3>
        <div className="checkout-form-grid">
          <div className="span-2">
            <label className="field-label">Current password</label>
            <input type="password" className="form-input" value={form.current_password} onChange={(event) => set('current_password', event.target.value)} />
          </div>
          <div>
            <label className="field-label">New password</label>
            <input type="password" className="form-input" value={form.new_password} onChange={(event) => set('new_password', event.target.value)} />
          </div>
          <div>
            <label className="field-label">Confirm new password</label>
            <input type="password" className="form-input" value={form.confirm_password} onChange={(event) => set('confirm_password', event.target.value)} />
          </div>
        </div>

        {error && <p className="form-error">{error}</p>}

        <button type="submit" className="shop-primary-button" style={{ marginTop: 18 }} disabled={busy}>
          {busy ? 'Saving…' : 'Save changes'}
        </button>
      </form>
    </div>
  );
}

function OrdersPanel() {
  const [orders, setOrders] = useState<any[]>([]);
  const [meta, setMeta] = useState<any>(null);
  const [detail, setDetail] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  const load = () => {
    setLoading(true);
    api
      .get(`customer/orders${shopQuery()}`)
      .then((data) => {
        setOrders(data?.data ?? data ?? []);
        setMeta(data?.meta ?? null);
      })
      .catch(() => setOrders([]))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  const openDetail = async (id: number) => {
    try {
      const data = await api.get(`customer/orders/${id}${shopQuery()}`);
      setDetail(data);
    } catch {
      setDetail(null);
    }
  };

  const cancel = async (id: number) => {
    try {
      await api.post(`customer/orders/${id}/cancel`);
      setDetail(null);
      load();
    } catch {
      /* ignore */
    }
  };

  return (
    <div className="account-card">
      <h2 className="account-card-title">My orders</h2>

      {loading ? (
        <div className="shimmer-line" style={{ height: 80 }} />
      ) : orders.length === 0 ? (
        <div className="account-empty">
          <IconPackage size={34} />
          <h3>No orders yet</h3>
          <p>When you place an order it will appear here.</p>
          <Link to="/search" className="shop-primary-button shop-button-sm">
            Start shopping
          </Link>
        </div>
      ) : (
        <div className="orders-list">
          {orders.map((order) => (
            <div className="order-row" key={order.id}>
              <div>
                <strong>#{order.increment_id}</strong>
                <span className="shop-muted">{formatDate(order.created_at)}</span>
              </div>
              <div>
                <span className="shop-muted">Total</span>
                <strong>{order.formatted_grand_total ?? `€${Number(order.grand_total ?? 0).toFixed(2)}`}</strong>
              </div>
              <div>
                <span className="shop-muted">Status</span>
                <span className={`status-badge ${statusColor(order.status)}`} style={{ textTransform: 'capitalize' }}>
                  {order.status}
                </span>
              </div>
              <button type="button" className="shop-secondary-button shop-button-sm" onClick={() => openDetail(order.id)}>
                View order
              </button>
            </div>
          ))}
        </div>
      )}

      {meta && meta.last_page > 1 && (
        <p className="shop-muted" style={{ fontSize: 12 }}>
          Page {meta.current_page} of {meta.last_page} · {meta.total} orders
        </p>
      )}

      <Modal opened={!!detail} onClose={() => setDetail(null)} size="lg" title={`Order #${detail?.increment_id ?? ''}`} centered>
        {detail && (
          <div>
            <div className="order-detail-meta">
              <div>
                <span>Placed on</span>
                <strong>{formatDate(detail.created_at, true)}</strong>
              </div>
              <div>
                <span>Status</span>
                <span className={`status-badge ${statusColor(detail.status)}`}>{detail.status}</span>
              </div>
              <div>
                <span>Total</span>
                <strong>€{Number(detail.grand_total ?? 0).toFixed(2)}</strong>
              </div>
            </div>

            <table className="cart-table" style={{ marginTop: 16 }}>
              <thead>
                <tr>
                  <th>Product</th>
                  <th>Qty</th>
                  <th style={{ textAlign: 'right' }}>Total</th>
                </tr>
              </thead>
              <tbody>
                {(detail.items ?? []).map((item: any) => (
                  <tr key={item.id}>
                    <td>
                      {item.name}
                      <br />
                      <span className="shop-muted" style={{ fontSize: 11 }}>
                        {item.sku}
                      </span>
                    </td>
                    <td>{item.qty_ordered}</td>
                    <td style={{ textAlign: 'right' }}>€{Number(item.total ?? 0).toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
            </table>

            {detail.billing_address && (
              <div style={{ marginTop: 16 }}>
                <h3 className="account-subtitle">Shipping address</h3>
                <p className="shop-muted" style={{ fontSize: 13, lineHeight: 1.7 }}>
                  {detail.billing_address.first_name} {detail.billing_address.last_name}
                  <br />
                  {detail.billing_address.address1}
                  <br />
                  {detail.billing_address.city}, {detail.billing_address.state} {detail.billing_address.postcode}
                  <br />
                  {detail.billing_address.country}
                </p>
              </div>
            )}

            {detail.can_cancel && (
              <button type="button" className="shop-secondary-button" style={{ marginTop: 16 }} onClick={() => cancel(detail.id)}>
                Cancel order
              </button>
            )}
          </div>
        )}
      </Modal>
    </div>
  );
}

function AddressesPanel() {
  const [addresses, setAddresses] = useState<any[]>([]);
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<any>(null);
  const [form, setForm] = useState({
    first_name: '',
    last_name: '',
    company_name: '',
    address: '',
    country: 'ES',
    state: '',
    city: '',
    postcode: '',
    phone: '',
    email: '',
  });
  const [countries, setCountries] = useState<{ code: string; name: string }[]>([]);
  const [error, setError] = useState('');
  const customer = useAuth((state) => state.customer);

  const load = () => {
    api
      .get('customer/addresses')
      .then((data) => setAddresses(Array.isArray(data) ? data : data?.data ?? []))
      .catch(() => setAddresses([]));
  };

  useEffect(() => {
    load();
    api
      .get('core/countries')
      .then((data) => {
        const unique = new Map<string, { code: string; name: string }>();
        (Array.isArray(data) ? data : []).forEach((country: any) => {
          if (!unique.has(country.code)) unique.set(country.code, { code: country.code, name: country.name });
        });
        setCountries(Array.from(unique.values()).sort((a, b) => a.name.localeCompare(b.name)));
      })
      .catch(() => setCountries([]));
  }, []);

  const startCreate = () => {
    setEditing(null);
    setError('');
    setForm({
      first_name: customer?.first_name ?? '',
      last_name: customer?.last_name ?? '',
      company_name: '',
      address: '',
      country: 'ES',
      state: '',
      city: '',
      postcode: '',
      phone: '',
      email: customer?.email ?? '',
    });
    setOpen(true);
  };

  const startEdit = (address: any) => {
    setEditing(address);
    setError('');
    setForm({
      first_name: address.first_name ?? '',
      last_name: address.last_name ?? '',
      company_name: address.company_name ?? '',
      address: address.address1 ?? '',
      country: address.country ?? 'ES',
      state: address.state ?? '',
      city: address.city ?? '',
      postcode: address.postcode ?? '',
      phone: address.phone ?? '',
      email: address.email ?? '',
    });
    setOpen(true);
  };

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    try {
      if (editing) await api.put(`customer/addresses/edit/${editing.id}`, form);
      else await api.post('customer/addresses', form);
      setOpen(false);
      load();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to save the address.');
    }
  };

  const remove = async (id: number) => {
    try {
      await api.delete(`customer/addresses/${id}`);
      load();
    } catch {
      /* ignore */
    }
  };

  const makeDefault = async (id: number) => {
    try {
      await api.patch(`customer/addresses/default/${id}`);
      load();
    } catch {
      /* ignore */
    }
  };

  return (
    <div className="account-card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <h2 className="account-card-title" style={{ margin: 0 }}>
          Address book
        </h2>
        <button type="button" className="shop-primary-button shop-button-sm" onClick={startCreate}>
          <IconPlus size={14} /> Add address
        </button>
      </div>

      {addresses.length === 0 ? (
        <div className="account-empty">
          <IconMapPin size={34} />
          <h3>No saved addresses</h3>
          <p>Save an address to make future checkouts faster.</p>
        </div>
      ) : (
        <div className="address-grid">
          {addresses.map((address) => (
            <div className="address-card" key={address.id}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <strong>
                  {address.first_name} {address.last_name}
                </strong>
                {address.default_address ? <span className="status-badge green">Default</span> : null}
              </div>
              <p className="shop-muted">
                {address.company_name && (
                  <>
                    {address.company_name}
                    <br />
                  </>
                )}
                {address.address1}
                <br />
                {address.city}, {address.state} {address.postcode}
                <br />
                {address.country}
                <br />
                {address.phone}
              </p>
              <div className="address-actions">
                <button type="button" onClick={() => startEdit(address)}>
                  Edit
                </button>
                {!address.default_address && (
                  <button type="button" onClick={() => makeDefault(address.id)}>
                    Set default
                  </button>
                )}
                <button type="button" className="danger" onClick={() => remove(address.id)}>
                  Delete
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      <Modal opened={open} onClose={() => setOpen(false)} title={editing ? 'Edit address' : 'New address'} centered size="lg">
        <form onSubmit={submit}>
          <div className="checkout-form-grid">
            <div>
              <label className="field-label">First name</label>
              <input className="form-input" value={form.first_name} onChange={(event) => setForm({ ...form, first_name: event.target.value })} required />
            </div>
            <div>
              <label className="field-label">Last name</label>
              <input className="form-input" value={form.last_name} onChange={(event) => setForm({ ...form, last_name: event.target.value })} required />
            </div>
            <div className="span-2">
              <label className="field-label">Street address</label>
              <input className="form-input" value={form.address} onChange={(event) => setForm({ ...form, address: event.target.value })} required />
            </div>
            <div>
              <label className="field-label">City</label>
              <input className="form-input" value={form.city} onChange={(event) => setForm({ ...form, city: event.target.value })} required />
            </div>
            <div>
              <label className="field-label">Postcode</label>
              <input className="form-input" value={form.postcode} onChange={(event) => setForm({ ...form, postcode: event.target.value })} required />
            </div>
            <div>
              <label className="field-label">State</label>
              <input className="form-input" value={form.state} onChange={(event) => setForm({ ...form, state: event.target.value })} />
            </div>
            <div>
              <label className="field-label">Country</label>
              <select className="form-input" value={form.country} onChange={(event) => setForm({ ...form, country: event.target.value })}>
                {countries.map((country) => (
                  <option key={country.code} value={country.code}>
                    {country.name}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="field-label">Phone</label>
              <input className="form-input" value={form.phone} onChange={(event) => setForm({ ...form, phone: event.target.value })} />
            </div>
            <div>
              <label className="field-label">Email</label>
              <input className="form-input" value={form.email} onChange={(event) => setForm({ ...form, email: event.target.value })} />
            </div>
          </div>
          {error && <p className="form-error">{error}</p>}
          <button type="submit" className="shop-primary-button" style={{ marginTop: 16 }}>
            Save address
          </button>
        </form>
      </Modal>
    </div>
  );
}

function WishlistPanel() {
  const [items, setItems] = useState<any[]>([]);
  const [busy, setBusy] = useState<number | null>(null);

  const load = () => {
    api
      .get(`customer/wishlist${shopQuery()}`)
      .then((data) => setItems(data?.data ?? []))
      .catch(() => setItems([]));
  };

  useEffect(load, []);

  const moveToCart = async (id: number) => {
    setBusy(id);
    try {
      await api.post(`customer/wishlist/${id}/move-to-cart`);
      window.dispatchEvent(new Event('cart-updated'));
      load();
    } finally {
      setBusy(null);
    }
  };

  const remove = async (id: number) => {
    await api.delete(`customer/wishlist/${id}`);
    load();
  };

  return (
    <div className="account-card">
      <h2 className="account-card-title">My wishlist</h2>
      {items.length === 0 ? (
        <div className="account-empty">
          <IconHeart size={34} />
          <h3>Your wishlist is empty</h3>
          <p>Save products you love and come back to them later.</p>
          <Link to="/search" className="shop-primary-button shop-button-sm">
            Discover products
          </Link>
        </div>
      ) : (
        <div className="wishlist-grid">
          {items.map((item) => (
            <div className="wishlist-card" key={item.id}>
              <Link to={`/product/${item.url_key ?? item.id}`} className="wishlist-media">
                <img src={mediaUrl(item.base_image?.url)} alt={item.name} />
              </Link>
              <div style={{ flex: 1, minWidth: 0 }}>
                <Link to={`/product/${item.url_key ?? item.id}`} className="cart-item-title">
                  {item.name}
                </Link>
                <div style={{ marginTop: 4 }}>
                  <Stars value={item.ratings?.average ?? 0} size={12} />
                </div>
                <div className="product-price-row" style={{ marginTop: 6, fontSize: 16 }}>
                  {item.formatted_price}
                </div>
                <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
                  <button
                    type="button"
                    className="shop-primary-button shop-button-sm"
                    disabled={busy === item.wishlist_id}
                    onClick={() => moveToCart(item.wishlist_id)}
                  >
                    <IconShoppingBag size={14} /> Add to cart
                  </button>
                  <button type="button" className="cart-remove" onClick={() => remove(item.wishlist_id)} aria-label="Remove">
                    <IconTrash size={16} />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
