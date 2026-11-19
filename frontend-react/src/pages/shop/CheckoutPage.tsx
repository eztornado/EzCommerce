import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import {
  IconArrowLeft,
  IconCheck,
  IconChevronRight,
  IconCreditCard,
  IconLock,
  IconTruck,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { useAuth, useCart, shopQuery } from '../../store';
import { mediaUrl, formattedMoney } from '../../lib/format';
import { notifyCart } from '../../lib/hooks';
import type { Cart } from '../../types';

interface AddressForm {
  first_name: string;
  last_name: string;
  email: string;
  company: string;
  address: string;
  country: string;
  state: string;
  city: string;
  postcode: string;
  phone: string;
}

const emptyAddress: AddressForm = {
  first_name: '',
  last_name: '',
  email: '',
  company: '',
  address: '',
  country: 'ES',
  state: '',
  city: '',
  postcode: '',
  phone: '',
};

function AddressFields({
  value,
  onChange,
  countries,
  prefix,
}: {
  value: AddressForm;
  onChange: (next: AddressForm) => void;
  countries: { code: string; name: string }[];
  prefix: string;
}) {
  const set = (field: keyof AddressForm, fieldValue: string) => onChange({ ...value, [field]: fieldValue });
  return (
    <div className="checkout-form-grid">
      <div>
        <label className="field-label" htmlFor={`${prefix}-first`}>
          First name *
        </label>
        <input id={`${prefix}-first`} className="form-input" value={value.first_name} onChange={(e) => set('first_name', e.target.value)} required />
      </div>
      <div>
        <label className="field-label" htmlFor={`${prefix}-last`}>
          Last name *
        </label>
        <input id={`${prefix}-last`} className="form-input" value={value.last_name} onChange={(e) => set('last_name', e.target.value)} required />
      </div>
      <div className="span-2">
        <label className="field-label" htmlFor={`${prefix}-email`}>
          Email *
        </label>
        <input id={`${prefix}-email`} type="email" className="form-input" value={value.email} onChange={(e) => set('email', e.target.value)} required />
      </div>
      <div className="span-2">
        <label className="field-label" htmlFor={`${prefix}-company`}>
          Company
        </label>
        <input id={`${prefix}-company`} className="form-input" value={value.company} onChange={(e) => set('company', e.target.value)} />
      </div>
      <div className="span-2">
        <label className="field-label" htmlFor={`${prefix}-address`}>
          Street address *
        </label>
        <input id={`${prefix}-address`} className="form-input" value={value.address} onChange={(e) => set('address', e.target.value)} required />
      </div>
      <div>
        <label className="field-label" htmlFor={`${prefix}-country`}>
          Country *
        </label>
        <select id={`${prefix}-country`} className="form-input" value={value.country} onChange={(e) => set('country', e.target.value)} required>
          {countries.map((country) => (
            <option value={country.code} key={country.code}>
              {country.name}
            </option>
          ))}
        </select>
      </div>
      <div>
        <label className="field-label" htmlFor={`${prefix}-state`}>
          State / Province *
        </label>
        <input id={`${prefix}-state`} className="form-input" value={value.state} onChange={(e) => set('state', e.target.value)} required />
      </div>
      <div>
        <label className="field-label" htmlFor={`${prefix}-city`}>
          City *
        </label>
        <input id={`${prefix}-city`} className="form-input" value={value.city} onChange={(e) => set('city', e.target.value)} required />
      </div>
      <div>
        <label className="field-label" htmlFor={`${prefix}-postcode`}>
          Postcode *
        </label>
        <input id={`${prefix}-postcode`} className="form-input" value={value.postcode} onChange={(e) => set('postcode', e.target.value)} required />
      </div>
      <div className="span-2">
        <label className="field-label" htmlFor={`${prefix}-phone`}>
          Phone
        </label>
        <input id={`${prefix}-phone`} className="form-input" value={value.phone} onChange={(e) => set('phone', e.target.value)} />
      </div>
    </div>
  );
}

export default function CheckoutPage() {
  const navigate = useNavigate();
  const customer = useAuth((state) => state.customer);
  const loggedIn = useAuth((state) => state.loggedIn);
  const refreshCart = useCart((state) => state.refresh);
  const [cart, setCart] = useState<Cart | null>(null);
  const [countries, setCountries] = useState<{ code: string; name: string }[]>([]);
  const [billing, setBilling] = useState<AddressForm>(emptyAddress);
  const [shipping, setShipping] = useState<AddressForm>(emptyAddress);
  const [differentShipping, setDifferentShipping] = useState(false);
  const [step, setStep] = useState(1);
  const [shippingMethods, setShippingMethods] = useState<any[]>([]);
  const [shippingMethod, setShippingMethod] = useState('');
  const [paymentMethods, setPaymentMethods] = useState<any[]>([]);
  const [paymentMethod, setPaymentMethod] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [coupon, setCoupon] = useState('');
  const [couponMessage, setCouponMessage] = useState('');

  useEffect(() => {
    api
      .get(`core/countries${shopQuery()}`)
      .then((data) => {
        const unique = new Map<string, { code: string; name: string }>();
        (Array.isArray(data) ? data : []).forEach((country: any) => {
          if (!unique.has(country.code)) unique.set(country.code, { code: country.code, name: country.name });
        });
        setCountries(Array.from(unique.values()).sort((a, b) => a.name.localeCompare(b.name)));
      })
      .catch(() => setCountries([{ code: 'ES', name: 'Spain' }]));

    api
      .get(`checkout/onepage/summary${shopQuery()}`)
      .then((data) => setCart(data))
      .catch(() => setCart(null));

    api
      .get(`checkout/onepage/payment-methods${shopQuery()}`)
      .then((data) => {
        const methods = data?.payment_methods ?? [];
        setPaymentMethods(methods);
        if (methods[0]) setPaymentMethod(methods[0].method);
      })
      .catch(() => setPaymentMethods([]));
  }, []);

  useEffect(() => {
    if (customer) {
      setBilling((current) => ({
        ...current,
        first_name: customer.first_name ?? current.first_name,
        last_name: customer.last_name ?? current.last_name,
        email: customer.email ?? current.email,
      }));
    }
  }, [customer]);

  useEffect(() => {
    if (loggedIn) {
      api
        .get('customer/addresses')
        .then((data) => {
          const list = Array.isArray(data) ? data : data?.data ?? [];
          if (list[0]) {
            const address = list[0];
            const mapped: AddressForm = {
              first_name: address.first_name ?? '',
              last_name: address.last_name ?? '',
              email: address.email ?? customer?.email ?? '',
              company: address.company_name ?? '',
              address: address.address1 ?? '',
              country: address.country ?? 'ES',
              state: address.state ?? '',
              city: address.city ?? '',
              postcode: address.postcode ?? '',
              phone: address.phone ?? '',
            };
            setBilling(mapped);
            setShipping(mapped);
          }
        })
        .catch(() => undefined);
    }
  }, [loggedIn]);

  const totalItems = useMemo(() => cart?.items_qty ?? 0, [cart]);

  const saveAddresses = async () => {
    setError('');
    setBusy(true);
    try {
      await api.post(`checkout/onepage/addresses${shopQuery()}`, {
        billing,
        shipping: differentShipping ? shipping : null,
      });
      const methods = await api.get(`checkout/onepage/shipping-methods${shopQuery()}`);
      const rates = methods?.shippingMethods ?? [];
      setShippingMethods(rates);
      if (rates[0]) setShippingMethod(rates[0].method);
      const summary = await api.get(`checkout/onepage/summary${shopQuery()}`);
      setCart(summary);
      setStep(2);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Please review the address fields.');
    } finally {
      setBusy(false);
    }
  };

  const saveShipping = async () => {
    setError('');
    if (!shippingMethod) {
      setError('Please choose a shipping method.');
      return;
    }
    setBusy(true);
    try {
      const data = await api.post(`checkout/onepage/shipping-methods${shopQuery()}`, {
        shipping_method: shippingMethod,
      });
      if (data?.payment_methods) {
        setPaymentMethods(data.payment_methods);
        if (data.payment_methods[0]) setPaymentMethod(data.payment_methods[0].method);
      }
      const summary = await api.get(`checkout/onepage/summary${shopQuery()}`);
      setCart(summary);
      setStep(3);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to save the shipping method.');
    } finally {
      setBusy(false);
    }
  };

  const placeOrder = async () => {
    setError('');
    if (!paymentMethod) {
      setError('Please choose a payment method.');
      return;
    }
    setBusy(true);
    try {
      await api.post(`checkout/onepage/payment-methods${shopQuery()}`, {
        payment_method: paymentMethod,
      });
      const data = await api.post(`checkout/onepage/orders${shopQuery()}`, {});
      localStorage.removeItem('cart_token');
      await refreshCart();
      notifyCart();
      const orderId = data?.order_id ?? data?.order?.id;
      navigate(`/checkout/success/${orderId}`);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to place the order.');
      setBusy(false);
    }
  };

  const applyCoupon = async (event: React.FormEvent) => {
    event.preventDefault();
    setCouponMessage('');
    try {
      await api.post(`checkout/cart/coupon${shopQuery()}`, { code: coupon });
      const summary = await api.get(`checkout/onepage/summary${shopQuery()}`);
      setCart(summary);
      setCoupon('');
      setCouponMessage('Coupon applied.');
    } catch (requestError: any) {
      setCouponMessage(requestError?.message ?? 'Invalid coupon.');
    }
  };

  if (!cart || !cart.items?.length) {
    return (
      <div className="shop-container listing-empty" style={{ paddingTop: 70, paddingBottom: 100 }}>
        <h2 className="shop-heading">Your cart is empty</h2>
        <p className="shop-muted">Add a few products before proceeding to checkout.</p>
        <Link to="/search" className="shop-primary-button">
          Continue shopping
        </Link>
      </div>
    );
  }

  return (
    <div className="shop-container">
      <div className="shop-breadcrumbs">
        <Link to="/">Home</Link>
        <span>/</span>
        <Link to="/cart">Cart</Link>
        <span>/</span>
        <span>Checkout</span>
      </div>

      <div className="listing-head" style={{ marginBottom: 18 }}>
        <h1 className="shop-heading" style={{ fontSize: 36 }}>
          Checkout
        </h1>
        <Link to="/cart" className="shop-link" style={{ fontSize: 14 }}>
          <IconArrowLeft size={14} /> Back to cart
        </Link>
      </div>

      <div className="checkout-steps">
        <span className="checkout-step" data-active={step === 1} data-done={step > 1}>
          {step > 1 ? <IconCheck size={14} /> : '1'} Address
        </span>
        <span className="checkout-step" data-active={step === 2} data-done={step > 2}>
          {step > 2 ? <IconCheck size={14} /> : '2'} Shipping
        </span>
        <span className="checkout-step" data-active={step === 3}>
          3 Payment
        </span>
      </div>

      <div className="checkout-layout">
        <div>
          <section className="checkout-panel">
            <div className="checkout-panel-head">
              <h2>
                <span className="checkout-number">1</span> Billing address
              </h2>
              {step > 1 && (
                <button type="button" className="shop-link" onClick={() => setStep(1)}>
                  Edit
                </button>
              )}
            </div>
            {step === 1 ? (
              <form
                onSubmit={(event) => {
                  event.preventDefault();
                  saveAddresses();
                }}
              >
                <AddressFields value={billing} onChange={setBilling} countries={countries.length ? countries : [{ code: 'ES', name: 'Spain' }]} prefix="billing" />

                <label className="checkout-check">
                  <input
                    type="checkbox"
                    checked={differentShipping}
                    onChange={(event) => {
                      setDifferentShipping(event.target.checked);
                      if (event.target.checked && !shipping.first_name) setShipping({ ...billing });
                    }}
                  />
                  Ship to a different address
                </label>

                {differentShipping && (
                  <div style={{ marginTop: 18 }}>
                    <h3 className="checkout-subtitle">Shipping address</h3>
                    <AddressFields value={shipping} onChange={setShipping} countries={countries.length ? countries : [{ code: 'ES', name: 'Spain' }]} prefix="shipping" />
                  </div>
                )}

                {!loggedIn && (
                  <p className="shop-muted" style={{ fontSize: 12, marginTop: 14 }}>
                    Already have an account?{' '}
                    <Link to="/login" className="shop-link">
                      Sign in
                    </Link>{' '}
                    for a faster checkout.
                  </p>
                )}

                {error && <p className="form-error">{error}</p>}

                <button type="submit" className="shop-primary-button" style={{ marginTop: 18 }} disabled={busy}>
                  {busy ? 'Saving…' : 'Continue to shipping'} <IconChevronRight size={16} />
                </button>
              </form>
            ) : (
              <div className="checkout-summary-address">
                <p>
                  <strong>
                    {billing.first_name} {billing.last_name}
                  </strong>
                  <br />
                  {billing.address}
                  <br />
                  {billing.city}, {billing.state} {billing.postcode}
                  <br />
                  {countries.find((country) => country.code === billing.country)?.name ?? billing.country}
                  <br />
                  {billing.email}
                  {billing.phone ? ` · ${billing.phone}` : ''}
                </p>
              </div>
            )}
          </section>

          {step >= 2 && (
            <section className="checkout-panel">
              <div className="checkout-panel-head">
                <h2>
                  <span className="checkout-number">2</span> Shipping method
                </h2>
                {step > 2 && (
                  <button type="button" className="shop-link" onClick={() => setStep(2)}>
                    Edit
                  </button>
                )}
              </div>

              {step === 2 ? (
                <div>
                  {shippingMethods.length === 0 ? (
                    <p className="shop-muted">No shipping methods are available for this address.</p>
                  ) : (
                    <div className="choice-list">
                      {shippingMethods.map((method) => (
                        <label className="choice-card" data-active={shippingMethod === method.method} key={method.method}>
                          <input
                            type="radio"
                            name="shipping"
                            value={method.method}
                            checked={shippingMethod === method.method}
                            onChange={() => setShippingMethod(method.method)}
                          />
                          <span className="choice-icon">
                            <IconTruck size={19} />
                          </span>
                          <span className="choice-body">
                            <strong>{method.method_title}</strong>
                            <span>{method.method_description}</span>
                          </span>
                          <span className="choice-price">{Number(method.price) === 0 ? 'Free' : `€${Number(method.price).toFixed(2)}`}</span>
                        </label>
                      ))}
                    </div>
                  )}
                  {error && <p className="form-error">{error}</p>}
                  <button type="button" className="shop-primary-button" style={{ marginTop: 18 }} onClick={saveShipping} disabled={busy || !shippingMethods.length}>
                    {busy ? 'Saving…' : 'Continue to payment'} <IconChevronRight size={16} />
                  </button>
                </div>
              ) : (
                <p className="shop-muted" style={{ fontSize: 14 }}>
                  {shippingMethods.find((method) => method.method === shippingMethod)?.method_title ?? 'Selected shipping method'}
                </p>
              )}
            </section>
          )}

          {step >= 3 && (
            <section className="checkout-panel">
              <div className="checkout-panel-head">
                <h2>
                  <span className="checkout-number">3</span> Payment method
                </h2>
              </div>

              <div className="choice-list">
                {paymentMethods.map((method) => (
                  <label className="choice-card" data-active={paymentMethod === method.method} key={method.method}>
                    <input
                      type="radio"
                      name="payment"
                      value={method.method}
                      checked={paymentMethod === method.method}
                      onChange={() => setPaymentMethod(method.method)}
                    />
                    <span className="choice-icon">
                      <IconCreditCard size={19} />
                    </span>
                    <span className="choice-body">
                      <strong>{method.method_title}</strong>
                      <span>{method.description}</span>
                    </span>
                  </label>
                ))}
              </div>

              {error && <p className="form-error">{error}</p>}

              <button type="button" className="shop-primary-button" style={{ marginTop: 18, width: '100%' }} onClick={placeOrder} disabled={busy}>
                {busy ? 'Placing order…' : `Place order · ${formattedMoney(cart.formatted_grand_total, cart.grand_total)}`} <IconLock size={15} />
              </button>
              <p className="shop-muted" style={{ fontSize: 11, textAlign: 'center', marginTop: 10 }}>
                By placing the order you agree to our terms and conditions.
              </p>
            </section>
          )}
        </div>

        <aside className="cart-summary checkout-summary">
          <h2 className="cart-summary-title">Your order</h2>
          <p className="shop-muted" style={{ fontSize: 12, marginTop: -8 }}>
            {totalItems} {totalItems === 1 ? 'item' : 'items'} in cart
          </p>

          <div className="checkout-items">
            {cart.items.map((item) => (
              <div className="checkout-item" key={item.id}>
                <div className="checkout-item-media">
                  <img src={mediaUrl(item.base_image?.url)} alt={item.name} />
                  <span className="checkout-item-qty">{item.quantity}</span>
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <p className="checkout-item-name">{item.name}</p>
                  <p className="shop-muted" style={{ fontSize: 11 }}>
                    {item.sku}
                  </p>
                </div>
                <span style={{ fontSize: 13, fontWeight: 600 }}>{formattedMoney(item.formatted_total, item.total)}</span>
              </div>
            ))}
          </div>

          <div className="cart-summary-row" style={{ marginTop: 8 }}>
            <span>Subtotal</span>
            <span>{formattedMoney(cart.formatted_sub_total, cart.sub_total)}</span>
          </div>
          {Number((cart as any).discount_amount ?? 0) > 0 && (
            <div className="cart-summary-row" style={{ color: '#047857' }}>
              <span>Discount</span>
              <span>−{formattedMoney(cart.formatted_discount_amount, (cart as any).discount_amount)}</span>
            </div>
          )}
          <div className="cart-summary-row">
            <span>Shipping</span>
            <span>{formattedMoney(cart.formatted_shipping_amount, cart.shipping_amount)}</span>
          </div>
          <div className="cart-summary-row">
            <span>Tax</span>
            <span>{formattedMoney((cart as any).formatted_tax_total ?? (cart as any).formatted_tax_amount, (cart as any).tax_total)}</span>
          </div>
          <div className="cart-summary-row total">
            <span>Grand total</span>
            <span>{formattedMoney(cart.formatted_grand_total, cart.grand_total)}</span>
          </div>

          {!cart.coupon_code && (
            <form onSubmit={applyCoupon} style={{ marginTop: 8 }}>
              <div style={{ display: 'flex', gap: 8 }}>
                <input className="form-input" value={coupon} onChange={(event) => setCoupon(event.target.value)} placeholder="Coupon code" />
                <button type="submit" className="shop-secondary-button shop-button-sm">
                  Apply
                </button>
              </div>
              {couponMessage && <p className="shop-muted" style={{ fontSize: 12, marginTop: 6 }}>{couponMessage}</p>}
            </form>
          )}
        </aside>
      </div>
    </div>
  );
}
