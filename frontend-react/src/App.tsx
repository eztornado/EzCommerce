import { lazy, Suspense, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useShop, useAuth } from './store';
import I18nBridge from './i18n/I18nBridge';
import { mediaUrl } from './lib/format';
import ErrorBoundary from './components/ErrorBoundary';
import RouteSkeleton from './components/RouteSkeleton';

import ShopLayout from './components/shop/ShopLayout';
import HomePage from './pages/shop/HomePage';

// Rutas secundarias y todo el panel se cargan bajo demanda: el storefront
// inicial solo descarga lo necesario para el home.
const CategoryPage = lazy(() => import('./pages/shop/CategoryPage'));
const SearchPage = lazy(() => import('./pages/shop/SearchPage'));
const ProductPage = lazy(() => import('./pages/shop/ProductPage'));
const CartPage = lazy(() => import('./pages/shop/CartPage'));
const CheckoutPage = lazy(() => import('./pages/shop/CheckoutPage'));
const SuccessPage = lazy(() => import('./pages/shop/SuccessPage'));
const SignInPage = lazy(() => import('./pages/shop/SignInPage'));
const RegisterPage = lazy(() => import('./pages/shop/RegisterPage'));
const AccountPage = lazy(() => import('./pages/shop/AccountPage'));

const AdminShell = lazy(() => import('./components/admin/AdminShell'));
const AdminLoginPage = lazy(() => import('./pages/admin/LoginPage'));
const DashboardPage = lazy(() => import('./pages/admin/DashboardPage'));
const ProductsPage = lazy(() => import('./pages/admin/ProductsPage'));
const ProductEditorPage = lazy(() => import('./pages/admin/ProductEditorPage'));
const CategoriesPage = lazy(() => import('./pages/admin/CategoriesPage'));
const OrderViewPage = lazy(() => import('./pages/admin/OrderViewPage'));
const CustomerViewPage = lazy(() => import('./pages/admin/CustomerViewPage'));
const CartRuleEditorPage = lazy(() => import('./pages/admin/CartRuleEditorPage'));
const CatalogRuleEditorPage = lazy(() => import('./pages/admin/CatalogRuleEditorPage'));
const ConfigurationPage = lazy(() => import('./pages/admin/ConfigurationPage'));
const DataTransferPage = lazy(() => import('./pages/admin/DataTransferPage'));
const AppearancePage = lazy(() => import('./pages/admin/AppearancePage'));
const ReportingPage = lazy(() => import('./pages/admin/ReportingPage'));

const AttributesPage = lazy(() =>
  import('./pages/admin/CatalogPages').then((module) => ({ default: module.AttributesPage })),
);
const FamiliesPage = lazy(() =>
  import('./pages/admin/CatalogPages').then((module) => ({ default: module.FamiliesPage })),
);
const OrdersPage = lazy(() =>
  import('./pages/admin/SalesPages').then((module) => ({ default: module.OrdersPage })),
);
const InvoicesPage = lazy(() =>
  import('./pages/admin/SalesPages').then((module) => ({ default: module.InvoicesPage })),
);
const ShipmentsPage = lazy(() =>
  import('./pages/admin/SalesPages').then((module) => ({ default: module.ShipmentsPage })),
);
const RefundsPage = lazy(() =>
  import('./pages/admin/SalesPages').then((module) => ({ default: module.RefundsPage })),
);
const CustomersPage = lazy(() =>
  import('./pages/admin/CustomersPages').then((module) => ({ default: module.CustomersPage })),
);
const GroupsPage = lazy(() =>
  import('./pages/admin/CustomersPages').then((module) => ({ default: module.GroupsPage })),
);
const ReviewsPage = lazy(() =>
  import('./pages/admin/CustomersPages').then((module) => ({ default: module.ReviewsPage })),
);
const RmaPage = lazy(() =>
  import('./pages/admin/RmaGdprPage').then((module) => ({ default: module.RmaPage })),
);
const GdprPage = lazy(() =>
  import('./pages/admin/RmaGdprPage').then((module) => ({ default: module.GdprPage })),
);
const CartRulesPage = lazy(() =>
  import('./pages/admin/MarketingPages').then((module) => ({ default: module.CartRulesPage })),
);
const CatalogRulesPage = lazy(() =>
  import('./pages/admin/MarketingPages').then((module) => ({ default: module.CatalogRulesPage })),
);
const TemplatesPage = lazy(() =>
  import('./pages/admin/CommunicationPages').then((module) => ({ default: module.TemplatesPage })),
);
const EventsPage = lazy(() =>
  import('./pages/admin/CommunicationPages').then((module) => ({ default: module.EventsPage })),
);
const CampaignsPage = lazy(() =>
  import('./pages/admin/CommunicationPages').then((module) => ({ default: module.CampaignsPage })),
);
const SubscribersPage = lazy(() =>
  import('./pages/admin/CommunicationPages').then((module) => ({ default: module.SubscribersPage })),
);
const UrlRewritesPage = lazy(() =>
  import('./pages/admin/SeoPages').then((module) => ({ default: module.UrlRewritesPage })),
);
const SearchTermsPage = lazy(() =>
  import('./pages/admin/SeoPages').then((module) => ({ default: module.SearchTermsPage })),
);
const SearchSynonymsPage = lazy(() =>
  import('./pages/admin/SeoPages').then((module) => ({ default: module.SearchSynonymsPage })),
);
const CmsPagesPage = lazy(() =>
  import('./pages/admin/CmsPages').then((module) => ({ default: module.CmsPagesPage })),
);
const CmsEditorPage = lazy(() =>
  import('./pages/admin/CmsPages').then((module) => ({ default: module.CmsEditorPage })),
);
const LocalesPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.LocalesPage })),
);
const CurrenciesPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.CurrenciesPage })),
);
const ExchangeRatesPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.ExchangeRatesPage })),
);
const ChannelsPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.ChannelsPage })),
);
const UsersPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.UsersPage })),
);
const RolesPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.RolesPage })),
);
const TaxCategoriesPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.TaxCategoriesPage })),
);
const TaxRatesPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.TaxRatesPage })),
);
const InventorySourcesPage = lazy(() =>
  import('./pages/admin/SettingsPages').then((module) => ({ default: module.InventorySourcesPage })),
);

