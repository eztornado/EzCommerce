import { useEffect, useState } from 'react';
import { api } from '../../api/client';
import ResourcePage from '../../components/admin/ResourcePage';

function useLookups() {
  const [locales, setLocales] = useState<any[]>([]);
  const [currencies, setCurrencies] = useState<any[]>([]);
  const [roles, setRoles] = useState<any[]>([]);
  const [categories, setCategories] = useState<any[]>([]);

  useEffect(() => {
    api.adminGet('admin/settings/locales').then((rows) => setLocales(Array.isArray(rows) ? rows : [])).catch(() => undefined);
    api.adminGet('admin/settings/currencies').then((rows) => setCurrencies(Array.isArray(rows) ? rows : [])).catch(() => undefined);
    api.adminGet('admin/settings/roles').then((rows) => setRoles(Array.isArray(rows) ? rows : [])).catch(() => undefined);
    api.adminGet('admin/catalog/categories/tree').then((rows) => setCategories(Array.isArray(rows) ? rows : [])).catch(() => undefined);
  }, []);

  return { locales, currencies, roles, categories };
}

export function LocalesPage() {
  return (
    <ResourcePage
      title="Locales"
      subtitle="Languages available across channels and storefront translations."
      endpoint="admin/settings/locales"
      createLabel="Create locale"
      createEndpoint="admin/settings/locales"
      updateEndpoint="admin/settings/locales/edit/{id}"
      deleteEndpoint="admin/settings/locales/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'code', label: 'Code', required: true, placeholder: 'es' },
        { name: 'name', label: 'Name', required: true, placeholder: 'Spanish' },
        {
          name: 'direction',
          label: 'Direction',
          type: 'select',
          options: [
            { label: 'Left to right', value: 'ltr' },
            { label: 'Right to left', value: 'rtl' },
          ],
          defaultValue: 'ltr',
        },
      ]}
      columns={['id', 'code', 'name', 'direction']}
      emptyLabel="No locales found."
    />
  );
}

export function CurrenciesPage() {
  return (
    <ResourcePage
      title="Currencies"
      subtitle="Currencies that can be used as channel base or display currencies."
      endpoint="admin/settings/currencies"
      createLabel="Create currency"
      createEndpoint="admin/settings/currencies"
      updateEndpoint="admin/settings/currencies/edit/{id}"
      deleteEndpoint="admin/settings/currencies/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'code', label: 'Code', required: true, placeholder: 'USD' },
        { name: 'name', label: 'Name', required: true, placeholder: 'US Dollar' },
        { name: 'symbol', label: 'Symbol', placeholder: '$' },
        { name: 'decimal', label: 'Decimals', type: 'number', defaultValue: 2 },
      ]}
      columns={['id', 'code', 'name', 'symbol', 'decimal']}
      emptyLabel="No currencies found."
    />
  );
}

export function ExchangeRatesPage() {
  const { currencies } = useLookups();
  return (
    <ResourcePage
      title="Exchange Rates"
      subtitle="Conversion rates applied when the storefront displays a non-base currency."
      endpoint="admin/settings/exchange-rates"
      createLabel="Add exchange rate"
      createEndpoint="admin/settings/exchange-rates"
      updateEndpoint="admin/settings/exchange-rates/edit/{id}"
      deleteEndpoint="admin/settings/exchange-rates/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        {
          name: 'target_currency',
          label: 'Currency',
          type: 'select',
          required: true,
          options: currencies.map((currency) => ({ label: `${currency.code} · ${currency.name}`, value: currency.id })),
        },
        { name: 'rate', label: 'Rate', type: 'number', step: 0.0001, required: true, defaultValue: 1 },
        {
          name: 'code',
          label: 'Code',
          formHidden: true,
          render: (value, row) => row.code ?? value,
        },
      ]}
      columns={['target_currency', 'code', 'rate']}
      cell={(field, row) => {
        if (field.name === 'target_currency') return row.code ?? `#${row.target_currency}`;
        if (field.name === 'rate') return row.rate;
        return row[field.name];
      }}
      transform={(values) => ({ target_currency: Number(values.target_currency), rate: Number(values.rate) })}
      emptyLabel="No exchange rates configured."
    />
  );
}

