import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import { IconHeart, IconMinus, IconPlus, IconShoppingBag, IconTruck, IconRefresh, IconShieldCheck } from '@tabler/icons-react';
import { api } from '../../api/client';
import { shopQuery, useAuth } from '../../store';
import { mediaUrl, money, stripHtml } from '../../lib/format';
import { addToCart, fallbackImage } from '../../lib/cart';
import { useShopReload } from '../../lib/hooks';
import Stars from '../../components/shop/Stars';
import ProductCard from '../../components/shop/ProductCard';
import type { ProductCard as ProductCardModel } from '../../types';

export default function ProductPage() {
  const { key } = useParams();
  const reload = useShopReload();
  const navigate = useNavigate();
  const loggedIn = useAuth((state) => state.loggedIn);
  const [product, setProduct] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [activeImage, setActiveImage] = useState(0);
  const [quantity, setQuantity] = useState(1);
  const [options, setOptions] = useState<Record<string, number>>({});
  const [tab, setTab] = useState<'description' | 'additional' | 'reviews'>('description');
  const [reviews, setReviews] = useState<any[]>([]);
  const [related, setRelated] = useState<ProductCardModel[]>([]);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [reviewForm, setReviewForm] = useState({ name: '', title: '', comment: '', rating: 5 });
  const [reviewSent, setReviewSent] = useState(false);

  useEffect(() => {
    setLoading(true);
    setActiveImage(0);
    setQuantity(1);
    setOptions({});
    setMessage('');
    setError('');
    api
      .get(`products/${key}${shopQuery()}`)
      .then((data) => {
        setProduct(data);
        const defaults: Record<string, number> = {};
        (data?.super_attributes ?? []).forEach((attribute: any) => {
          if (attribute.options?.length) defaults[attribute.code] = attribute.options[0].id;
        });
        setOptions(defaults);
        if (data?.id) {
          api.get(`products/${data.id}/related${shopQuery()}`).then((items) => setRelated(items?.data ?? items ?? [])).catch(() => setRelated([]));
          api.get(`product/${data.id}/reviews${shopQuery()}`).then((payload) => setReviews(payload?.data ?? [])).catch(() => setReviews([]));
        }
      })
      .catch(() => setProduct(null))
      .finally(() => setLoading(false));
  }, [key, reload]);

  const images = useMemo(() => {
    const list = [product?.base_image, ...(product?.images ?? [])].filter(Boolean) as { url: string }[];
    const unique = new Map(list.map((image) => [image.url, image]));
    return Array.from(unique.values());
  }, [product]);

  const variant = useMemo(() => {
    if (!product?.variants?.length) return null;
    return (
      product.variants.find((entry: any) =>
        Object.entries(entry.attributes ?? {}).every(([code, value]) => Number(options[code]) === Number(value)),
      ) ?? null
    );
  }, [product, options]);

  const add = async () => {
    setError('');
    setMessage('');
    try {
      await addToCart(product.id, quantity, options);
      setMessage('The item has been added to your cart.');
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to add this product to the cart.');
    }
  };

  const buyNow = async () => {
    await add();
    navigate('/checkout');
  };

  const addWishlist = async () => {
    if (!loggedIn) {
      navigate('/login');
      return;
    }
    try {
      await api.post('customer/wishlist', { product_id: product.id });
      setMessage('Added to your wishlist.');
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to add to the wishlist.');
    }
  };

  const submitReview = async (event: React.FormEvent) => {
    event.preventDefault();
    setError('');
    try {
      await api.post(`product/${product.id}/review`, reviewForm);
      setReviewSent(true);
      setReviewForm({ name: '', title: '', comment: '', rating: 5 });
      const payload = await api.get(`product/${product.id}/reviews${shopQuery()}`);
      setReviews(payload?.data ?? []);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to submit your review.');
    }
  };

  if (loading) {
    return (
      <div className="shop-container" style={{ paddingTop: 40 }}>
        <div className="product-detail">
          <div className="shimmer-block" style={{ aspectRatio: '1' }} />
          <div>
            <div className="shimmer-line" style={{ width: '70%', height: 36 }} />
            <div className="shimmer-line" style={{ width: '35%', marginTop: 16 }} />
            <div className="shimmer-line" style={{ width: '90%', marginTop: 24 }} />
            <div className="shimmer-line" style={{ width: '60%', marginTop: 10 }} />
          </div>
        </div>
      </div>
    );
  }

  if (!product) {
    return (
      <div className="shop-container listing-empty" style={{ paddingTop: 80, paddingBottom: 80 }}>
        <IconShoppingBag size={42} />
        <h2>Product not found</h2>
        <p>The product you are looking for is no longer available.</p>
        <Link to="/search" className="shop-primary-button">
          Continue shopping
        </Link>
      </div>
    );
  }

  const currentPrice = variant?.price != null ? Number(variant.price) : Number(product.price);
  const regularNumber =
    product.regular_price != null
      ? Number(product.regular_price)
      : Number(String(product.formatted_regular_price ?? '').replace(/[^0-9.]/g, '')) || null;
  const showCompare = Boolean(product.on_sale && regularNumber && currentPrice && regularNumber > currentPrice + 0.001);
  const price = variant?.price != null ? money(currentPrice) : product.formatted_price;
  const sku = variant?.sku ?? product.sku;

  return (
    <div className="shop-container">
      <nav className="shop-breadcrumbs" aria-label="Breadcrumb">
        <Link to="/">Home</Link>
        <span>/</span>
        <Link to="/search">Shop</Link>
        <span>/</span>
        <span>{product.name}</span>
      </nav>

      <div className="product-detail">
        <div className="product-gallery">
          <div className="product-thumbs">
            {images.map((image, index) => (
              <button
                type="button"
                key={image.url}
                className="product-thumb"
                data-active={index === activeImage}
                onClick={() => setActiveImage(index)}
                aria-label={`View image ${index + 1}`}
              >
                <img src={mediaUrl(image.url)} alt={`${product.name} ${index + 1}`} />
              </button>
            ))}
          </div>
          <div className="product-main-image">
            <img
              src={mediaUrl(images[activeImage]?.url) || fallbackImage}
              alt={product.name}
            />
            {product.on_sale && <span className="product-badge sale">Sale</span>}
          </div>
        </div>

        <div className="product-info">
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10 }}>
            <Stars value={product.ratings?.average ?? 0} size={15} />
            <button
              type="button"
              className="shop-link"
              style={{ background: 'none', border: 0, padding: 0, cursor: 'pointer', fontSize: 13 }}
              onClick={() => setTab('reviews')}
            >
              {product.ratings?.total ?? 0} Reviews
            </button>
          </div>

          <h1 className="product-info-title">{product.name}</h1>

          <div className="product-info-price-row">
            <span className="product-info-price">{price}</span>
            {showCompare && (
              <span className="product-price-regular" style={{ fontSize: 18 }}>
                {product.formatted_regular_price}
              </span>
            )}
            {showCompare && (
              <span className="product-save-badge">Save {money(regularNumber! - currentPrice)}</span>
            )}
          </div>

          <p className="shop-muted" style={{ fontSize: 14, marginTop: 6 }}>
            SKU: <strong style={{ color: '#111827' }}>{sku}</strong>
            {'  ·  '}
            {product.in_stock ? (
              <span style={{ color: '#047857', fontWeight: 600 }}>In stock</span>
            ) : (
              <span style={{ color: '#b91c1c', fontWeight: 600 }}>Out of stock</span>
            )}
          </p>

          {product.short_description && (
            <p style={{ marginTop: 18, lineHeight: 1.7, color: '#4b5563', fontSize: 15 }}>{product.short_description}</p>
          )}

          {(product.super_attributes ?? []).map((attribute: any) => (
            <div className="product-option" key={attribute.code}>
              <p className="product-option-label">Choose {attribute.label ?? attribute.code}</p>
              <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
                {(attribute.options ?? []).map((option: any) => {
                  const active = Number(options[attribute.code]) === Number(option.id);
                  const swatch = option.swatch_value;
                  return (
                    <button
                      key={option.id}
                      type="button"
                      className={swatch ? 'option-swatch' : 'option-chip'}
                      data-active={active}
                      style={swatch ? { background: swatch } : undefined}
                      title={option.admin_name}
                      onClick={() => setOptions((current) => ({ ...current, [attribute.code]: option.id }))}
                    >
                      {!swatch && option.admin_name}
                    </button>
                  );
                })}
              </div>
            </div>
          ))}

          <div className="product-buy-row">
            <div className="qty-stepper">
              <button type="button" onClick={() => setQuantity((value) => Math.max(1, value - 1))} aria-label="Decrease quantity">
                <IconMinus size={14} />
              </button>
              <input
                value={quantity}
                onChange={(event) => setQuantity(Math.max(1, Number(event.target.value) || 1))}
                aria-label="Quantity"
              />
              <button type="button" onClick={() => setQuantity((value) => value + 1)} aria-label="Increase quantity">
                <IconPlus size={14} />
              </button>
            </div>
            <button type="button" className="shop-primary-button" onClick={add} disabled={!product.in_stock}>
              <IconShoppingBag size={17} /> Add to cart
            </button>
            <button type="button" className="shop-secondary-button" onClick={buyNow} disabled={!product.in_stock}>
              Buy now
            </button>
            <button type="button" className="product-wishlist" onClick={addWishlist} aria-label="Add to wishlist">
              <IconHeart size={20} />
            </button>
          </div>

          {message && <p className="form-success">{message}</p>}
          {error && <p className="form-error">{error}</p>}

          <div className="product-assurances">
            <div>
              <IconTruck size={20} />
              <span>Free delivery over €100</span>
            </div>
            <div>
              <IconRefresh size={20} />
              <span>30-day returns</span>
            </div>
            <div>
              <IconShieldCheck size={20} />
              <span>Secure payment</span>
            </div>
          </div>
        </div>
      </div>

      <div className="product-tabs">
        <div className="product-tab-list">
          <button type="button" className="product-tab" data-active={tab === 'description'} onClick={() => setTab('description')}>
            Description
          </button>
          <button type="button" className="product-tab" data-active={tab === 'additional'} onClick={() => setTab('additional')}>
            Additional information
          </button>
          <button type="button" className="product-tab" data-active={tab === 'reviews'} onClick={() => setTab('reviews')}>
            Reviews ({reviews.length})
          </button>
        </div>

        <div className="product-tab-content">
          {tab === 'description' && (
            <div style={{ maxWidth: 900, lineHeight: 1.8, color: '#374151' }}>
              <p>{stripHtml(product.description) || product.short_description || 'No description available for this product.'}</p>
            </div>
          )}

          {tab === 'additional' && (
            <table className="additional-table">
              <tbody>
                <tr>
                  <th>SKU</th>
                  <td>{sku}</td>
                </tr>
                <tr>
                  <th>Type</th>
                  <td style={{ textTransform: 'capitalize' }}>{product.type}</td>
                </tr>
                {(product.additional_information ?? []).map((row: any) => (
                  <tr key={row.code}>
                    <th>{row.label}</th>
                    <td>{row.value}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {tab === 'reviews' && (
            <div className="reviews-layout">
              <div>
                <h3 className="shop-subheading" style={{ fontSize: 18 }}>
                  Customer reviews
                </h3>
                {reviews.length === 0 ? (
                  <p className="shop-muted" style={{ marginTop: 10 }}>
                    There are no reviews yet. Be the first to review this product.
                  </p>
                ) : (
                  <div className="reviews-list">
                    {reviews.map((review) => (
                      <article className="review-card" key={review.id}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', gap: 12 }}>
                          <div>
                            <strong>{review.title}</strong>
                            <p className="shop-muted" style={{ fontSize: 12, margin: '3px 0 0' }}>
                              {review.name} · {new Date(review.created_at?.replace(' ', 'T') ?? Date.now()).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' })}
                            </p>
                          </div>
                          <Stars value={review.rating} />
                        </div>
                        <p style={{ margin: '10px 0 0', color: '#4b5563', lineHeight: 1.65 }}>{review.comment}</p>
                      </article>
                    ))}
                  </div>
                )}
              </div>

              <form className="review-form" onSubmit={submitReview}>
                <h3 className="shop-subheading" style={{ fontSize: 18 }}>
                  Write a review
                </h3>
                <label className="field-label">Rating</label>
                <div style={{ display: 'flex', gap: 4, marginBottom: 14 }}>
                  {[1, 2, 3, 4, 5].map((value) => (
                    <button
                      key={value}
                      type="button"
                      className="rating-button"
                      onClick={() => setReviewForm((current) => ({ ...current, rating: value }))}
                      aria-label={`${value} stars`}
                    >
                      <Stars value={reviewForm.rating >= value ? 5 : 0} size={20} />
                    </button>
                  ))}
                </div>
                <label className="field-label" htmlFor="review-name">
                  Name
                </label>
                <input
                  id="review-name"
                  className="form-input"
                  value={reviewForm.name}
                  onChange={(event) => setReviewForm((current) => ({ ...current, name: event.target.value }))}
                  required
                />
                <label className="field-label" htmlFor="review-title">
                  Title
                </label>
                <input
                  id="review-title"
                  className="form-input"
                  value={reviewForm.title}
                  onChange={(event) => setReviewForm((current) => ({ ...current, title: event.target.value }))}
                  required
                />
                <label className="field-label" htmlFor="review-comment">
                  Review
                </label>
                <textarea
                  id="review-comment"
                  className="form-input"
                  rows={4}
                  value={reviewForm.comment}
                  onChange={(event) => setReviewForm((current) => ({ ...current, comment: event.target.value }))}
                  required
                />
                <button type="submit" className="shop-primary-button" style={{ width: '100%' }}>
                  Submit review
                </button>
                {reviewSent && <p className="form-success">Thank you! Your review has been submitted.</p>}
              </form>
            </div>
          )}
        </div>
      </div>

      {related.length > 0 && (
        <section className="shop-section">
          <div className="shop-section-head">
            <h2 className="shop-heading">You may also like</h2>
            <Link to="/search" className="shop-link" style={{ fontSize: 14 }}>
              View all →
            </Link>
          </div>
          <div className="shop-grid">
            {related.slice(0, 4).map((item) => (
              <ProductCard key={item.id} product={item} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
