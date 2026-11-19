import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { IconArrowLeft, IconArrowRight, IconCreditCard, IconHeadphones, IconRefresh, IconTruck } from '@tabler/icons-react';
import { api } from '../../api/client';
import { shopQuery } from '../../store';
import { mediaUrl } from '../../lib/format';
import { fallbackImage } from '../../lib/cart';
import { useShopReload } from '../../lib/hooks';
import ProductCard from './ProductCard';
import type { Category, ProductCard as ProductCardModel } from '../../types';

export function SectionHead({ title, href, linkLabel = 'View all' }: { title: string; href?: string; linkLabel?: string }) {
  const trackRef = useRef<HTMLDivElement>(null);
  const scroll = (direction: number) => {
    const node = trackRef.current;
    if (!node) return;
    node.scrollBy({ left: direction * (node.clientWidth * 0.8), behavior: 'smooth' });
  };
  return (
    <div className="shop-section-head">
      <h2 className="shop-heading">{title}</h2>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        {href && (
          <Link to={href} className="shop-link" style={{ fontSize: 14 }}>
            {linkLabel} →
          </Link>
        )}
        <CarouselArrows onPrev={() => scroll(-1)} onNext={() => scroll(1)} />
      </div>
    </div>
  );
}

export function CarouselArrows({ onPrev, onNext }: { onPrev: () => void; onNext: () => void }) {
  return (
    <div style={{ display: 'flex', gap: 8 }}>
      <button type="button" className="shop-carousel-arrow" onClick={onPrev} aria-label="Previous">
        <IconArrowLeft size={18} />
      </button>
      <button type="button" className="shop-carousel-arrow" onClick={onNext} aria-label="Next">
        <IconArrowRight size={18} />
      </button>
    </div>
  );
}

export function useTrackScroll() {
  const ref = useRef<HTMLDivElement>(null);
  const scroll = (direction: number) => {
    const node = ref.current;
    if (!node) return;
    node.scrollBy({ left: direction * (node.clientWidth * 0.8), behavior: 'smooth' });
  };
  return { ref, scroll };
}

