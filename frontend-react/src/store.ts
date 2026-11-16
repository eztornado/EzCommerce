import { create } from 'zustand';
import { api } from './api/client';

export interface ShopContext {
  locale: string;
  currency: string;
  exchange_rate?: number;
  header_offer?: string | null;
  locales: { code: string; name: string }[];
  currencies: { code: string; name: string; symbol: string }[];
  channel: { code: string; name: string };
}

interface ShopState {
  context: ShopContext | null;
  loadContext: () => Promise<void>;
  changeLocale: (code: string) => Promise<void>;
  changeCurrency: (code: string) => Promise<void>;
}

export const useShop = create<ShopState>((set) => ({
  context: null,
  loadContext: async () => {
    try { set({ context: await api.get('core/context') }); } catch { /* ignore */ }
  },
  changeLocale: async (code) => {
    localStorage.setItem('shop_locale', code);
    await useShop.getState().loadContext();
    window.dispatchEvent(new Event('shop-reload'));
  },
  changeCurrency: async (code) => {
    localStorage.setItem('shop_currency', code);
    await useShop.getState().loadContext();
    window.dispatchEvent(new Event('shop-reload'));
  },
}));

// Idioma y moneda elegidos se adjuntan a cada llamada de shop.
export function shopQuery(extra: Record<string, string> = {}): string {
  const params = new URLSearchParams(extra);
  const locale = localStorage.getItem('shop_locale');
  const currency = localStorage.getItem('shop_currency');
  if (locale) params.set('locale', locale);
  if (currency) params.set('currency', currency);
  const q = params.toString();
  return q ? `?${q}` : '';
}

export const shopCurrency = () => localStorage.getItem('shop_currency') ?? '';

interface CartState {
  count: number;
  refresh?: () => Promise<void>;
}

export const useCart = create<CartState>(() => ({ count: 0 }));

interface AuthState {
  customer: { email: string; first_name?: string } | null;
  loggedIn: boolean;
  restore: () => Promise<void>;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

export const useAuth = create<AuthState>((set) => ({
  customer: null,
  loggedIn: !!localStorage.getItem('customer_token'),
  restore: async () => {
    if (!localStorage.getItem('customer_token')) return;
    try {
      const data = await api.get('customer/get');
      set({ customer: data, loggedIn: true });
    } catch { api.clearCustomerSession(); set({ loggedIn: false }); }
  },
  login: async (email, password) => {
    const data = await api.post('customer/login', { email, password });
    localStorage.setItem('customer_token', data.token);
    set({ customer: data.user, loggedIn: true });
  },
  logout: async () => {
    try { await api.post('customer/logout'); } catch { /* ignore */ }
    api.clearCustomerSession();
    set({ customer: null, loggedIn: false });
  },
}));

interface AdminAuthState {
  loggedIn: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

export const useAdminAuth = create<AdminAuthState>(() => ({
  loggedIn: !!localStorage.getItem('admin_token'),
  login: async (email, password) => {
    const data = await api.post('admin/login', { email, password }, true);
    localStorage.setItem('admin_token', data.token);
  },
  logout: async () => {
    try { await api.adminPost('admin/logout'); } catch { /* ignore */ }
    localStorage.removeItem('admin_token');
  },
}));
