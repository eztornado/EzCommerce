import { useEffect, useMemo, useState } from 'react';
import {
  IconBuildingStore,
  IconBox,
  IconUsers,
  IconShoppingCart,
  IconCreditCard,
  IconCheck,
  IconPhoto,
  IconTrash,
  IconUpload,
} from '@tabler/icons-react';
import { api } from '../../api/client';
import { AdminPageHeader } from '../../components/admin/ui';
import { useShop } from '../../store';
import { t } from '../../i18n';
import { mediaUrl } from '../../lib/format';

type FieldType = 'text' | 'number' | 'select' | 'boolean' | 'textarea' | 'image';

interface ConfigField {
  code: string;
  label: string;
  type: FieldType;
  options?: { label: string; value: string }[];
  placeholder?: string;
  hint?: string;
  accept?: string;
}

interface ConfigGroup {
  title: string;
  info?: string;
  fields: ConfigField[];
}

interface ConfigSection {
  key: string;
  label: string;
  icon: React.ReactNode;
  title: string;
  info: string;
  groups: ConfigGroup[];
}

const SECTIONS: ConfigSection[] = [
  {
    key: 'general',
    label: 'General',
    icon: <IconBuildingStore size={17} />,
    title: 'General',
    info: 'Store identity, content shown in the storefront header and footer, and design assets.',
    groups: [
      {
        title: 'Store',
        info: 'Storefront language, measurement units and breadcrumb defaults.',
        fields: [
          {
            code: 'general.general.locale_options.ui_locale',
            label: 'Language',
            type: 'select',
            hint: 'Interface language for the storefront and the admin panel.',
            options: [
              { label: 'English', value: 'en' },
              { label: 'Español', value: 'es' },
              { label: '日本語', value: 'ja' },
            ],
          },
          {
            code: 'general.general.locale_options.weight_unit',
            label: 'Weight unit',
            type: 'select',
            options: [
              { label: 'lbs', value: 'lbs' },
              { label: 'kgs', value: 'kgs' },
            ],
          },
          {
            code: 'general.general.breadcrumbs.shop',
            label: 'Display shop breadcrumbs',
            type: 'boolean',
            hint: 'Show breadcrumbs on storefront category and product pages.',
          },
        ],
      },
      {
        title: 'Header offer',
        info: 'The promotional strip shown at the very top of every storefront page.',
        fields: [
          { code: 'general.content.header_offer.title', label: 'Offer title', type: 'text', placeholder: 'Free shipping on all orders over €100' },
          { code: 'general.content.header_offer.redirection_title', label: 'Link title', type: 'text', placeholder: 'Shop now' },
          { code: 'general.content.header_offer.redirection_link', label: 'Link URL', type: 'text', placeholder: '/products' },
        ],
      },
      {
        title: 'Footer',
        info: 'Copyright line rendered at the bottom of the storefront.',
        fields: [
          { code: 'general.content.footer.copyright_content', label: 'Copyright content', type: 'text', placeholder: '© 2026 EzCommerce. All rights reserved.' },
        ],
      },
      {
        title: 'Design',
        info: 'Brand assets used by the storefront and the admin panel.',
        fields: [
          {
            code: 'general.design.admin_logo.logo_image',
            label: 'Logo',
            type: 'image',
            accept: '.svg,.png,.jpg,.jpeg,.webp',
            hint: 'Upload the storefront and admin logo. PNG, JPG, SVG or WebP. Max 8 MB.',
          },
          {
            code: 'general.design.admin_logo.favicon',
            label: 'Favicon',
            type: 'image',
            accept: '.ico,.png,.svg',
            hint: 'Upload the browser favicon. ICO, PNG or SVG. Max 8 MB.',
          },
        ],
      },
    ],
  },
  {
    key: 'catalog',
    label: 'Catalog',
    icon: <IconBox size={17} />,
    title: 'Catalog',
    info: 'Product listing behaviour, storefront sorting and inventory defaults.',
    groups: [
      {
        title: 'Product settings',
        info: 'Storefront options for product comparison and search.',
        fields: [
          { code: 'catalog.products.settings.compare_option', label: 'Enable product comparison', type: 'boolean' },
          { code: 'catalog.products.settings.image_search', label: 'Enable image search', type: 'boolean' },
          {
            code: 'catalog.products.settings.enable_flat',
            label: 'Enable flat table index',
            type: 'boolean',
            hint: 'Improves listing performance for large catalogs.',
          },
        ],
      },
      {
        title: 'Storefront',
        info: 'Default presentation of product listings.',
        fields: [
          {
            code: 'catalog.products.storefront.mode',
            label: 'Default list mode',
            type: 'select',
            options: [
              { label: 'Grid', value: 'grid' },
              { label: 'List', value: 'list' },
            ],
          },
          { code: 'catalog.products.storefront.products_per_page', label: 'Products per page', type: 'text', placeholder: '9, 15, 21, 30', hint: 'Comma separated list of page sizes.' },
          {
            code: 'catalog.products.storefront.sort_by',
            label: 'Default sort by',
            type: 'select',
            options: [
              { label: 'Name A–Z', value: 'name-asc' },
              { label: 'Name Z–A', value: 'name-desc' },
              { label: 'Latest first', value: 'created_at-desc' },
              { label: 'Oldest first', value: 'created_at-asc' },
              { label: 'Cheapest first', value: 'price-asc' },
              { label: 'Most expensive first', value: 'price-desc' },
            ],
          },
          { code: 'catalog.products.home_page.out_of_stock_items', label: 'Show out of stock products on home page', type: 'boolean' },
        ],
      },
      {
        title: 'Inventory',
        info: 'Stock display rules.',
        fields: [
          { code: 'catalog.inventory.stock_options.show_out_of_stock_products', label: 'Show out of stock products', type: 'boolean' },
          { code: 'catalog.inventory.stock_options.only_x_left_in_stock', label: 'Show “only X left” hint', type: 'boolean' },
        ],
      },
    ],
  },
  {
    key: 'customer',
    label: 'Customer',
    icon: <IconUsers size={17} />,
    title: 'Customer',
    info: 'Account creation, email verification, login redirection and newsletter options.',
    groups: [
      {
        title: 'Create new account options',
        info: 'Defaults applied when a customer registers.',
        fields: [
          {
            code: 'customer.settings.create_new_account_options.default_group',
            label: 'Default customer group',
            type: 'select',
            options: [
              { label: 'General', value: 'general' },
              { label: 'Guest', value: 'guest' },
              { label: 'Wholesale', value: 'wholesale' },
            ],
          },
          { code: 'customer.settings.create_new_account_options.news_letter', label: 'Subscribe to newsletter by default', type: 'boolean' },
        ],
      },
      {
        title: 'Email',
        fields: [{ code: 'customer.settings.email.verification', label: 'Require email verification', type: 'boolean' }],
      },
      {
        title: 'Login options',
        fields: [
          {
            code: 'customer.settings.login_options.redirected_to_page',
            label: 'Redirect customers to',
            type: 'select',
            options: [
              { label: 'Home page', value: 'home' },
              { label: 'Account dashboard', value: 'account' },
            ],
          },
        ],
      },
      {
        title: 'Wishlist',
        fields: [{ code: 'customer.settings.wishlist.general_option', label: 'Enable wishlist', type: 'boolean' }],
      },
    ],
  },
  {
    key: 'sales',
    label: 'Sales',
    icon: <IconShoppingCart size={17} />,
    title: 'Sales',
    info: 'Shipping origin, carriers, taxes and order processing defaults.',
    groups: [
      {
        title: 'Shipping origin',
        info: 'Address used to calculate shipping rates.',
        fields: [
          { code: 'sales.shipping.origin.country', label: 'Country', type: 'text', placeholder: 'ES' },
          { code: 'sales.shipping.origin.state', label: 'State / province', type: 'text', placeholder: 'Barcelona' },
          { code: 'sales.shipping.origin.city', label: 'City', type: 'text', placeholder: 'Barcelona' },
          { code: 'sales.shipping.origin.postcode', label: 'Postcode', type: 'text', placeholder: '08001' },
        ],
      },
      {
        title: 'Flat rate carrier',
        fields: [
          { code: 'sales.carriers.flatrate.active', label: 'Active', type: 'boolean' },
          { code: 'sales.carriers.flatrate.title', label: 'Title', type: 'text', placeholder: 'Flat Rate' },
          { code: 'sales.carriers.flatrate.default_rate', label: 'Default rate', type: 'number', placeholder: '7.5' },
          {
            code: 'sales.carriers.flatrate.type',
            label: 'Calculation type',
            type: 'select',
            options: [
              { label: 'Per unit', value: 'per_unit' },
              { label: 'Per order', value: 'per_order' },
            ],
          },
        ],
      },
      {
        title: 'Free shipping',
        fields: [
          { code: 'sales.carriers.free.active', label: 'Active', type: 'boolean' },
          { code: 'sales.carriers.free.title', label: 'Title', type: 'text', placeholder: 'Free Shipping' },
        ],
      },
      {
        title: 'Taxes',
        fields: [
          {
            code: 'sales.taxes.calculation.based_on',
            label: 'Tax calculation based on',
            type: 'select',
            options: [
              { label: 'Billing address', value: 'billing_address' },
              { label: 'Shipping address', value: 'shipping_address' },
            ],
          },
          { code: 'sales.taxes.calculation.shipping_tax_class', label: 'Default shipping tax class', type: 'text', placeholder: 'standard' },
        ],
      },
      {
        title: 'Order settings',
        fields: [
          { code: 'sales.order_settings.order_number.prefix', label: 'Order number prefix', type: 'text', placeholder: '#' },
          { code: 'sales.order_settings.minimum_order.enable', label: 'Enable minimum order amount', type: 'boolean' },
          { code: 'sales.order_settings.minimum_order.amount', label: 'Minimum order amount', type: 'number', placeholder: '0' },
        ],
      },
    ],
  },
  {
    key: 'sales.payment_methods',
    label: 'Payment methods',
    icon: <IconCreditCard size={17} />,
    title: 'Payment methods',
    info: 'Enable the payment methods offered during storefront checkout.',
    groups: [
      {
        title: 'Cash on delivery',
        fields: [
          { code: 'sales.payment_methods.cashondelivery.active', label: 'Active', type: 'boolean' },
          { code: 'sales.payment_methods.cashondelivery.title', label: 'Title', type: 'text', placeholder: 'Cash On Delivery' },
          { code: 'sales.payment_methods.cashondelivery.description', label: 'Description', type: 'textarea', placeholder: 'Pay with cash when your order is delivered.' },
        ],
      },
      {
        title: 'Money transfer',
        fields: [
          { code: 'sales.payment_methods.moneytransfer.active', label: 'Active', type: 'boolean' },
          { code: 'sales.payment_methods.moneytransfer.title', label: 'Title', type: 'text', placeholder: 'Money Transfer' },
          { code: 'sales.payment_methods.moneytransfer.description', label: 'Description', type: 'textarea', placeholder: 'Transfer the total to our bank account.' },
        ],
      },
    ],
  },
];