export function ChannelsPage() {
  const { locales, currencies, categories } = useLookups();
  return (
    <ResourcePage
      title="Channels"
      subtitle="Sales channels with their own locale, currency, hostname and catalog root."
      endpoint="admin/settings/channels"
      createLabel="Create channel"
      createEndpoint="admin/settings/channels"
      updateEndpoint="admin/settings/channels/edit/{id}"
      deleteEndpoint="admin/settings/channels/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'name', label: 'Name', required: true, placeholder: 'Store front' },
        { name: 'code', label: 'Code', required: true, placeholder: 'default' },
        { name: 'hostname', label: 'Hostname', placeholder: 'shop.example.com' },
        { name: 'timezone', label: 'Timezone', placeholder: 'UTC' },
        {
          name: 'theme',
          label: 'Theme',
          type: 'select',
          options: [{ label: 'Default', value: 'default' }],
          defaultValue: 'default',
        },
        {
          name: 'default_locale_id',
          label: 'Default locale',
          type: 'select',
          options: locales.map((locale) => ({ label: `${locale.code} · ${locale.name}`, value: locale.id })),
        },
        {
          name: 'base_currency_id',
          label: 'Base currency',
          type: 'select',
          options: currencies.map((currency) => ({ label: `${currency.code} · ${currency.name}`, value: currency.id })),
        },
        {
          name: 'root_category_id',
          label: 'Root category',
          type: 'select',
          options: categories.map((category) => ({ label: category.name, value: category.id })),
        },
        { name: 'is_maintenance_on', label: 'Maintenance mode', type: 'checkbox', defaultValue: false },
      ]}
      columns={['id', 'name', 'code', 'hostname', 'timezone', 'theme']}
      transform={(values) => {
        const locale = values.default_locale_id ? [Number(values.default_locale_id)] : [];
        const currency = values.base_currency_id ? [Number(values.base_currency_id)] : [];
        return {
          ...values,
          default_locale_id: values.default_locale_id ? Number(values.default_locale_id) : null,
          base_currency_id: values.base_currency_id ? Number(values.base_currency_id) : null,
          root_category_id: values.root_category_id ? Number(values.root_category_id) : null,
          locales: locale,
          currencies: currency,
        };
      }}
      emptyLabel="No channels found."
    />
  );
}

export function UsersPage() {
  const { roles } = useLookups();
  return (
    <ResourcePage
      title="Users"
      subtitle="Administrators who can sign in to this panel."
      endpoint="admin/settings/users"
      createLabel="Create user"
      canEdit={false}
      createEndpoint="admin/settings/users"
      deleteEndpoint="admin/settings/users/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'name', label: 'Name', required: true },
        { name: 'email', label: 'Email', type: 'email', required: true },
        { name: 'password', label: 'Password', type: 'password', required: true, createOnly: true },
        {
          name: 'role_id',
          label: 'Role',
          type: 'select',
          required: true,
          options: roles.map((role) => ({ label: role.name, value: role.id })),
          defaultValue: 1,
        },
        { name: 'status', label: 'Status', type: 'checkbox', defaultValue: true },
        { name: 'role', label: 'Role', formHidden: true },
      ]}
      columns={['id', 'name', 'email', 'role', 'status']}
      transform={(values, editing) => ({
        ...values,
        role_id: Number(values.role_id || 1),
        status: Number(values.status) ? 1 : 0,
        ...(editing ? {} : { password: values.password }),
      })}
      emptyLabel="No admin users found."
    />
  );
}

export function RolesPage() {
  return (
    <ResourcePage
      title="Roles"
      subtitle="Permission groups assigned to administrators."
      endpoint="admin/settings/roles"
      createLabel="Create role"
      canEdit={false}
      canDelete={false}
      createEndpoint="admin/settings/roles"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'name', label: 'Name', required: true, placeholder: 'Catalog manager' },
        { name: 'description', label: 'Description', type: 'textarea', colSpan: 2 },
        {
          name: 'permission_type',
          label: 'Permission type',
          type: 'select',
          options: [
            { label: 'All permissions', value: 'all' },
            { label: 'Custom permissions', value: 'custom' },
          ],
          defaultValue: 'all',
        },
      ]}
      columns={['id', 'name', 'description', 'permission_type']}
      emptyLabel="No roles found."
    />
  );
}

