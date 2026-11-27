import { useEffect, useState } from 'react';
import { api } from '../../api/client';
import { shopQuery } from '../../store';
import { useShopReload } from '../../lib/hooks';
import { t } from '../../i18n';
import {
  CarouselSkeleton,
  CategoryCarousel,
  HeroCarousel,
  HeroSkeleton,
  ProductCarousel,
  PromoBanner,
  PromoSkeleton,
  ServicesStrip,
} from '../../components/shop/Sections';

interface Section {
  id: number;
  type: string;
  name: string;
  options: any;
}

export default function HomePage() {
  const reload = useShopReload();
  const [sections, setSections] = useState<Section[]>([]);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    let cancelled = false;
    api
      .get(`core/home${shopQuery()}`)
      .then((data) => {
        if (!cancelled) setSections(Array.isArray(data) ? data : []);
      })
      .catch(() => {
        if (!cancelled) setSections([]);
      })
      .finally(() => {
        if (!cancelled) setLoaded(true);
      });
    return () => {
      cancelled = true;
    };
  }, [reload]);

  const hero = sections.find((section) => section.type === 'image_carousel');
  const staticSections = sections.filter(
    (section) => section.type !== 'image_carousel' && section.type !== 'category_carousel' && section.type !== 'product_carousel',
  );

  if (!loaded) {
    return (
      <div>
        <HeroSkeleton />
        <ServicesStrip />
        <CarouselSkeleton title={t('Shop by Category')} variant="category" />
        <CarouselSkeleton title={t('New Products')} />
        <PromoSkeleton />
        <CarouselSkeleton title={t('Best Sellers')} />
        <EditorialSection />
      </div>
    );
  }

  return (
    <div>
      {hero?.options?.slides?.length ? <HeroCarousel slides={hero.options.slides} /> : null}

      <ServicesStrip />

      {sections
        .filter((section) => section.type === 'category_carousel')
        .map((section) => (
          <CategoryCarousel
            key={section.id}
            title={section.options?.title ?? section.name}
            limit={section.options?.limit ?? 8}
          />
        ))}

      {sections
        .filter((section) => section.type === 'product_carousel')
        .map((section) => (
          <ProductCarousel
            key={section.id}
            title={section.options?.title ?? section.name}
            params={section.options?.filters ?? {}}
            href={section.options?.filters?.sort === 'newest' ? '/search?sort=newest' : '/search'}
          />
        ))}

      {staticSections.map((section) => (
        <PromoBanner key={section.id} options={section.options} />
      ))}

      <ProductCarousel title={t('Best Sellers')} params={{ sort: 'price_asc' }} href="/search?sort=price_asc" />

      <EditorialSection />
    </div>
  );
}

function EditorialSection() {
  return (
    <section className="shop-section">
      <div className="shop-container">
        <div className="home-editorial">
          <div className="home-editorial-copy">
            <span className="shop-hero-eyebrow" style={{ color: '#6b4f1d' }}>
              {t('The EzCommerce journal')}
            </span>
            <h2 className="shop-heading" style={{ fontSize: 38 }}>
              {t('Considered design, made to last.')}
            </h2>
            <p className="shop-muted" style={{ lineHeight: 1.7, margin: '16px 0 24px' }}>
              {t(
                'Every piece in our collection is chosen for its craftsmanship, its materials and the way it fits into everyday life. Discover the stories behind the products and the people who make them.',
              )}
            </p>
            <a href="#top" className="shop-secondary-button">
              {t('Read the journal')}
            </a>
          </div>
          <div className="home-editorial-stats">
            <div>
              <strong>12k+</strong>
              <span>{t('Happy customers')}</span>
            </div>
            <div>
              <strong>4.8/5</strong>
              <span>{t('Average rating')}</span>
            </div>
            <div>
              <strong>{t('30 days')}</strong>
              <span>{t('Free returns')}</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
