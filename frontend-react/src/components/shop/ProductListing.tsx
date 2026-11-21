import { useEffect, useMemo, useState } from 'react';
import { IconChevronDown, IconFilter, IconGridDots, IconList, IconSearch } from '@tabler/icons-react';
import { api } from '../../api/client';
import { shopQuery } from '../../store';
import { useShopReload } from '../../lib/hooks';
import ProductCard from './ProductCard';
import { t } from '../../i18n';
import type { ProductCard as ProductCardModel } from '../../types';

interface AttributeOption {
  id: number;
  admin_name: string;
  swatch_value?: string | null;
}

interface Attribute {
  id: number;
  code: string;
  admin_name: string;
  type: string;
  swatch_type?: string | null;
  options?: AttributeOption[];
}

interface Props {
  title: string;
  description?: string | null;
  categoryId?: number;
  search?: string;
  eyebrow?: string;
}

const SORTS = [
  { value: 'newest', label: 'Newest first' },
  { value: 'price_asc', label: 'Price: low to high' },
  { value: 'price_desc', label: 'Price: high to low' },
  { value: 'name_asc', label: 'Name: A to Z' },
  { value: 'name_desc', label: 'Name: Z to A' },
];

export default function ProductListing({ title, description, categoryId, search, eyebrow }: Props) {
  const reload = useShopReload();
  const [products, setProducts] = useState<ProductCardModel[]>([]);
  const [meta, setMeta] = useState<{ current_page: number; last_page: number; total: number; per_page: number }>({
    current_page: 1,
    last_page: 1,
    total: 0,
    per_page: 12,
  });
  const [loading, setLoading] = useState(true);
  const [sort, setSort] = useState('newest');
  const [page, setPage] = useState(1);
  const [view, setView] = useState<'grid' | 'list'>('grid');
  const [attributes, setAttributes] = useState<Attribute[]>([]);
  const [selected, setSelected] = useState<Record<string, string[]>>({});
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [priceMin, setPriceMin] = useState('');
  const [priceMax, setPriceMax] = useState('');
  const [appliedPrice, setAppliedPrice] = useState<{ min: string; max: string }>({ min: '', max: '' });
  const [filterOpen, setFilterOpen] = useState(false);

  useEffect(() => {
    setPage(1);
    setSelected({});
    setAppliedPrice({ min: '', max: '' });
    setPriceMin('');
    setPriceMax('');
  }, [categoryId, search]);

  useEffect(() => {
    let cancelled = false;
    api
      .get(`categories/attributes${shopQuery()}`)
      .then(async (data) => {
        if (cancelled) return;
        const list: Attribute[] = Array.isArray(data) ? data.filter((attribute: Attribute) => attribute.type !== 'price') : [];
        setAttributes(list);
        const missing = list.filter((attribute) => attribute.type === 'select' && !attribute.options);
        if (!missing.length) return;
        const loaded = await Promise.all(
          missing.map(async (attribute) => {
            try {
              const options = await api.get(`categories/attributes/${attribute.id}/options${shopQuery()}`);
              return { ...attribute, options: options ?? [] };
            } catch {
              return attribute;
            }
          }),
        );
        if (cancelled) return;
        setAttributes((current) => current.map((attribute) => loaded.find((entry) => entry.id === attribute.id) ?? attribute));
      })
      .catch(() => {
        if (!cancelled) setAttributes([]);
      });
    return () => {
      cancelled = true;
    };
  }, [reload, categoryId]);

  useEffect(() => {
    setLoading(true);
    const params: Record<string, string | number> = {
      sort,
      page,
      per_page: 12,
    };
    if (categoryId) params.category_id = categoryId;
    if (search) params.query = search;
    if (appliedPrice.min) params.price_min = appliedPrice.min;
    if (appliedPrice.max) params.price_max = appliedPrice.max;
    Object.entries(selected).forEach(([code, values]) => {
      if (values.length) params[code] = values.join(',');
    });

    api
      .get(`products${shopQuery(params)}`)
      .then((data) => {
        setProducts((data?.data ?? []).filter((product: any) => product?.name));
        setMeta(data?.meta ?? { current_page: 1, last_page: 1, total: 0, per_page: 12 });
      })
      .catch(() => setProducts([]))
      .finally(() => setLoading(false));
  }, [reload, categoryId, search, sort, page, appliedPrice, selected]);

  const loadOptions = async (attribute: Attribute) => {
    try {
      const options = await api.get(`categories/attributes/${attribute.id}/options${shopQuery()}`);
      setAttributes((current) =>
        current.map((item) => (item.id === attribute.id ? { ...item, options: options ?? [] } : item)),
      );
    } catch {
      /* ignore */
    }
  };

  const toggleAttribute = (attribute: Attribute, option: AttributeOption) => {
    const value = String(option.id);
    setSelected((current) => {
      const existing = current[attribute.code] ?? [];
      const values = existing.includes(value) ? existing.filter((entry) => entry !== value) : [...existing, value];
      return { ...current, [attribute.code]: values };
    });
    setPage(1);
  };

  const clearFilters = () => {
    setSelected({});
    setAppliedPrice({ min: '', max: '' });
    setPriceMin('');
    setPriceMax('');
    setPage(1);
  };

  const activeFilters = useMemo(
    () => Object.values(selected).reduce((total, values) => total + values.length, 0) + (appliedPrice.min || appliedPrice.max ? 1 : 0),
    [selected, appliedPrice],
  );

  const pages = useMemo(() => {
    const total = meta.last_page;
    const current = meta.current_page;
    const set = new Set<number>([1, total, current, current - 1, current + 1, current - 2, current + 2]);
    return Array.from(set)
      .filter((value) => value >= 1 && value <= total)
      .sort((a, b) => a - b);
  }, [meta]);

  const filterPanel = (
    <aside className={`shop-filters ${filterOpen ? 'is-open' : ''}`}>
      <div className="shop-filter-group" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <p className="shop-filter-title" style={{ margin: 0 }}>
          {t('Filters')}
        </p>
        {activeFilters > 0 && (
          <button type="button" className="shop-link" style={{ fontSize: 12, background: 'none', border: 0, cursor: 'pointer' }} onClick={clearFilters}>
            {t('Clear all')}
          </button>
        )}
      </div>

      <div className="shop-filter-group">
        <p className="shop-filter-title">{t('Price')}</p>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <input
            className="filter-input"
            value={priceMin}
            onChange={(event) => setPriceMin(event.target.value)}
            placeholder={t('Min')}
            inputMode="decimal"
          />
          <span className="shop-muted">—</span>
          <input
            className="filter-input"
            value={priceMax}
            onChange={(event) => setPriceMax(event.target.value)}
            placeholder={t('Max')}
            inputMode="decimal"
          />
        </div>
        <button
          type="button"
          className="shop-secondary-button shop-button-sm"
          style={{ marginTop: 10, width: '100%' }}
          onClick={() => {
            setAppliedPrice({ min: priceMin, max: priceMax });
            setPage(1);
          }}
        >
          {t('Apply')}
        </button>
      </div>

      {attributes.map((attribute) => {
        const isBoolean = attribute.type === 'boolean';
        const isSelect = attribute.type === 'select';
        const values = selected[attribute.code] ?? [];
        return (
          <div className="shop-filter-group" key={attribute.id}>
            <button
              type="button"
              className="filter-heading"
              onClick={() => setExpanded((current) => ({ ...current, [attribute.code]: !current[attribute.code] }))}
            >
              <span className="shop-filter-title" style={{ margin: 0 }}>
                {t(attribute.admin_name)}
              </span>
              <IconChevronDown
                size={16}
                style={{ transform: expanded[attribute.code] ? 'rotate(180deg)' : undefined, transition: 'transform .2s' }}
              />
            </button>

            {(expanded[attribute.code] ?? true) && (
              <div style={{ display: 'grid', gap: 8, marginTop: 10 }}>
                {isBoolean && (
                  <label className="filter-check">
                    <input
                      type="checkbox"
                      checked={values.includes('1')}
                      onChange={() =>
                        setSelected((current) => ({
                          ...current,
                          [attribute.code]: values.includes('1') ? [] : ['1'],
                        }))
                      }
                    />
                    <span>{t(attribute.admin_name)}</span>
                  </label>
                )}

                {isSelect && (
                  <div style={{ display: 'grid', gap: 8 }}>
                    {(attribute.options ?? []).map((option) => (
                      <label className="filter-check" key={option.id}>
                        <input
                          type="checkbox"
                          checked={values.includes(String(option.id))}
                          onChange={() => toggleAttribute(attribute, option)}
                        />
                        {option.swatch_value && (
                          <span
                            style={{
                              width: 14,
                              height: 14,
                              borderRadius: 999,
                              background: option.swatch_value,
                              border: '1px solid #d4d4d8',
                              display: 'inline-block',
                            }}
                          />
                        )}
                        <span>{t(option.admin_name)}</span>
                      </label>
                    ))}
                    {!attribute.options && (
                      <button
                        type="button"
                        className="shop-link"
                        style={{ fontSize: 12, background: 'none', border: 0, textAlign: 'left', cursor: 'pointer', padding: 0 }}
                        onClick={() => loadOptions(attribute)}
                      >
                        {t('Load options')}
                      </button>
                    )}
                  </div>
                )}

                {!isBoolean && !isSelect && !attribute.options && (
                  <span className="shop-muted" style={{ fontSize: 12 }}>
                    {t('No available options')}
                  </span>
                )}
              </div>
            )}
          </div>
        );
      })}
    </aside>
  );

  return (
    <div className="shop-container" style={{ paddingTop: 34 }}>
      <div className="listing-head">
        <div>
          {eyebrow && <span className="listing-eyebrow">{t(eyebrow)}</span>}
          <h1 className="shop-heading" style={{ fontSize: 38 }}>
            {t(title)}
          </h1>
          {description && (
            <p className="shop-muted" style={{ maxWidth: 680, marginTop: 10, lineHeight: 1.65 }}>
              {t(description)}
            </p>
          )}
        </div>
        <span className="listing-count">
          {t(meta.total === 1 ? '{count} product' : '{count} products', { count: meta.total })}
        </span>
      </div>

      <div className="shop-listing">
        <div className="shop-filters-desktop">{filterPanel}</div>

        <div style={{ flex: 1, minWidth: 0 }}>
          <div className="shop-toolbar">
            <div style={{ display: 'flex', gap: 10 }}>
              <button type="button" className="toolbar-pill filter-toggle" onClick={() => setFilterOpen((open) => !open)}>
                <IconFilter size={16} /> {t('Filters')} {activeFilters > 0 ? `(${activeFilters})` : ''}
              </button>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <div className="view-toggle">
                <button type="button" data-active={view === 'grid'} onClick={() => setView('grid')} aria-label={t('Grid view')}>
                  <IconGridDots size={17} />
                </button>
                <button type="button" data-active={view === 'list'} onClick={() => setView('list')} aria-label={t('List view')}>
                  <IconList size={17} />
                </button>
              </div>

              <label className="toolbar-pill" style={{ gap: 8 }}>
                <span className="shop-muted">{t('Sort by')}</span>
                <select value={sort} onChange={(event) => { setSort(event.target.value); setPage(1); }} className="toolbar-select">
                  {SORTS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {t(option.label)}
                    </option>
                  ))}
                </select>
              </label>
            </div>
          </div>

          {filterOpen && <div className="shop-filters-mobile">{filterPanel}</div>}

          {loading ? (
            <div className="listing-loading">
              {Array.from({ length: 6 }).map((_, index) => (
                <div className="shimmer-card" key={index} />
              ))}
            </div>
          ) : products.length === 0 ? (
            <div className="listing-empty">
              <IconSearch size={38} />
              <h3>{t('No products found')}</h3>
              <p>{t('Try adjusting your filters or search for something else.')}</p>
              {activeFilters > 0 && (
                <button type="button" className="shop-secondary-button shop-button-sm" onClick={clearFilters}>
                  {t('Clear filters')}
                </button>
              )}
            </div>
          ) : view === 'grid' ? (
            <div className="shop-grid" style={{ marginTop: 26 }}>
              {products.map((product) => (
                <ProductCard product={product} key={product.id} />
              ))}
            </div>
          ) : (
            <div style={{ display: 'grid', gap: 26, marginTop: 26 }}>
              {products.map((product) => (
                <ProductCard product={product} key={product.id} list />
              ))}
            </div>
          )}

          {meta.last_page > 1 && (
            <nav className="shop-pagination" aria-label={t('Pagination')}>
              <button type="button" disabled={meta.current_page <= 1} onClick={() => setPage((current) => Math.max(1, current - 1))}>
                {t('Previous')}
              </button>
              {pages.map((value, index) => {
                const previous = pages[index - 1];
                return (
                  <span key={value} style={{ display: 'contents' }}>
                    {previous && value - previous > 1 && <span className="pagination-gap">…</span>}
                    <button type="button" data-active={value === meta.current_page} onClick={() => setPage(value)}>
                      {value}
                    </button>
                  </span>
                );
              })}
              <button
                type="button"
                disabled={meta.current_page >= meta.last_page}
                onClick={() => setPage((current) => Math.min(meta.last_page, current + 1))}
              >
                {t('Next')}
              </button>
            </nav>
          )}
        </div>
      </div>
    </div>
  );
}
