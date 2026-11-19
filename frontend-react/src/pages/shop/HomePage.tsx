import { useEffect, useState } from 'react';
import { api } from '../../api/client';
import { shopQuery } from '../../store';
import { useShopReload } from '../../lib/hooks';
import {
  CategoryCarousel,
  HeroCarousel,
  ProductCarousel,
  PromoBanner,
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

  useEffect(() => {
    api
      .get(`core/home${shopQuery()}`)
      .then((data) => setSections(Array.isArray(data) ? data : []))
      .catch(() => setSections([]));
  }, [reload]);

  const hero = sections.find((section) => section.type === 'image_carousel');
  const staticSections = sections.filter(
    (section) => section.type !== 'image_carousel' && section.type !== 'category_carousel' && section.type !== 'product_carousel',
  );

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

      <ProductCarousel title="Best Sellers" params={{ sort: 'price_asc' }} href="/search?sort=price_asc" />

      <section className="shop-section">
        <div className="shop-container">
          <div className="home-editorial">
            <div className="home-editorial-copy">
              <span className="shop-hero-eyebrow" style={{ color: '#c2a26b' }}>
                The EzCommerce journal
              </span>
              <h2 className="shop-heading" style={{ fontSize: 38 }}>
                Considered design, made to last.
              </h2>
              <p className="shop-muted" style={{ lineHeight: 1.7, margin: '16px 0 24px' }}>
                Every piece in our collection is chosen for its craftsmanship, its materials and the way it fits into
                everyday life. Discover the stories behind the products and the people who make them.
              </p>
              <a href="#top" className="shop-secondary-button">
                Read the journal
              </a>
            </div>
            <div className="home-editorial-stats">
              <div>
                <strong>12k+</strong>
                <span>Happy customers</span>
              </div>
              <div>
                <strong>4.8/5</strong>
                <span>Average rating</span>
              </div>
              <div>
                <strong>30 days</strong>
                <span>Free returns</span>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
