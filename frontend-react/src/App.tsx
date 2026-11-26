import { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useShop, useAuth } from './store';
import I18nBridge from './i18n/I18nBridge';
import { mediaUrl } from './lib/format';

import ShopLayout from './components/shop/ShopLayout';
import HomePage from './pages/shop/HomePage';
import CategoryPage from './pages/shop/CategoryPage';
import SearchPage from './pages/shop/SearchPage';
import ProductPage from './pages/shop/ProductPage';
import CartPage from './pages/shop/CartPage';
import CheckoutPage from './pages/shop/CheckoutPage';
import SuccessPage from './pages/shop/SuccessPage';
import SignInPage from './pages/shop/SignInPage';
import RegisterPage from './pages/shop/RegisterPage';
import AccountPage from './pages/shop/AccountPage';

import AdminShell from './components/admin/AdminShell';
import AdminLoginPage from './pages/admin/LoginPage';
import DashboardPage from './pages/admin/DashboardPage';
import ProductsPage from './pages/admin/ProductsPage';
import ProductEditorPage from './pages/admin/ProductEditorPage';
import { AttributesPage, FamiliesPage } from './pages/admin/CatalogPages';
import CategoriesPage from './pages/admin/CategoriesPage';
import { OrdersPage, InvoicesPage, ShipmentsPage, RefundsPage } from './pages/admin/SalesPages';
import OrderViewPage from './pages/admin/OrderViewPage';
import { CustomersPage, GroupsPage, ReviewsPage } from './pages/admin/CustomersPages';
import CustomerViewPage from './pages/admin/CustomerViewPage';
import { RmaPage, GdprPage } from './pages/admin/RmaGdprPage';
import { CartRulesPage, CatalogRulesPage } from './pages/admin/MarketingPages';
import CartRuleEditorPage from './pages/admin/CartRuleEditorPage';
import CatalogRuleEditorPage from './pages/admin/CatalogRuleEditorPage';
import {
  TemplatesPage,
  EventsPage,
  CampaignsPage,
  SubscribersPage,
} from './pages/admin/CommunicationPages';
import { UrlRewritesPage, SearchTermsPage, SearchSynonymsPage } from './pages/admin/SeoPages';
import { CmsPagesPage, CmsEditorPage } from './pages/admin/CmsPages';
import {
  LocalesPage,
  CurrenciesPage,
  ExchangeRatesPage,
  ChannelsPage,
  UsersPage,
  RolesPage,
  TaxCategoriesPage,
  TaxRatesPage,
  InventorySourcesPage,
} from './pages/admin/SettingsPages';
import ConfigurationPage from './pages/admin/ConfigurationPage';
import DataTransferPage from './pages/admin/DataTransferPage';
import AppearancePage from './pages/admin/AppearancePage';
import ReportingPage from './pages/admin/ReportingPage';

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

  return (
    <I18nBridge>
      <BrowserRouter>
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
      </BrowserRouter>
    </I18nBridge>
  );
}
