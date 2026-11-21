import { create } from 'zustand';
import { api, queryString } from './api/client';

export interface ShopContext {
  locale: string;
  ui_locale?: string;
  currency: string;
  exchange_rate?: number;
  header_offer?: string | null;
  locales: { code: string; name: string; direction?: string }[];
  currencies: { code: string; name: string; symbol: string }[];
  channel: {
    code: string;
    name: string;
    logo?: string | null;
    base_currency_code?: string;
    default_locale_code?: string | null;
  };
}

interface ShopState {
  context: ShopContext | null;
  loading: boolean;
  loadContext: () => Promise<void>;
  changeLocale: (code: string) => Promise<void>;
  changeCurrency: (code: string) => Promise<void>;
}

export const useShop = create<ShopState>((set) => ({
  context: null,
  loading: false,
  loadContext: async () => {
    try {
      set({ loading: true });
      const context = await api.get(`core/context${shopQuery()}`);
      set({ context, loading: false });
    } catch {
      set({ loading: false });
    }
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

// Idioma y moneda elegidos se adjuntan a cada llamada de la tienda.
export function shopQuery(extra: Record<string, string | number | null | undefined> = {}): string {
  return queryString({
    locale: localStorage.getItem('shop_locale'),
    currency: localStorage.getItem('shop_currency'),
    ...extra,
  });
}

export const shopCurrency = () => localStorage.getItem('shop_currency') ?? '';

interface CartState {
  count: number;
  items: number;
  cart: any;
  loading: boolean;
  refresh: () => Promise<void>;
}

export const useCart = create<CartState>((set) => ({
  count: 0,
  items: 0,
  cart: null,
  loading: false,
  refresh: async () => {
    try {
      const cart = await api.get(`checkout/cart${shopQuery()}`);
      set({
        cart: cart && cart.items ? cart : null,
        count: cart?.items_qty ?? 0,
        items: cart?.items_count ?? 0,
        loading: false,
      });
    } catch {
      set({ count: 0, items: 0, cart: null, loading: false });
    }
  },
}));

interface AuthState {
  customer: { email: string; first_name?: string; last_name?: string } | null;
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
    } catch {
      api.clearCustomerSession();
      set({ loggedIn: false });
    }
  },
  login: async (email, password) => {
    const data = await api.post('customer/login', { email, password });
    localStorage.setItem('customer_token', data.token);
    set({ customer: data.user, loggedIn: true });
  },
  logout: async () => {
    try {
      await api.post('customer/logout');
    } catch {
      /* sesión ya expirada */
    }
    api.clearCustomerSession();
    set({ customer: null, loggedIn: false });
  },
}));

interface AdminAuthState {
  loggedIn: boolean;
  user: { name?: string; email?: string; id?: number } | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  restore: () => Promise<void>;
}

export const useAdminAuth = create<AdminAuthState>((set) => ({
  loggedIn: !!localStorage.getItem('admin_token'),
  user: null,
  login: async (email, password) => {
    const data = await api.post('admin/login', { email, password }, true);
    localStorage.setItem('admin_token', data.token);
    if (data.user) localStorage.setItem('admin_user', JSON.stringify(data.user));
    set({ loggedIn: true, user: data.user ?? null });
  },
  logout: async () => {
    try {
      await api.adminPost('admin/logout');
    } catch {
      /* token caducado */
    }
    api.clearAdminSession();
    localStorage.removeItem('admin_user');
    set({ loggedIn: false, user: null });
  },
  restore: async () => {
    if (!localStorage.getItem('admin_token')) return;
    const cached = localStorage.getItem('admin_user');
    if (cached) {
      try {
        set({ user: JSON.parse(cached), loggedIn: true });
      } catch {
        /* ignore */
      }
    }
    try {
      const me = await api.adminGet('admin/me');
      set({ user: me, loggedIn: true });
      localStorage.setItem('admin_user', JSON.stringify(me));
    } catch (error: any) {
      if (error?.status === 401) {
        api.clearAdminSession();
        localStorage.removeItem('admin_user');
        set({ loggedIn: false, user: null });
      }
    }
  },
}));