export default function App() {
  const loadContext = useShop((state) => state.loadContext);
  const favicon = useShop((state) => state.context?.channel?.favicon ?? state.brand?.favicon);
  const restore = useAuth((state) => state.restore);

  useEffect(() => {
    loadContext();
    restore();
  }, [loadContext, restore]);

  useEffect(() => {
    if (!favicon) return;
    let link = document.querySelector<HTMLLinkElement>("link[rel~='icon']");
    if (!link) {
      link = document.createElement('link');
      link.rel = 'icon';
      document.head.appendChild(link);
    }
    link.href = mediaUrl(favicon);
  }, [favicon]);

  // Prefetch en idle de las páginas de la tienda (#39): elimina el flash en
  // blanco al navegar. Se hace tras 'load' y con requestIdleCallback para no
  // competir con el render inicial ni con el LCP; el panel sigue bajo demanda.
  useEffect(() => {
    const w = window as Window & {
      requestIdleCallback?: (cb: () => void, opts?: { timeout: number }) => number;
      cancelIdleCallback?: (id: number) => void;
    };
    let idleId: number | null = null;
    const start = () => {
      const run = () => {
        void import('./pages/shop/CategoryPage');
        void import('./pages/shop/SearchPage');
        void import('./pages/shop/ProductPage');
        void import('./pages/shop/CartPage');
        void import('./pages/shop/CheckoutPage');
        void import('./pages/shop/SuccessPage');
        void import('./pages/shop/SignInPage');
        void import('./pages/shop/RegisterPage');
        void import('./pages/shop/AccountPage');
      };
      if (w.requestIdleCallback) {
        idleId = w.requestIdleCallback(run, { timeout: 4000 });
      } else {
        idleId = window.setTimeout(run, 2000);
      }
    };
    if (document.readyState === 'complete') {
      start();
    } else {
      window.addEventListener('load', start, { once: true });
      return () => window.removeEventListener('load', start);
    }
    return () => {
      if (idleId !== null && w.cancelIdleCallback) w.cancelIdleCallback(idleId);
    };
  }, []);

  return (
    <I18nBridge>
      <BrowserRouter>
      <ErrorBoundary>
      <Suspense fallback={<RouteSkeleton />}>
      <Routes>
        {/* Storefront */}
        <Route element={<ShopLayout />}>
          <Route index element={<HomePage />} />
          <Route path="/products" element={<SearchPage />} />
          <Route path="/search" element={<SearchPage />} />
          <Route path="/category/:id" element={<CategoryPage />} />
          <Route path="/product/:key" element={<ProductPage />} />
          <Route path="/cart" element={<CartPage />} />
          <Route path="/checkout" element={<CheckoutPage />} />
          <Route path="/checkout/success/:id" element={<SuccessPage />} />
          <Route path="/login" element={<SignInPage />} />
          <Route path="/register" element={<RegisterPage />} />
          <Route path="/account" element={<AccountPage />} />
          <Route path="/account/:tab" element={<AccountPage />} />
        </Route>

        {/* Admin */}
        <Route path="/admin/login" element={<AdminLoginPage />} />
        <Route path="/admin" element={<AdminShell />}>
          <Route index element={<DashboardPage />} />

          {/* Sales */}
          <Route path="sales/orders" element={<OrdersPage />} />
          <Route path="sales/orders/view/:id" element={<OrderViewPage />} />
          <Route path="sales/invoices" element={<InvoicesPage />} />
          <Route path="sales/shipments" element={<ShipmentsPage />} />
          <Route path="sales/refunds" element={<RefundsPage />} />

          {/* Catalog */}
          <Route path="catalog/products" element={<ProductsPage />} />
          <Route path="catalog/products/create" element={<ProductEditorPage />} />
          <Route path="catalog/products/edit/:id" element={<ProductEditorPage />} />
          <Route path="catalog/categories" element={<CategoriesPage />} />
          <Route path="catalog/attributes" element={<AttributesPage />} />
          <Route path="catalog/families" element={<FamiliesPage />} />

          {/* Customers */}
          <Route path="customers" element={<CustomersPage />} />
          <Route path="customers/view/:id" element={<CustomerViewPage />} />
          <Route path="customers/groups" element={<GroupsPage />} />
          <Route path="customers/reviews" element={<ReviewsPage />} />
          <Route path="customers/rma" element={<RmaPage />} />
          <Route path="customers/gdpr" element={<GdprPage />} />

          {/* Marketing */}
          <Route path="marketing/promotions/cart-rules" element={<CartRulesPage />} />
          <Route path="marketing/promotions/cart-rules/create" element={<CartRuleEditorPage />} />
          <Route path="marketing/promotions/cart-rules/edit/:id" element={<CartRuleEditorPage />} />
          <Route path="marketing/promotions/catalog-rules" element={<CatalogRulesPage />} />
          <Route path="marketing/promotions/catalog-rules/create" element={<CatalogRuleEditorPage />} />
          <Route path="marketing/promotions/catalog-rules/edit/:id" element={<CatalogRuleEditorPage />} />
          <Route path="marketing/communications/templates" element={<TemplatesPage />} />
          <Route path="marketing/communications/events" element={<EventsPage />} />
          <Route path="marketing/communications/campaigns" element={<CampaignsPage />} />
          <Route path="marketing/communications/subscribers" element={<SubscribersPage />} />
          <Route path="marketing/search-seo/url-rewrites" element={<UrlRewritesPage />} />
          <Route path="marketing/search-seo/search-terms" element={<SearchTermsPage />} />
          <Route path="marketing/search-seo/search-synonyms" element={<SearchSynonymsPage />} />

          {/* CMS */}
          <Route path="cms/pages" element={<CmsPagesPage />} />
          <Route path="cms/pages/create" element={<CmsEditorPage />} />
          <Route path="cms/pages/edit/:id" element={<CmsEditorPage />} />

          {/* Reporting */}
          <Route path="reporting" element={<Navigate to="/admin/reporting/sales" replace />} />
          <Route path="reporting/:module" element={<ReportingPage />} />

          {/* Settings */}
          <Route path="settings/locales" element={<LocalesPage />} />
          <Route path="settings/currencies" element={<CurrenciesPage />} />
          <Route path="settings/exchange-rates" element={<ExchangeRatesPage />} />
          <Route path="settings/channels" element={<ChannelsPage />} />
          <Route path="settings/users" element={<UsersPage />} />
          <Route path="settings/roles" element={<RolesPage />} />
          <Route path="settings/taxes/categories" element={<TaxCategoriesPage />} />
          <Route path="settings/taxes/rates" element={<TaxRatesPage />} />
          <Route path="settings/inventory-sources" element={<InventorySourcesPage />} />
          <Route path="settings/data-transfer" element={<DataTransferPage />} />
          <Route path="settings/appearance" element={<AppearancePage />} />
          <Route path="configuration" element={<ConfigurationPage />} />
        </Route>

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      </Suspense>
      </ErrorBoundary>
      </BrowserRouter>
    </I18nBridge>
  );
}
