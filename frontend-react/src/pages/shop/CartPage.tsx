import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { IconArrowRight, IconShoppingBag, IconTag, IconTrash } from '@tabler/icons-react';
import { api } from '../../api/client';
import { useCart, shopQuery } from '../../store';
import { mediaUrl, formattedMoney } from '../../lib/format';
import { updateCartQty, removeCartItem } from '../../lib/cart';
import { useShopReload } from '../../lib/hooks';
import ProductCard from '../../components/shop/ProductCard';
import type { ProductCard as ProductCardModel } from '../../types';

export default function CartPage() {
  const reload = useShopReload();
  const cart = useCart((state) => state.cart);
  const refresh = useCart((state) => state.refresh);
  const [busy, setBusy] = useState<number | null>(null);
  const [coupon, setCoupon] = useState('');
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [crossSell, setCrossSell] = useState<ProductCardModel[]>([]);

  useEffect(() => {
    refresh();
    api
      .get(`checkout/cart/cross-sell${shopQuery()}`)
      .then((data) => setCrossSell(data?.data ?? data ?? []))
      .catch(() => setCrossSell([]));
  }, [refresh, reload]);

  const changeQty = async (itemId: number, quantity: number) => {
    if (quantity < 1) return;
    setBusy(itemId);
    try {
      await updateCartQty(itemId, quantity);
      await refresh();
    } finally {
      setBusy(null);
    }
  };

  const remove = async (itemId: number) => {
    setBusy(itemId);
    try {
      await removeCartItem(itemId);
      await refresh();
    } finally {
      setBusy(null);
    }
  };

  const applyCoupon = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    setMessage('');
    try {
      await api.post(`checkout/cart/coupon${shopQuery()}`, { code: coupon.trim() });
      setMessage('The coupon has been applied.');
      setCoupon('');
      await refresh();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to apply the coupon.');
    }
  };

  const removeCoupon = async () => {
    setError('');
    setMessage('');
    try {
      await api.delete(`checkout/cart/coupon${shopQuery()}`);
      await refresh();
      setMessage('The coupon has been removed.');
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to remove the coupon.');
    }
  };

  if (!cart || !cart.items?.length) {
    return (
      <div className="shop-container">
        <div className="shop-breadcrumbs">
          <Link to="/">Home</Link>
          <span>/</span>
          <span>Shopping cart</span>
        </div>
        <div className="listing-empty" style={{ paddingTop: 50, paddingBottom: 90 }}>
          <span className="empty-icon">
            <IconShoppingBag size={34} />
          </span>
          <h2 className="shop-heading">Your cart is empty</h2>
          <p className="shop-muted">Looks like you haven’t added anything yet. Let’s change that.</p>
          <Link to="/search" className="shop-primary-button">
            Continue shopping <IconArrowRight size={16} />
          </Link>
        </div>

        {crossSell.length > 0 && (
          <section className="shop-section">
            <div className="shop-section-head">
              <h2 className="shop-heading">You may also like</h2>
            </div>
            <div className="shop-grid">
              {crossSell.slice(0, 4).map((product) => (
                <ProductCard key={product.id} product={product} />
              ))}
            </div>
          </section>
        )}
      </div>
    );
  }

  return (
    <div className="shop-container">
      <div className="shop-breadcrumbs">
        <Link to="/">Home</Link>
        <span>/</span>
        <span>Shopping cart</span>
      </div>

      <div className="listing-head" style={{ marginBottom: 20 }}>
        <h1 className="shop-heading" style={{ fontSize: 36 }}>
          Shopping cart
        </h1>
        <span className="listing-count">
          {cart.items_qty} {cart.items_qty === 1 ? 'item' : 'items'}
        </span>
      </div>

      <div className="cart-layout">
        <div>
          <div className="cart-table-wrap">
            <table className="cart-table">
              <thead>
                <tr>
                  <th>Product</th>
                  <th>Price</th>
                  <th>Quantity</th>
                  <th style={{ textAlign: 'right' }}>Total</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {cart.items.map((item: any) => (
                  <tr key={item.id} style={{ opacity: busy === item.id ? 0.55 : 1 }}>
                    <td>
                      <div className="cart-item">
                        <Link to={`/product/${item.product_id}`} className="cart-item-media">
                          <img src={mediaUrl(item.base_image?.url)} alt={item.name} />
                        </Link>
                        <div>
                          <Link to={`/product/${item.product_id}`} className="cart-item-title">
                            {item.name}
                          </Link>
                          <p className="shop-muted" style={{ fontSize: 12, margin: '4px 0 0' }}>
                            SKU: {item.sku}
                          </p>
                        </div>
                      </div>
                    </td>
                    <td>{formattedMoney(item.formatted_price, item.price)}</td>
                    <td>
                      <div className="qty-stepper qty-stepper-sm">
                        <button type="button" onClick={() => changeQty(item.id, item.quantity - 1)} aria-label="Decrease">
                          −
                        </button>
                        <input
                          value={item.quantity}
                          onChange={(event) => changeQty(item.id, Number(event.target.value) || 1)}
                          aria-label="Quantity"
                        />
                        <button type="button" onClick={() => changeQty(item.id, item.quantity + 1)} aria-label="Increase">
                          +
                        </button>
                      </div>
                    </td>
                    <td style={{ textAlign: 'right', fontWeight: 600 }}>
                      {formattedMoney(item.formatted_total, item.total)}
                    </td>
                    <td style={{ textAlign: 'right' }}>
                      <button type="button" className="cart-remove" onClick={() => remove(item.id)} aria-label="Remove item">
                        <IconTrash size={17} />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div style={{ marginTop: 26 }}>
            <Link to="/search" className="shop-secondary-button shop-button-sm">
              Continue shopping
            </Link>
          </div>
        </div>

        <aside className="cart-summary">
          <h2 className="cart-summary-title">Order summary</h2>

          <div className="cart-summary-row">
            <span>Subtotal</span>
            <span>{formattedMoney(cart.formatted_sub_total, cart.sub_total)}</span>
          </div>

          {Number(cart.discount_amount ?? 0) > 0 && (
            <div className="cart-summary-row" style={{ color: '#047857' }}>
              <span>Discount</span>
              <span>−{formattedMoney(cart.formatted_discount_amount, cart.discount_amount)}</span>
            </div>
          )}

          <div className="cart-summary-row">
            <span>Shipping</span>
            <span>{formattedMoney(cart.formatted_shipping_amount, cart.shipping_amount)}</span>
          </div>

          <div className="cart-summary-row">
            <span>Tax</span>
            <span>{formattedMoney(cart.formatted_tax_total, cart.tax_total)}</span>
          </div>

          <div className="cart-summary-row total">
            <span>Grand total</span>
            <span>{formattedMoney(cart.formatted_grand_total, cart.grand_total)}</span>
          </div>

          <div className="coupon-box">
            {cart.coupon_code ? (
              <div className="coupon-applied">
                <span>
                  <IconTag size={14} /> {cart.coupon_code}
                </span>
                <button type="button" onClick={removeCoupon}>
                  Remove
                </button>
              </div>
            ) : (
              <form onSubmit={applyCoupon}>
                <label className="field-label" htmlFor="coupon">
                  Coupon code
                </label>
                <div style={{ display: 'flex', gap: 8 }}>
                  <input
                    id="coupon"
                    className="form-input"
                    value={coupon}
                    onChange={(event) => setCoupon(event.target.value)}
                    placeholder="Enter code"
                  />
                  <button type="submit" className="shop-secondary-button shop-button-sm">
                    Apply
                  </button>
                </div>
              </form>
            )}
            {message && <p className="form-success">{message}</p>}
            {error && <p className="form-error">{error}</p>}
          </div>

          <Link to="/checkout" className="shop-primary-button" style={{ width: '100%', marginTop: 16 }}>
            Proceed to checkout <IconArrowRight size={16} />
          </Link>

          <p className="shop-muted" style={{ fontSize: 11, textAlign: 'center', marginTop: 12 }}>
            Secure checkout · Free returns within 30 days
          </p>
        </aside>
      </div>

      {crossSell.length > 0 && (
        <section className="shop-section">
          <div className="shop-section-head">
            <h2 className="shop-heading">You may also like</h2>
          </div>
          <div className="shop-grid">
            {crossSell.slice(0, 4).map((product) => (
              <ProductCard key={product.id} product={product} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
