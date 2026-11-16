import { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useShop, useAuth } from './store';
import ShopLayout from './pages/shop/ShopLayout';
import HomePage from './pages/shop/HomePage';
import CategoryPage from './pages/shop/CategoryPage';
import ProductPage from './pages/shop/ProductPage';
import CartPage from './pages/shop/CartPage';
import CheckoutPage from './pages/shop/CheckoutPage';
import SignInPage from './pages/shop/SignInPage';
import AccountPage from './pages/shop/AccountPage';
import AdminLoginPage from './pages/admin/LoginPage';
import AdminShell from './pages/admin/AdminShell';
import DashboardPage from './pages/admin/DashboardPage';
import ProductsPage from './pages/admin/ProductsPage';
import ProductEditorPage from './pages/admin/ProductEditorPage';
import { OrdersPage, InvoicesPage, ShipmentsPage, RefundsPage } from './pages/admin/SalesPages';
import { CategoriesPage, AttributesPage, FamiliesPage } from './pages/admin/CatalogPages';
import { CustomersPage, GroupsPage, ReviewsPage } from './pages/admin/CustomersPages';
import { CartRulesPage, CatalogRulesPage, SearchSeoPage } from './pages/admin/MarketingPages';
import { InventorySourcesPage } from './pages/admin/SettingsPages';
import { ConfigurationPage, DataTransferPage } from './pages/admin/SettingsExtra';
import ReportingPage from './pages/admin/ReportingPage';

export default function App() {
  const loadContext = useShop((s) => s.loadContext);
  const restore = useAuth((s) => s.restore);
  useEffect(() => { loadContext(); restore(); }, [loadContext, restore]);

  return (
    <BrowserRouter>
      <Routes>
        <Route element={<ShopLayout />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/category/:id" element={<CategoryPage />} />
          <Route path="/product/:key" element={<ProductPage />} />
          <Route path="/cart" element={<CartPage />} />
          <Route path="/checkout" element={<CheckoutPage />} />
          <Route path="/login" element={<SignInPage />} />
          <Route path="/account" element={<AccountPage />} />
        </Route>
        <Route path="/admin/login" element={<AdminLoginPage />} />
        <Route path="/admin" element={<AdminShell />}>
          <Route index element={<DashboardPage />} />
          <Route path="products" element={<ProductsPage />} />
          <Route path="products/edit/:id" element={<ProductEditorPage />} />
          <Route path="categories" element={<CategoriesPage />} />
          <Route path="attributes" element={<AttributesPage />} />
          <Route path="families" element={<FamiliesPage />} />
          <Route path="orders" element={<OrdersPage />} />
          <Route path="invoices" element={<InvoicesPage />} />
          <Route path="shipments" element={<ShipmentsPage />} />
          <Route path="refunds" element={<RefundsPage />} />
          <Route path="customers" element={<CustomersPage />} />
          <Route path="customers/groups" element={<GroupsPage />} />
          <Route path="customers/reviews" element={<ReviewsPage />} />
          <Route path="cart-rules" element={<CartRulesPage />} />
          <Route path="catalog-rules" element={<CatalogRulesPage />} />
          <Route path="search-seo" element={<SearchSeoPage />} />
          <Route path="inventory-sources" element={<InventorySourcesPage />} />
          <Route path="configuration" element={<ConfigurationPage />} />
          <Route path="data-transfer" element={<DataTransferPage />} />
          <Route path="reporting" element={<ReportingPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
