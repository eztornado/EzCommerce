// Cliente API: mismos endpoints que consume el backend Rust (Larust).
// Tokens JWT (admin/customer) y carrito de invitado (X-Cart-Token) en
// localStorage, igual que el cliente Flutter anterior.

const BASE = import.meta.env.VITE_API_BASE ?? '';

function headers(admin = false): Record<string, string> {
  const h: Record<string, string> = { 'Content-Type': 'application/json', Accept: 'application/json' };
  const adminToken = localStorage.getItem('admin_token');
  const customerToken = localStorage.getItem('customer_token');
  const cartToken = localStorage.getItem('cart_token');
  if (admin && adminToken) h.Authorization = `Bearer ${adminToken}`;
  if (!admin && customerToken) h.Authorization = `Bearer ${customerToken}`;
  if (!admin && cartToken) h['X-Cart-Token'] = cartToken;
  return h;
}

async function handle(response: Response): Promise<any> {
  const text = await response.text();
  let data: any = null;
  try { data = text ? JSON.parse(text) : null; } catch { /* no JSON */ }
  if (!response.ok) {
    const message = data?.message ?? `HTTP ${response.status}`;
    const error: any = new Error(message);
    error.status = response.status;
    error.errors = data?.errors;
    throw error;
  }
  return data?.data !== undefined ? data.data : data;
}

export const api = {
  async get(path: string): Promise<any> {
    return handle(await fetch(`${BASE}/api/${path}`, { headers: headers() }));
  },
  async adminGet(path: string): Promise<any> {
    return handle(await fetch(`${BASE}/api/${path}`, { headers: headers(true) }));
  },
  async post(path: string, body?: any, admin = false): Promise<any> {
    return handle(await fetch(`${BASE}/api/${path}`, {
      method: 'POST', headers: headers(admin), body: JSON.stringify(body ?? {}),
    }));
  },
  async adminPost(path: string, body?: any): Promise<any> {
    return this.post(path, body, true);
  },
  async put(path: string, body?: any, admin = false): Promise<any> {
    return handle(await fetch(`${BASE}/api/${path}`, {
      method: 'PUT', headers: headers(admin), body: JSON.stringify(body ?? {}),
    }));
  },
  async adminPut(path: string, body?: any): Promise<any> {
    return this.put(path, body, true);
  },
  async adminDelete(path: string): Promise<any> {
    return handle(await fetch(`${BASE}/api/${path}`, { method: 'DELETE', headers: headers(true) }));
  },
  setCartToken(token: string) { localStorage.setItem('cart_token', token); },
  clearCustomerSession() { localStorage.removeItem('customer_token'); },
};
