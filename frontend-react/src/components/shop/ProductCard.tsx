import { Link } from 'react-router-dom';
import { IconHeart, IconShoppingBag, IconEye } from '@tabler/icons-react';
import { mediaUrl, stripHtml } from '../../lib/format';
import { fallbackImage } from '../../lib/cart';
import Stars from './Stars';
import type { ProductCard as ProductCardModel } from '../../types';

interface Props {
  product: ProductCardModel;
  className?: string;
  list?: boolean;
}

export default function ProductCard({ product, className = '', list = false }: Props) {
  const href = `/product/${product.url_key ?? product.id}`;
  const rating = product.ratings?.average ?? 0;
  const reviews = product.ratings?.total ?? product.reviews?.total ?? 0;
  const image = product.base_image?.url ?? product.images?.[0]?.url ?? '';

  if (list) {
    return (
      <div className={`shop-list-card ${className}`}>
        <Link to={href} className="media">
          <img src={mediaUrl(image) || fallbackImage} alt={product.name ?? 'Product'} loading="lazy" />
        </Link>
        <div style={{ flex: 1, minWidth: 0 }}>
          <Link to={href}>
            <h3 className="product-card-title" style={{ marginTop: 0, fontSize: 20 }}>
              {product.name ?? product.sku}
            </h3>
          </Link>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 8 }}>
            <Stars value={rating} />
            <span className="shop-muted" style={{ fontSize: 12 }}>
              {reviews} {reviews === 1 ? 'Review' : 'Reviews'}
            </span>
          </div>
          <p className="shop-muted" style={{ fontSize: 13, margin: '12px 0', maxWidth: 520 }}>
            {stripHtml(product.short_description) || ' '}
          </p>
          <div className="product-price-row" style={{ marginTop: 0 }}>
            <span>{product.formatted_price}</span>
            {product.on_sale && product.formatted_regular_price && (
              <span className="product-price-regular">{product.formatted_regular_price}</span>
            )}
          </div>
          <div style={{ marginTop: 16, display: 'flex', gap: 10 }}>
            <Link to={href} className="shop-primary-button shop-button-sm">
              <IconShoppingBag size={15} /> View product
            </Link>
            <Link to={`/product/${product.url_key ?? product.id}`} className="shop-secondary-button shop-button-sm">
              Details
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`product-card ${className}`}>
      <Link to={href} className="product-card-media" aria-label={product.name ?? 'Product'}>
        <img src={mediaUrl(image) || fallbackImage} alt={product.name ?? 'Product'} loading="lazy" />
        {product.on_sale && <span className="product-badge sale">Sale</span>}
        {!product.on_sale && product.is_new && <span className="product-badge new">New</span>}
        {reviews > 0 && (
          <span className="product-rating-chip">
            <Stars value={rating} size={11} /> {rating.toFixed(1)}
          </span>
        )}
        <div className="product-card-quick">
          <button
            type="button"
            className="quick-cart"
            onClick={(event) => {
              event.preventDefault();
              window.location.assign(href);
            }}
          >
            {product.type === 'configurable' ? 'Choose options' : 'Add to cart'}
          </button>
          <span className="quick-icon" role="button" aria-label="View product">
            <IconEye size={16} />
          </span>
          <span className="quick-icon" role="button" aria-label="Add to wishlist">
            <IconHeart size={16} />
          </span>
        </div>
      </Link>
      <Link to={href}>
        <h3 className="product-card-title">{product.name ?? product.sku ?? 'Unnamed product'}</h3>
      </Link>
      <div className="product-price-row">
        <span>{product.formatted_price}</span>
        {product.on_sale && product.formatted_regular_price && (
          <span className="product-price-regular">{product.formatted_regular_price}</span>
        )}
      </div>
      {!product.in_stock && (
        <span className="shop-muted" style={{ fontSize: 12, marginTop: 4 }}>
          Out of stock
        </span>
      )}
    </div>
  );
}
