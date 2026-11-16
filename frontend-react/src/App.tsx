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
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