export function TaxCategoriesPage() {
  return (
    <ResourcePage
      title="Tax Categories"
      subtitle="Product tax categories grouping one or more tax rates."
      endpoint="admin/settings/taxes/categories"
      createLabel="Create tax category"
      createEndpoint="admin/settings/taxes/categories"
      updateEndpoint="admin/settings/taxes/categories/edit/{id}"
      deleteEndpoint="admin/settings/taxes/categories/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'code', label: 'Code', required: true, placeholder: 'standard' },
        { name: 'name', label: 'Name', required: true, placeholder: 'Standard rate' },
        { name: 'description', label: 'Description', type: 'textarea', colSpan: 2 },
      ]}
      columns={['id', 'code', 'name', 'description']}
      emptyLabel="No tax categories found."
    />
  );
}

export function TaxRatesPage() {
  return (
    <ResourcePage
      title="Tax Rates"
      subtitle="Country, state and ZIP-level tax percentages."
      endpoint="admin/settings/taxes/rates"
      createLabel="Create tax rate"
      createEndpoint="admin/settings/taxes/rates"
      updateEndpoint="admin/settings/taxes/rates/edit/{id}"
      deleteEndpoint="admin/settings/taxes/rates/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'identifier', label: 'Identifier', required: true, placeholder: 'ES-VAT' },
        { name: 'country', label: 'Country code', required: true, placeholder: 'ES' },
        { name: 'state', label: 'State code', placeholder: 'MD' },
        { name: 'tax_rate', label: 'Rate %', type: 'number', step: 0.01, defaultValue: 0 },
        { name: 'zip_code', label: 'ZIP code', placeholder: '28001' },
        { name: 'zip_from', label: 'ZIP from', placeholder: '28000' },
        { name: 'zip_to', label: 'ZIP to', placeholder: '28999' },
        { name: 'is_zip', label: 'ZIP range', type: 'checkbox', defaultValue: false },
      ]}
      columns={['id', 'identifier', 'country', 'state', 'tax_rate']}
      transform={(values) => ({
        ...values,
        country: String(values.country ?? '').toUpperCase(),
        tax_rate: Number(values.tax_rate || 0),
        is_zip: Number(values.is_zip) ? 1 : 0,
      })}
      emptyLabel="No tax rates found."
    />
  );
}

export function InventorySourcesPage() {
  return (
    <ResourcePage
      title="Inventory Sources"
      subtitle="Warehouses and stock locations used to fulfil orders."
      endpoint="admin/settings/inventory-sources"
      createLabel="Create source"
      createEndpoint="admin/settings/inventory-sources/store"
      updateEndpoint="admin/settings/inventory-sources/edit/{id}"
      deleteEndpoint="admin/settings/inventory-sources/{id}"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'code', label: 'Code', required: true, placeholder: 'madrid' },
        { name: 'name', label: 'Name', required: true, placeholder: 'Madrid warehouse' },
        { name: 'status', label: 'Active', type: 'checkbox', defaultValue: true },
        { name: 'description', label: 'Description', type: 'textarea', colSpan: 2 },
        { name: 'contact_name', label: 'Contact name' },
        { name: 'contact_email', label: 'Contact email', type: 'email' },
        { name: 'contact_number', label: 'Contact phone' },
        { name: 'street', label: 'Street' },
        { name: 'city', label: 'City' },
        { name: 'state', label: 'State' },
        { name: 'country', label: 'Country code' },
        { name: 'postcode', label: 'Postcode' },
        { name: 'priority', label: 'Priority', type: 'number', defaultValue: 0 },
      ]}
      columns={['id', 'code', 'name', 'city', 'country', 'status']}
      transform={(values) => ({ ...values, status: Number(values.status) ? 1 : 0, priority: Number(values.priority || 0) })}
      emptyLabel="No inventory sources found."
    />
  );
}
