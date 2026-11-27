import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import {
  IconCheck,
  IconPlus,
  IconTrash,
  IconUpload,
  IconX,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { mediaUrl, money } from '../../lib/format';
import { AdminPageHeader, AdminTabs } from '../../components/admin/ui';
import { t } from '../../i18n';

interface AdminAttribute {
  id: number;
  code: string;
  admin_name: string;
  type: string;
  options?: { id: number; admin_name: string }[];
}

const TABS = [
  { key: 'general', label: 'General' },
  { key: 'price', label: 'Price' },
  { key: 'inventory', label: 'Inventory' },
  { key: 'images', label: 'Images' },
  { key: 'categories', label: 'Categories' },
  { key: 'variants', label: 'Variants' },
  { key: 'seo', label: 'SEO' },
];

const FIXED = ['sku', 'name', 'url_key', 'price', 'special_price', 'cost', 'special_price_from', 'special_price_to', 'weight', 'length', 'width', 'height', 'meta_title', 'description', 'short_description', 'status', 'new', 'visible_individually', 'featured', 'manage_stock', 'tax_category_id'];

export default function ProductEditorPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const isCreate = !id || id === 'create';
  const [tab, setTab] = useState('general');
  const [loading, setLoading] = useState(!isCreate);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  const [productType, setProductType] = useState('simple');
  const [sku, setSku] = useState('');
  const [attributes, setAttributes] = useState<Record<string, any>>({});
  const [categories, setCategories] = useState<number[]>([]);
  const [images, setImages] = useState<{ path: string; id?: number }[]>([]);
  const [inventories, setInventories] = useState<Record<string, number>>({});
  const [familyId, setFamilyId] = useState(1);
  const [superAttributes, setSuperAttributes] = useState<any[]>([]);
  const [variants, setVariants] = useState<any[]>([]);
  const [links, setLinks] = useState<any>({ related: [], up_sells: [], cross_sells: [] });

  const [categoryTree, setCategoryTree] = useState<any[]>([]);
  const [attrDefs, setAttrDefs] = useState<AdminAttribute[]>([]);
  const [families, setFamilies] = useState<any[]>([]);
  const [sources, setSources] = useState<any[]>([]);
  const [variantForm, setVariantForm] = useState<Record<string, any>>({ sku: '', price: '', qty: '', name: '' });

  const reload = async (targetId = id!) => {
    setLoading(true);
    try {
      const data = await api.adminGet(`admin/catalog/products/edit/${targetId}`);
      setProductType(data.product?.type ?? 'simple');
      setSku(data.product?.sku ?? data.attributes?.sku ?? '');
      setAttributes(data.attributes ?? {});
      setCategories((data.categories ?? []).map((entry: any) => entry.category_id ?? entry.id).filter(Boolean));
      setImages((data.images ?? []).map((image: any) => ({ path: image.path, id: image.id })));
      const inv: Record<string, number> = {};
      (data.inventories ?? []).forEach((entry: any) => {
        inv[String(entry.inventory_source_id)] = entry.qty ?? 0;
      });
      setInventories(inv);
      setFamilyId(data.product?.attribute_family_id ?? 1);
      setSuperAttributes(data.super_attributes ?? []);
      setVariants(data.variants ?? []);
      setLinks(data.links ?? { related: [], up_sells: [], cross_sells: [] });
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to load the product.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!isCreate) reload();
    api.adminGet('admin/catalog/categories/tree').then((data) => setCategoryTree(Array.isArray(data) ? data : [])).catch(() => undefined);
    api.adminGet('admin/catalog/families').then((data) => setFamilies(Array.isArray(data) ? data : [])).catch(() => undefined);
    api.adminGet('admin/settings/inventory-sources').then((data) => setSources(Array.isArray(data) ? data : [])).catch(() => undefined);
    api
      .adminGet('admin/catalog/attributes/all')
      .then(async (data) => {
        const list: AdminAttribute[] = Array.isArray(data) ? data : [];
        const enriched = await Promise.all(
          list
            .filter((attribute) => !FIXED.includes(attribute.code))
            .map(async (attribute) => {
              if (attribute.type !== 'select') return attribute;
              try {
                const payload = await api.adminGet(`admin/catalog/attributes/edit/${attribute.id}`);
                return { ...attribute, options: payload?.options ?? [] };
              } catch {
                return attribute;
              }
            }),
        );
        setAttrDefs(enriched);
      })
      .catch(() => undefined);
  }, []);

  const extraAttributes = useMemo(() => attrDefs.filter((attribute) => !FIXED.includes(attribute.code)), [attrDefs]);

  const setAttribute = (code: string, value: any) => setAttributes((current) => ({ ...current, [code]: value }));

  const save = async (redirectAfter = false) => {
    setError('');
    setMessage('');
    if (!sku.trim()) {
      setError(t('The SKU field is required.'));
      setTab('general');
      return;
    }
    setSaving(true);
    try {
      let productId = id;
      if (isCreate) {
        const created = await api.adminPost('admin/catalog/products/create', {
          sku: sku.trim(),
          type: productType,
          attribute_family_id: familyId,
        });
        productId = String(created?.id);
      }
      const payloadAttributes: Record<string, any> = { ...attributes, sku: sku.trim() };
      delete payloadAttributes.categories;
      await api.adminPut(`admin/catalog/products/edit/${productId}`, {
        sku: sku.trim(),
        attributes: payloadAttributes,
        categories: categories.map((categoryId) => ({ id: categoryId })),
        images: images.map((image) => ({ path: image.path })),
        inventories,
        links,
      });
      setMessage(t('The product has been saved successfully.'));
      if (isCreate && productId) {
        if (redirectAfter) navigate(`/admin/catalog/products/edit/${productId}`);
        else {
          await reload(productId);
          navigate(`/admin/catalog/products/edit/${productId}`, { replace: true });
        }
      } else {
        await reload(productId!);
      }
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to save the product.'));
    } finally {
      setSaving(false);
    }
  };

  const uploadImages = async (files: FileList | null) => {
    if (!files?.length) return;
    setError('');
    for (const file of Array.from(files)) {
      try {
        const result = await api.upload(file, 'products');
        if (result?.path) setImages((current) => [...current, { path: result.path }]);
        else if (result?.url) setImages((current) => [...current, { path: String(result.url).replace(/^\/?storage\//, '') }]);
      } catch (requestError: any) {
        setError(requestError?.message ?? t('Unable to upload the image.'));
      }
    }
  };

  const addVariant = async () => {
    setError('');
    if (!variantForm.sku) {
      setError(t('The variant SKU is required.'));
      return;
    }
    const variantAttributes: Record<string, any> = {};
    superAttributes.forEach((attribute) => {
      if (variantForm[attribute.code]) variantAttributes[attribute.code] = Number(variantForm[attribute.code]);
    });
    try {
      await api.adminPost(`admin/catalog/products/edit/${id}/variants`, {
        sku: variantForm.sku,
        attributes: {
          ...variantAttributes,
          ...(variantForm.name ? { name: variantForm.name } : {}),
          ...(variantForm.price ? { price: Number(variantForm.price) } : {}),
        },
        qty: variantForm.qty ? Number(variantForm.qty) : 0,
      });
      setVariantForm({ sku: '', price: '', qty: '', name: '' });
      await reload();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to add the variant.'));
    }
  };

  const deleteVariant = async (variantId: number) => {
    if (!window.confirm(t('Delete this variant?'))) return;
    try {
      await api.adminDelete(`admin/catalog/products/variants/${variantId}`);
      await reload();
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to delete the variant.'));
    }
  };

  if (loading) {
    return (
      <div>
        <AdminPageHeader title={t('Edit product')} backTo="/admin/catalog/products" />
        <div className="admin-card admin-card-pad">
          <div className="admin-skeleton-row" style={{ height: 40, marginBottom: 14 }} />
          <div className="admin-skeleton-row" style={{ height: 40, marginBottom: 14 }} />
          <div className="admin-skeleton-row" style={{ height: 140 }} />
        </div>
      </div>
    );
  }

  return (
    <div>
      <AdminPageHeader
        title={isCreate ? t('Create product') : attributes.name ?? t('Edit product')}
        subtitle={isCreate ? t('Add a new product to your catalog.') : t('SKU: {sku} · Type: {type}', { sku, type: t(productType) })}
        backTo="/admin/catalog/products"
        actions={
          <>
            <button type="button" className="admin-button" onClick={() => save(true)} disabled={saving}>
              {t('Save & continue')}
            </button>
            <button type="button" className="admin-button primary" onClick={() => save(false)} disabled={saving}>
              <IconCheck size={15} /> {saving ? t('Saving…') : t('Save product')}
            </button>
          </>
        }
      />

      {message && <div className="admin-alert success">{message}</div>}
      {error && <div className="admin-alert danger">{error}</div>}

      <div className="admin-card">
        <AdminTabs tabs={TABS} active={tab} onChange={setTab} />
        <div className="admin-card-pad">
          {tab === 'general' && (
            <div className="product-editor-grid">
              <div>
                <div className="admin-form-grid">
                  <label className="admin-field span-2">
                    <span className="admin-field-label">{t('Product name')} *</span>
                    <input
                      className="admin-input"
                      value={attributes.name ?? ''}
                      onChange={(event) => setAttribute('name', event.target.value)}
                      placeholder={t('e.g. Running Sneaker')}
                    />
                  </label>
                  <label className="admin-field">
                    <span className="admin-field-label">{t('SKU')} *</span>
                    <input className="admin-input" value={sku} onChange={(event) => setSku(event.target.value)} placeholder="EZ-SKU-01" />
                  </label>
                  <label className="admin-field">
                    <span className="admin-field-label">{t('URL key')}</span>
                    <input
                      className="admin-input"
                      value={attributes.url_key ?? ''}
                      onChange={(event) => setAttribute('url_key', event.target.value)}
                      placeholder={t('product-url-key')}
                    />
                  </label>
                  <label className="admin-field">
                    <span className="admin-field-label">{t('Product type')}</span>
                    <select className="admin-input" value={productType} onChange={(event) => setProductType(event.target.value)} disabled={!isCreate}>
                      {['simple', 'configurable', 'virtual', 'downloadable', 'grouped', 'bundle', 'booking'].map((type) => (
                        <option key={type} value={type} style={{ textTransform: 'capitalize' }}>
                          {t(type)}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="admin-field">
                    <span className="admin-field-label">{t('Attribute family')}</span>
                    <select className="admin-input" value={familyId} onChange={(event) => setFamilyId(Number(event.target.value))} disabled={!isCreate}>
                      {families.map((family) => (
                        <option key={family.id} value={family.id}>
                          {family.name}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>

                <label className="admin-field" style={{ marginTop: 16 }}>
                  <span className="admin-field-label">{t('Short description')}</span>
                  <textarea
                    className="admin-input"
                    rows={3}
                    value={attributes.short_description ?? ''}
                    onChange={(event) => setAttribute('short_description', event.target.value)}
                  />
                </label>

                <label className="admin-field" style={{ marginTop: 16 }}>
                  <span className="admin-field-label">{t('Description')}</span>
                  <textarea
                    className="admin-input"
                    rows={7}
                    value={attributes.description ?? ''}
                    onChange={(event) => setAttribute('description', event.target.value)}
                  />
                </label>

                <div className="admin-form-grid" style={{ marginTop: 16 }}>
                  {extraAttributes.map((attribute) => (
                    <label className="admin-field" key={attribute.code}>
                      <span className="admin-field-label">{attribute.admin_name}</span>
                      {attribute.type === 'select' ? (
                        <select
                          className="admin-input"
                          value={attributes[attribute.code] ?? ''}
                          onChange={(event) => setAttribute(attribute.code, event.target.value === '' ? null : Number(event.target.value))}
                        >
                          <option value="">{t('Select {option}', { option: attribute.admin_name })}</option>
                          {(attribute.options ?? []).map((option) => (
                            <option key={option.id} value={option.id}>
                              {option.admin_name}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <input
                          className="admin-input"
                          value={attributes[attribute.code] ?? ''}
                          onChange={(event) => setAttribute(attribute.code, event.target.value)}
                        />
                      )}
                    </label>
                  ))}
                </div>
              </div>

              <aside className="product-editor-side">
                <div className="admin-side-card">
                  <h3>{t('Status')}</h3>
                  <label className="admin-switch-row">
                    <span>{t('Active')}</span>
                    <input
                      type="checkbox"
                      checked={Number(attributes.status ?? 1) === 1}
                      onChange={(event) => setAttribute('status', event.target.checked ? 1 : 0)}
                    />
                  </label>
                  <label className="admin-switch-row">
                    <span>{t('New')}</span>
                    <input
                      type="checkbox"
                      checked={Number(attributes.new ?? 0) === 1}
                      onChange={(event) => setAttribute('new', event.target.checked ? 1 : 0)}
                    />
                  </label>
                  <label className="admin-switch-row">
                    <span>{t('Featured')}</span>
                    <input
                      type="checkbox"
                      checked={Number(attributes.featured ?? 0) === 1}
                      onChange={(event) => setAttribute('featured', event.target.checked ? 1 : 0)}
                    />
                  </label>
                  <label className="admin-switch-row">
                    <span>{t('Visible individually')}</span>
                    <input
                      type="checkbox"
                      checked={Number(attributes.visible_individually ?? 1) === 1}
                      onChange={(event) => setAttribute('visible_individually', event.target.checked ? 1 : 0)}
                    />
                  </label>
                </div>

                <div className="admin-side-card">
                  <h3>{t('Shipping')}</h3>
                  <label className="admin-switch-row">
                    <span>{t('Manage stock')}</span>
                    <input
                      type="checkbox"
                      checked={Number(attributes.manage_stock ?? 1) === 1}
                      onChange={(event) => setAttribute('manage_stock', event.target.checked ? 1 : 0)}
                    />
                  </label>
                  <div className="admin-form-grid">
                    <label className="admin-field">
                      <span className="admin-field-label">{t('Weight')}</span>
                      <input className="admin-input" value={attributes.weight ?? ''} onChange={(event) => setAttribute('weight', event.target.value)} />
                    </label>
                    <label className="admin-field">
                      <span className="admin-field-label">{t('Length')}</span>
                      <input className="admin-input" value={attributes.length ?? ''} onChange={(event) => setAttribute('length', event.target.value)} />
                    </label>
                    <label className="admin-field">
                      <span className="admin-field-label">{t('Width')}</span>
                      <input className="admin-input" value={attributes.width ?? ''} onChange={(event) => setAttribute('width', event.target.value)} />
                    </label>
                    <label className="admin-field">
                      <span className="admin-field-label">{t('Height')}</span>
                      <input className="admin-input" value={attributes.height ?? ''} onChange={(event) => setAttribute('height', event.target.value)} />
                    </label>
                  </div>
                </div>
              </aside>
            </div>
          )}

          {tab === 'price' && (
            <div style={{ maxWidth: 640 }}>
              <div className="admin-form-grid">
                <label className="admin-field">
                  <span className="admin-field-label">{t('Price')} *</span>
                  <input
                    type="number"
                    step="0.01"
                    className="admin-input"
                    value={attributes.price ?? ''}
                    onChange={(event) => setAttribute('price', event.target.value)}
                    placeholder="0.00"
                  />
                </label>
                <label className="admin-field">
                  <span className="admin-field-label">{t('Cost')}</span>
                  <input
                    type="number"
                    step="0.01"
                    className="admin-input"
                    value={attributes.cost ?? ''}
                    onChange={(event) => setAttribute('cost', event.target.value)}
                  />
                </label>
                <label className="admin-field">
                  <span className="admin-field-label">{t('Special price')}</span>
                  <input
                    type="number"
                    step="0.01"
                    className="admin-input"
                    value={attributes.special_price ?? ''}
                    onChange={(event) => setAttribute('special_price', event.target.value)}
                  />
                </label>
                <label className="admin-field">
                  <span className="admin-field-label">{t('Tax category')}</span>
                  <input
                    className="admin-input"
                    value={attributes.tax_category_id ?? ''}
                    onChange={(event) => setAttribute('tax_category_id', event.target.value)}
                  />
                </label>
                <label className="admin-field">
                  <span className="admin-field-label">{t('Special price from')}</span>
                  <input
                    type="date"
                    className="admin-input"
                    value={attributes.special_price_from ?? ''}
                    onChange={(event) => setAttribute('special_price_from', event.target.value)}
                  />
                </label>
                <label className="admin-field">
                  <span className="admin-field-label">{t('Special price to')}</span>
                  <input
                    type="date"
                    className="admin-input"
                    value={attributes.special_price_to ?? ''}
                    onChange={(event) => setAttribute('special_price_to', event.target.value)}
                  />
                </label>
              </div>
              <p className="admin-muted" style={{ fontSize: 13, marginTop: 14 }}>
                {t('When a special price is set, the storefront shows it as a sale price with the regular price struck through.')}
              </p>
            </div>
          )}

          {tab === 'inventory' && (
            <div style={{ maxWidth: 640 }}>
              <p className="admin-muted" style={{ fontSize: 13, marginTop: 0 }}>
                {t('Quantities are tracked per inventory source. The default source is used for the storefront availability.')}
              </p>
              <div className="admin-form-grid">
                {sources.map((source) => (
                  <label className="admin-field" key={source.id}>
                    <span className="admin-field-label">{source.name}</span>
                    <input
                      type="number"
                      className="admin-input"
                      value={inventories[String(source.id)] ?? 0}
                      onChange={(event) =>
                        setInventories((current) => ({ ...current, [String(source.id)]: Number(event.target.value) }))
                      }
                    />
                    <span className="admin-field-hint">{t('Code:')} {source.code}</span>
                  </label>
                ))}
              </div>
            </div>
          )}

          {tab === 'images' && (
            <div>
              <div className="admin-upload-grid">
                {images.map((image, index) => (
                  <div className="admin-upload-tile" key={`${image.path}-${index}`}>
                    <img src={mediaUrl(image.path)} alt={`Product ${index + 1}`} />
                    <button
                      type="button"
                      className="admin-upload-remove"
                      onClick={() => setImages((current) => current.filter((_, position) => position !== index))}
                      aria-label={t('Remove image')}
                    >
                      <IconX size={14} />
                    </button>
                    {index === 0 && <span className="admin-upload-primary">{t('Base')}</span>}
                  </div>
                ))}

                <label className="admin-upload-add">
                  <IconUpload size={22} />
                  <span>{t('Upload images')}</span>
                  <input type="file" accept="image/*" multiple hidden onChange={(event) => uploadImages(event.target.files)} />
                </label>
              </div>
              <p className="admin-muted" style={{ fontSize: 12, marginTop: 12 }}>
                {t('Upload JPG, PNG, WebP, AVIF, GIF, BMP or TIFF. Images are resized to 160/360/720/1200 px and served as AVIF with JPEG/PNG fallback. The first image is the base image.')}
              </p>
            </div>
          )}

          {tab === 'categories' && (
            <div style={{ maxWidth: 640 }}>
              <p className="admin-muted" style={{ fontSize: 13, marginTop: 0 }}>
                {t('Assign this product to one or more categories.')}
              </p>
              <div className="admin-category-picker">
                {categoryTree.map((category) => (
                  <CategoryChoice
                    key={category.id}
                    category={category}
                    selected={categories}
                    onChange={(next) => setCategories(next)}
                  />
                ))}
              </div>
            </div>
          )}

          {tab === 'variants' && (
            <div>
              {productType !== 'configurable' ? (
                <div className="admin-empty">
                  <h3>{t('This product is not configurable')}</h3>
                  <p>{t('Create a configurable product to manage variants such as colours and sizes.')}</p>
                </div>
              ) : isCreate ? (
                <div className="admin-empty">
                  <h3>{t('Save the product first')}</h3>
                  <p>{t('After creating the configurable product you can add variants.')}</p>
                </div>
              ) : (
                <>
                  <table className="admin-table" style={{ minWidth: 620 }}>
                    <thead>
                      <tr>
                        <th>{t('SKU')}</th>
                        {superAttributes.map((attribute) => (
                          <th key={attribute.code}>{attribute.admin_name}</th>
                        ))}
                        <th>{t('Quantity')}</th>
                        <th>{t('Price')}</th>
                        <th className="actions">{t('Action')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {variants.length === 0 ? (
                        <tr>
                          <td colSpan={superAttributes.length + 4}>
                            <div className="admin-empty">{t('No variants yet.')}</div>
                          </td>
                        </tr>
                      ) : (
                        variants.map((variant) => (
                          <tr key={variant.id}>
                            <td>
                              <span className="admin-code">{variant.sku}</span>
                            </td>
                            {superAttributes.map((attribute) => {
                              const optionId = variant.attributes?.[attribute.code];
                              const option = attribute.options?.find((entry: any) => Number(entry.id) === Number(optionId));
                              return <td key={attribute.code}>{option?.admin_name ?? optionId ?? '—'}</td>;
                            })}
                            <td>{variant.qty ?? 0}</td>
                            <td>{money(Number(variant.attributes?.price ?? 0))}</td>
                            <td className="actions">
                              <button type="button" className="admin-link danger" onClick={() => deleteVariant(variant.id)}>
                                {t('Delete')}
                              </button>
                            </td>
                          </tr>
                        ))
                      )}
                    </tbody>
                  </table>

                  <div className="admin-inline-form">
                    <h3>{t('Add variant')}</h3>
                    <div className="admin-form-grid">
                      <label className="admin-field">
                        <span className="admin-field-label">{t('SKU')} *</span>
                        <input
                          className="admin-input"
                          value={variantForm.sku}
                          onChange={(event) => setVariantForm({ ...variantForm, sku: event.target.value })}
                          placeholder={`${sku}-NEW`}
                        />
                      </label>
                      <label className="admin-field">
                        <span className="admin-field-label">{t('Name')}</span>
                        <input
                          className="admin-input"
                          value={variantForm.name}
                          onChange={(event) => setVariantForm({ ...variantForm, name: event.target.value })}
                          placeholder={t('Optional variant name')}
                        />
                      </label>
                      {superAttributes.map((attribute) => (
                        <label className="admin-field" key={attribute.code}>
                          <span className="admin-field-label">{attribute.admin_name}</span>
                          <select
                            className="admin-input"
                            value={variantForm[attribute.code] ?? ''}
                            onChange={(event) => setVariantForm({ ...variantForm, [attribute.code]: event.target.value })}
                          >
                            <option value="">{t('Select')}</option>
                            {(attribute.options ?? []).map((option: any) => (
                              <option key={option.id} value={option.id}>
                                {option.admin_name}
                              </option>
                            ))}
                          </select>
                        </label>
                      ))}
                      <label className="admin-field">
                        <span className="admin-field-label">{t('Price')}</span>
                        <input
                          type="number"
                          step="0.01"
                          className="admin-input"
                          value={variantForm.price}
                          onChange={(event) => setVariantForm({ ...variantForm, price: event.target.value })}
                        />
                      </label>
                      <label className="admin-field">
                        <span className="admin-field-label">{t('Quantity')}</span>
                        <input
                          type="number"
                          className="admin-input"
                          value={variantForm.qty}
                          onChange={(event) => setVariantForm({ ...variantForm, qty: event.target.value })}
                        />
                      </label>
                    </div>
                    <button type="button" className="admin-button primary" onClick={addVariant} style={{ marginTop: 14 }}>
                      <IconPlus size={15} /> {t('Add variant')}
                    </button>
                  </div>
                </>
              )}
            </div>
          )}

          {tab === 'seo' && (
            <div style={{ maxWidth: 640 }}>
              <label className="admin-field">
                <span className="admin-field-label">{t('URL key')}</span>
                <input
                  className="admin-input"
                  value={attributes.url_key ?? ''}
                  onChange={(event) => setAttribute('url_key', event.target.value)}
                />
                <span className="admin-field-hint">{t('The storefront path, for example /product/running-sneaker.')}</span>
              </label>
              <label className="admin-field" style={{ marginTop: 16 }}>
                <span className="admin-field-label">{t('Meta title')}</span>
                <input
                  className="admin-input"
                  value={attributes.meta_title ?? ''}
                  onChange={(event) => setAttribute('meta_title', event.target.value)}
                />
              </label>
              <label className="admin-field" style={{ marginTop: 16 }}>
                <span className="admin-field-label">{t('Meta description')}</span>
                <textarea
                  className="admin-input"
                  rows={3}
                  value={attributes.meta_description ?? ''}
                  onChange={(event) => setAttribute('meta_description', event.target.value)}
                />
              </label>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function CategoryChoice({ category, selected, onChange }: { category: any; selected: number[]; onChange: (next: number[]) => void }) {
  return (
    <div className="admin-category-choice">
      <label>
        <input
          type="checkbox"
          checked={selected.includes(category.id)}
          onChange={() =>
            onChange(selected.includes(category.id) ? selected.filter((id) => id !== category.id) : [...selected, category.id])
          }
        />
        <span>{category.name}</span>
        <span className="admin-muted admin-small">#{category.id}</span>
      </label>
      {category.children?.length ? (
        <div style={{ paddingLeft: 22 }}>
          {category.children.map((child: any) => (
            <CategoryChoice key={child.id} category={child} selected={selected} onChange={onChange} />
          ))}
        </div>
      ) : null}
    </div>
  );
}
