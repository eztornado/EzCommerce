import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { IconArrowRight, IconShoppingBag, IconTag, IconTrash } from '@tabler/icons-react';
import { api } from '../../api/client';
import { useCart, shopQuery } from '../../store';
import { mediaUrl, formattedMoney } from '../../lib/format';
import { updateCartQty, removeCartItem } from '../../lib/cart';
import { useShopReload } from '../../lib/hooks';
import { t } from '../../i18n';
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
      setMessage(t('The coupon has been applied.'));
      setCoupon('');
      await refresh();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to apply the coupon.'));
    }
  };

  const removeCoupon = async () => {
    setError('');
    setMessage('');
    try {
      await api.delete(`checkout/cart/coupon${shopQuery()}`);
      await refresh();
      setMessage(t('The coupon has been removed.'));
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to remove the coupon.'));
    }
  };

  if (!cart || !cart.items?.length) {
    return (
      <div className="shop-container">
        <div className="shop-breadcrumbs">
          <Link to="/">{t('Home')}</Link>
          <span>/</span>
          <span>{t('Shopping cart')}</span>
        </div>
        <div className="listing-empty" style={{ paddingTop: 50, paddingBottom: 90 }}>
          <span className="empty-icon">
            <IconShoppingBag size={34} />
          </span>
          <h2 className="shop-heading">{t('Your cart is empty')}</h2>
          <p className="shop-muted">{t('Looks like you haven’t added anything yet. Let’s change that.')}</p>
          <Link to="/search" className="shop-primary-button">
            {t('Continue shopping')} <IconArrowRight size={16} />
          </Link>
        </div>

        {crossSell.length > 0 && (
          <section className="shop-section">
            <div className="shop-section-head">
              <h2 className="shop-heading">{t('You may also like')}</h2>
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
        <Link to="/">{t('Home')}</Link>
        <span>/</span>
        <span>{t('Shopping cart')}</span>
      </div>

      <div className="listing-head" style={{ marginBottom: 20 }}>
        <h1 className="shop-heading" style={{ fontSize: 36 }}>
          {t('Shopping cart')}
        </h1>
        <span className="listing-count">
          {t(cart.items_qty === 1 ? '{count} item' : '{count} items', { count: cart.items_qty })}
        </span>
      </div>

      <div className="cart-layout">
        <div>
          <div className="cart-table-wrap">
            <table className="cart-table">
              <thead>
                <tr>
                  <th>{t('Product')}</th>
                  <th>{t('Price')}</th>
                  <th>{t('Quantity')}</th>
                  <th style={{ textAlign: 'right' }}>{t('Total')}</th>
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
                            {t('SKU')}: {item.sku}
                          </p>
                        </div>
                      </div>
                    </td>
                    <td>{formattedMoney(item.formatted_price, item.price)}</td>
                    <td>
                      <div className="qty-stepper qty-stepper-sm">
                        <button type="button" onClick={() => changeQty(item.id, item.quantity - 1)} aria-label={t('Decrease')}>
                          −
                        </button>
                        <input
                          value={item.quantity}
                          onChange={(event) => changeQty(item.id, Number(event.target.value) || 1)}
                          aria-label={t('Quantity')}
                        />
                        <button type="button" onClick={() => changeQty(item.id, item.quantity + 1)} aria-label={t('Increase')}>
                          +
                        </button>
                      </div>
                    </td>
                    <td style={{ textAlign: 'right', fontWeight: 600 }}>
                      {formattedMoney(item.formatted_total, item.total)}
                    </td>
                    <td style={{ textAlign: 'right' }}>
                      <button type="button" className="cart-remove" onClick={() => remove(item.id)} aria-label={t('Remove item')}>
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
              {t('Continue shopping')}
            </Link>
          </div>
        </div>

        <aside className="cart-summary">
          <h2 className="cart-summary-title">{t('Order summary')}</h2>

          <div className="cart-summary-row">
            <span>{t('Subtotal')}</span>
            <span>{formattedMoney(cart.formatted_sub_total, cart.sub_total)}</span>
          </div>

          {Number(cart.discount_amount ?? 0) > 0 && (
            <div className="cart-summary-row" style={{ color: '#047857' }}>
              <span>{t('Discount')}</span>
              <span>−{formattedMoney(cart.formatted_discount_amount, cart.discount_amount)}</span>
            </div>
          )}

          <div className="cart-summary-row">
            <span>{t('Shipping')}</span>
            <span>{formattedMoney(cart.formatted_shipping_amount, cart.shipping_amount)}</span>
          </div>

          <div className="cart-summary-row">
            <span>{t('Tax')}</span>
            <span>{formattedMoney(cart.formatted_tax_total, cart.tax_total)}</span>
          </div>

          <div className="cart-summary-row total">
            <span>{t('Grand total')}</span>
            <span>{formattedMoney(cart.formatted_grand_total, cart.grand_total)}</span>
          </div>

          <div className="coupon-box">
            {cart.coupon_code ? (
              <div className="coupon-applied">
                <span>
                  <IconTag size={14} /> {cart.coupon_code}
                </span>
                <button type="button" onClick={removeCoupon}>
                  {t('Remove')}
                </button>
              </div>
            ) : (
              <form onSubmit={applyCoupon}>
                <label className="field-label" htmlFor="coupon">
                  {t('Coupon code')}
                </label>
                <div style={{ display: 'flex', gap: 8 }}>
                  <input
                    id="coupon"
                    className="form-input"
                    value={coupon}
                    onChange={(event) => setCoupon(event.target.value)}
                    placeholder={t('Enter code')}
                  />
                  <button type="submit" className="shop-secondary-button shop-button-sm">
                    {t('Apply')}
                  </button>
                </div>
              </form>
            )}
            {message && <p className="form-success">{message}</p>}
            {error && <p className="form-error">{error}</p>}
          </div>

          <Link to="/checkout" className="shop-primary-button" style={{ width: '100%', marginTop: 16 }}>
            {t('Proceed to checkout')} <IconArrowRight size={16} />
          </Link>

          <p className="shop-muted" style={{ fontSize: 11, textAlign: 'center', marginTop: 12 }}>
            {t('Secure checkout · Free returns within 30 days')}
          </p>
        </aside>
      </div>

      {crossSell.length > 0 && (
        <section className="shop-section">
          <div className="shop-section-head">
            <h2 className="shop-heading">{t('You may also like')}</h2>
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