export default function ConfigurationPage() {
  const [active, setActive] = useState('general');
  const [stored, setStored] = useState<Record<string, string>>({});
  const [values, setValues] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [saved, setSaved] = useState(false);
  const [uploading, setUploading] = useState<string | null>(null);

  const section = SECTIONS.find((entry) => entry.key === active) ?? SECTIONS[0];

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError('');
    api
      .adminGet(`admin/configuration/${section.key}`)
      .then((data) => {
        if (cancelled) return;
        const rows: any[] = Array.isArray(data) ? data : data?.data ?? [];
        const map: Record<string, string> = {};
        rows.forEach((row) => {
          map[row.code] = row.value ?? '';
        });
        setStored(map);
        const next: Record<string, string> = {};
        section.groups.forEach((group) =>
          group.fields.forEach((field) => {
            next[field.code] = map[field.code] ?? '';
          }),
        );
        setValues(next);
      })
      .catch((requestError) => {
        if (!cancelled) setError(requestError?.message ?? t('Unable to load the configuration.'));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [section.key]);

  const changes = useMemo(
    () =>
      Object.entries(values).filter(([code, value]) => {
        const current = stored[code] ?? '';
        return String(value) !== String(current);
      }).length,
    [values, stored],
  );

  const setValue = (code: string, value: string) => {
    setSaved(false);
    setValues((current) => ({ ...current, [code]: value }));
  };

  const uploadImage = async (field: ConfigField, file: File | null) => {
    if (!file) return;
    setUploading(field.code);
    setError('');
    try {
      const result = await api.upload(file, 'brand');
      const path = result?.path ?? String(result?.url ?? '').replace(/^\/?storage\//, '');
      if (!path) throw new Error(t('Unable to upload the image.'));
      setValue(field.code, String(path));
    } catch (requestError: any) {
      setError(requestError?.message ?? t('Unable to upload the image.'));
    } finally {
      setUploading(null);
    }
  };

  const save = async () => {
    setSaving(true);
    setError('');
    try {
      const payload: Record<string, string> = {};
      section.groups.forEach((group) =>
        group.fields.forEach((field) => {
          payload[field.code] = values[field.code] ?? '';
        }),
      );
      await api.adminPost('admin/configuration', { values: payload });
      setStored((current) => ({ ...current, ...payload }));
      setSaved(true);
      // El idioma de la interfaz se sirve desde /core/context: refrescamos el
      // contexto para que tienda y panel se redibujen con el nuevo idioma.
      await useShop.getState().loadContext();
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to save the configuration.');
    } finally {
      setSaving(false);
    }
  };

  const renderField = (field: ConfigField) => {
    const value = values[field.code] ?? '';
    const inherited = !stored[field.code];

    if (field.type === 'boolean') {
      const on = value === '1' || value === 'true';
      return (
        <button
          type="button"
          className="admin-switch"
          data-on={on}
          aria-pressed={on}
          onClick={() => setValue(field.code, on ? '0' : '1')}
        >
          <span className="admin-switch-knob" />
        </button>
      );
    }

    if (field.type === 'select') {
      return (
        <select id={field.code} className="admin-input" value={value} onChange={(event) => setValue(field.code, event.target.value)}>
          <option value="">{t('Use system default')}</option>
          {field.options?.map((option) => (
            <option key={option.value} value={option.value}>
              {field.code.endsWith('ui_locale') ? option.label : t(option.label)}
            </option>
          ))}
        </select>
      );
    }

    if (field.type === 'image') {
      const preview = value ? mediaUrl(value) : '';
      const busy = uploading === field.code;
      return (
        <div className="admin-image-field">
          <div className="admin-image-field-preview">
            {preview ? <img src={preview} alt={t(field.label)} /> : <IconPhoto size={26} />}
          </div>
          <div className="admin-image-field-body">
            <div className="admin-image-field-actions">
              <label className={`admin-button${busy ? ' is-disabled' : ''}`}>
                <IconUpload size={15} />
                {busy ? t('Uploading…') : value ? t('Replace image') : t('Upload image')}
                <input
                  id={field.code}
                  type="file"
                  accept={field.accept ?? 'image/*'}
                  hidden
                  disabled={busy}
                  onChange={(event) => {
                    void uploadImage(field, event.target.files?.[0] ?? null);
                    event.target.value = '';
                  }}
                />
              </label>
              {value && (
                <button type="button" className="admin-button" onClick={() => setValue(field.code, '')} disabled={busy}>
                  <IconTrash size={15} /> {t('Remove')}
                </button>
              )}
            </div>
            <span className="admin-image-field-name">
              {value ? t('Image uploaded') : t('No image selected')}
            </span>
          </div>
        </div>
      );
    }

    if (field.type === 'textarea') {
      return (
        <textarea
          id={field.code}
          className="admin-input admin-textarea"
          rows={3}
          value={value}
          placeholder={field.placeholder ? t(field.placeholder) : undefined}
          onChange={(event) => setValue(field.code, event.target.value)}
        />
      );
    }

    return (
      <input
        id={field.code}
        className="admin-input"
        type={field.type === 'number' ? 'number' : 'text'}
        value={value}
        placeholder={field.placeholder ? t(field.placeholder) : undefined}
        onChange={(event) => setValue(field.code, event.target.value)}
      />
    );
  };

  return (
    <div>
      <AdminPageHeader
        title={t('Configuration')}
        subtitle={t('Manage storefront behaviour, sales channels, carriers and customer defaults.')}
        actions={
          <button type="button" className="admin-button primary" onClick={save} disabled={saving || loading}>
            {saving ? t('Saving…') : t('Save configuration')}
          </button>
        }
      />

      {error && <div className="admin-alert danger">{error}</div>}
      {saved && !error && (
        <div className="admin-alert success">
          <IconCheck size={15} /> {t('Configuration saved successfully.')}
        </div>
      )}

      <div className="admin-config-layout">
        <aside className="admin-config-nav">
          {SECTIONS.map((entry) => (
            <button
              key={entry.key}
              type="button"
              className="admin-config-nav-item"
              data-active={active === entry.key}
              onClick={() => setActive(entry.key)}
            >
              {entry.icon}
              <span>{t(entry.label)}</span>
            </button>
          ))}
        </aside>

        <div className="admin-config-content">
          <div className="admin-config-head">
            <div>
              <h2>{t(section.title)}</h2>
              <p>{t(section.info)}</p>
            </div>
            {changes > 0 && <span className="admin-pill blue">{t(changes === 1 ? '{count} unsaved change' : '{count} unsaved changes', { count: changes })}</span>}
          </div>

          {loading ? (
            <div className="admin-config-group">
              <div className="admin-skeleton-row" />
              <div className="admin-skeleton-row" />
              <div className="admin-skeleton-row" />
            </div>
          ) : (
            section.groups.map((group) => (
              <section className="admin-config-group" key={group.title}>
                <div className="admin-config-group-head">
                  <h3>{t(group.title)}</h3>
                  {group.info && <p>{t(group.info)}</p>}
                </div>
                <div className="admin-config-fields">
                  {group.fields.map((field) => (
                    <div className="admin-config-field" key={field.code}>
                      <div className="admin-config-label">
                        <label htmlFor={field.code}>{t(field.label)}</label>
                        <code>{field.code}</code>
                        {field.hint && <span>{t(field.hint)}</span>}
                      </div>
                      <div className="admin-config-control">
                        {renderField(field)}
                        {!stored[field.code] && (
                          <span className="admin-config-inherited">
                            {values[field.code] ? t('Custom value') : t('Not configured yet')}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </section>
            ))
          )}

          <p className="admin-muted admin-small" style={{ marginTop: 18 }}>
            {t('Values are stored in')} <code>core_config</code> {t('and applied immediately to the storefront.')}
          </p>
        </div>
      </div>
    </div>
  );
}
