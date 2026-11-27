import { Link } from 'react-router-dom';
import { IconHeart, IconShoppingBag, IconEye } from '@tabler/icons-react';
import { stripHtml } from '../../lib/format';
import Stars from './Stars';
import ProductImage from './ProductImage';
import { t } from '../../i18n';
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
  const image = product.base_image ?? product.images?.[0] ?? null;

  if (list) {
    return (
      <div className={`shop-list-card ${className}`}>
        <Link to={href} className="media">
          <ProductImage
            image={image}
            alt={product.name ?? t('Product')}
            sizes="(max-width: 900px) 100vw, 220px"
          />
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
              {t(reviews === 1 ? '{count} Review' : '{count} Reviews', { count: reviews })}
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
              <IconShoppingBag size={15} /> {t('View product')}
            </Link>
            <Link to={`/product/${product.url_key ?? product.id}`} className="shop-secondary-button shop-button-sm">
              {t('Details')}
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`product-card ${className}`}>
      <Link to={href} className="product-card-media">
        <ProductImage
          image={image}
          alt={product.name ?? t('Product')}
          sizes="(max-width: 640px) 50vw, (max-width: 1024px) 33vw, 25vw"
        />
        {product.on_sale && <span className="product-badge sale">{t('Sale')}</span>}
        {!product.on_sale && product.is_new && <span className="product-badge new">{t('New')}</span>}
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
            {product.type === 'configurable' ? t('Choose options') : t('Add to cart')}
          </button>
          <span className="quick-icon" role="button" aria-label={t('View product')}>
            <IconEye size={16} />
          </span>
          <span className="quick-icon" role="button" aria-label={t('Add to wishlist')}>
            <IconHeart size={16} />
          </span>
        </div>
      </Link>
      <Link to={href}>
        <h3 className="product-card-title">{product.name ?? product.sku ?? t('Unnamed product')}</h3>
      </Link>
      <div className="product-price-row">
        <span>{product.formatted_price}</span>
        {product.on_sale && product.formatted_regular_price && (
          <span className="product-price-regular">{product.formatted_regular_price}</span>
        )}
      </div>
      {!product.in_stock && (
        <span className="shop-muted" style={{ fontSize: 12, marginTop: 4 }}>
          {t('Out of stock')}
        </span>
      )}
    </div>
  );
}