export function ProductCarousel({ title, params, href }: { title: string; params?: Record<string, string | number>; href?: string }) {
  const reload = useShopReload();
  const [products, setProducts] = useState<ProductCardModel[]>([]);
  const { ref, scroll } = useTrackScroll();

  useEffect(() => {
    api
      .get(`products${shopQuery({ per_page: 8, ...params })}`)
      .then((data) => {
        const list: ProductCardModel[] = data?.data ?? (Array.isArray(data) ? data : []);
        setProducts(list.filter((product) => product?.name));
      })
      .catch(() => setProducts([]));
  }, [reload, JSON.stringify(params)]);

  if (!products.length) return null;

  return (
    <section className="shop-section">
      <div className="shop-container">
        <div className="shop-section-head">
          <h2 className="shop-heading">{title}</h2>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            {href && (
              <Link to={href} className="shop-link" style={{ fontSize: 14 }}>
                View all →
              </Link>
            )}
            <CarouselArrows onPrev={() => scroll(-1)} onNext={() => scroll(1)} />
          </div>
        </div>
        <div className="shop-carousel-track" ref={ref}>
          {products.map((product) => (
            <div className="shop-carousel-item" key={product.id}>
              <ProductCard product={product} />
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

export function CategoryCarousel({ title, limit = 8 }: { title: string; limit?: number }) {
  const reload = useShopReload();
  const [categories, setCategories] = useState<Category[]>([]);
  const { ref, scroll } = useTrackScroll();

  useEffect(() => {
    api
      .get(`categories${shopQuery({ per_page: limit })}`)
      .then((data) => setCategories(data?.data ?? (Array.isArray(data) ? data : [])))
      .catch(() => setCategories([]));
  }, [reload, limit]);

  if (!categories.length) return null;

  return (
    <section className="shop-section">
      <div className="shop-container">
        <div className="shop-section-head">
          <h2 className="shop-heading">{title}</h2>
          <CarouselArrows onPrev={() => scroll(-1)} onNext={() => scroll(1)} />
        </div>
        <div className="shop-carousel-track" ref={ref}>
          {categories.map((category) => (
            <Link to={`/category/${category.id}`} className="shop-carousel-item category-tile" key={category.id}>
              <div className="category-tile-media">
                <img src={mediaUrl(category.banner_url) || fallbackImage} alt={category.name ?? 'Category'} loading="lazy" />
              </div>
              <h3 className="category-tile-title">{category.name}</h3>
              <span className="shop-muted" style={{ fontSize: 13 }}>
                Shop now →
              </span>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}

export function HeroCarousel({ slides }: { slides: any[] }) {
  const [active, setActive] = useState(0);
  useEffect(() => {
    if (slides.length < 2) return;
    const timer = window.setInterval(() => setActive((current) => (current + 1) % slides.length), 6000);
    return () => window.clearInterval(timer);
  }, [slides.length]);

  if (!slides.length) return null;

  return (
    <section className="shop-hero" aria-label="Featured collections">
      {slides.map((slide, index) => (
        <div className="shop-hero-slide" data-active={index === active} key={index} aria-hidden={index !== active}>
          <img src={mediaUrl(slide.image)} alt={slide.title ?? 'Collection'} />
          <div className="shop-hero-overlay" />
          <div className="shop-container" style={{ height: '100%' }}>
            <div className="shop-hero-content">
              <span className="shop-hero-eyebrow">EzCommerce · New season</span>
              <h1 className="shop-hero-title">{slide.title}</h1>
              <p className="shop-hero-subtitle">{slide.subtitle}</p>
              <Link
                to={slide.link ?? slide.button?.link ?? '/products'}
                className="shop-primary-button"
                style={{ background: '#fff', color: '#060C3B', borderColor: '#fff' }}
              >
                {slide.button?.title ?? 'Shop now'}
              </Link>
            </div>
          </div>
        </div>
      ))}
      <div className="shop-hero-dots">
        {slides.map((_, index) => (
          <button
            key={index}
            type="button"
            className="shop-hero-dot"
            data-active={index === active}
            aria-label={`Go to slide ${index + 1}`}
            onClick={() => setActive(index)}
          />
        ))}
      </div>
    </section>
  );
}

export function PromoBanner({ options }: { options: any }) {
  return (
    <section className="shop-section">
      <div className="shop-container">
        <div className="promo-banner" style={options?.image ? { backgroundImage: `url(${mediaUrl(options.image)})` } : undefined}>
          <div className="promo-banner-content">
            <span className="shop-hero-eyebrow" style={{ color: '#c2a26b' }}>
              Limited time
            </span>
            <h2 className="shop-heading" style={{ fontSize: 40 }}>
              {options?.title}
            </h2>
            <p style={{ fontSize: 16, margin: '12px 0 22px', color: 'rgba(255,255,255,0.86)' }}>
              {options?.subtitle}
            </p>
            {options?.button?.link && (
              <Link to={options.button.link} className="shop-primary-button">
                {options.button.title ?? 'Shop now'}
              </Link>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}

export function ServicesStrip() {
  const items = [
    { icon: <IconTruck size={22} />, title: 'Free Shipping', text: 'On orders over €100' },
    { icon: <IconRefresh size={22} />, title: 'Easy Returns', text: '30-day return policy' },
    { icon: <IconCreditCard size={22} />, title: 'Secure Payment', text: 'Protected checkout' },
    { icon: <IconHeadphones size={22} />, title: '24/7 Support', text: 'We are here to help' },
  ];
  return (
    <div className="shop-container">
      <div className="shop-services">
        {items.map((item) => (
          <div className="shop-service" key={item.title}>
            <span className="shop-service-icon">{item.icon}</span>
            <div>
              <p className="shop-subheading" style={{ fontSize: 14 }}>
                {item.title}
              </p>
              <p className="shop-muted" style={{ fontSize: 12, margin: '3px 0 0' }}>
                {item.text}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
