// Cliente API del backend Rust (Larust). Mantiene los tokens JWT de admin y
// cliente, además del token firmado del carrito de invitado, en localStorage.

const BASE = (import.meta.env.VITE_API_BASE as string | undefined) ?? '';

export const apiUrl = (path: string) => `${BASE}/api/${path.replace(/^\/+/, '')}`;

export function queryString(params: Record<string, string | number | null | undefined>): string {
  const search = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value !== null && value !== undefined && String(value) !== '') {
      search.set(key, String(value));
    }
  });
  const result = search.toString();
  return result ? `?${result}` : '';
}

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
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    /* respuesta sin JSON */
  }
  if (!response.ok) {
    const message = data?.message ?? `HTTP ${response.status}`;
    const error: any = new Error(message);
    error.status = response.status;
    error.errors = data?.errors;
    throw error;
  }
  // Bagisto envuelve los recursos en `{data, message}`. El cliente devuelve
  // directamente el recurso, salvo en las respuestas ya planas.
  return data?.data !== undefined ? data.data : data;
}

async function request(path: string, init: RequestInit, admin = false): Promise<any> {
  return handle(await fetch(apiUrl(path), { ...init, headers: { ...headers(admin), ...(init.headers ?? {}) } }));
}

export const api = {
  get(path: string): Promise<any> {
    return request(path, { method: 'GET' });
  },
  adminGet(path: string): Promise<any> {
    return request(path, { method: 'GET' }, true);
  },
  post(path: string, body?: any, admin = false): Promise<any> {
    return request(path, { method: 'POST', body: JSON.stringify(body ?? {}) }, admin);
  },
  adminPost(path: string, body?: any): Promise<any> {
    return request(path, { method: 'POST', body: JSON.stringify(body ?? {}) }, true);
  },
  put(path: string, body?: any, admin = false): Promise<any> {
    return request(path, { method: 'PUT', body: JSON.stringify(body ?? {}) }, admin);
  },
  adminPut(path: string, body?: any): Promise<any> {
    return request(path, { method: 'PUT', body: JSON.stringify(body ?? {}) }, true);
  },
  patch(path: string, body?: any, admin = false): Promise<any> {
    return request(path, { method: 'PATCH', body: JSON.stringify(body ?? {}) }, admin);
  },
  adminPatch(path: string, body?: any): Promise<any> {
    return request(path, { method: 'PATCH', body: JSON.stringify(body ?? {}) }, true);
  },
  delete(path: string, admin = false, body?: any): Promise<any> {
    return request(path, { method: 'DELETE', ...(body !== undefined ? { body: JSON.stringify(body) } : {}) }, admin);
  },
  adminDelete(path: string): Promise<any> {
    return request(path, { method: 'DELETE' }, true);
  },
  setCartToken(token: string) {
    localStorage.setItem('cart_token', token);
  },
  clearCustomerSession() {
    localStorage.removeItem('customer_token');
  },
  clearAdminSession() {
    localStorage.removeItem('admin_token');
  },
  /** Descarga el CSV de un datagrid (los grids del admin lo generan con ?format=csv). */
  async downloadGridCsv(gridPath: string, body: any, filename = 'export.csv') {
    const token = localStorage.getItem('admin_token');
    const response = await fetch(apiUrl(`${gridPath}${gridPath.includes('?') ? '&' : '?'}format=csv`), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'text/csv',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify(body ?? {}),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = filename;
    anchor.click();
    URL.revokeObjectURL(url);
  },
  /** Descarga cualquier documento del admin (CSV de exportación, etc.). */
  async downloadAdminFile(path: string, filename = 'export.csv', init?: RequestInit) {
    const clean = path.replace(/^\/?api\//, '');
    const token = localStorage.getItem('admin_token');
    const response = await fetch(apiUrl(clean), {
      ...init,
      headers: {
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(init?.headers ?? {}),
      },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = filename;
    anchor.click();
    URL.revokeObjectURL(url);
  },
  /** Abre un documento del admin (facturas, albaranes) con el token adjunto. */
  async openAdminDocument(path: string) {
    const clean = path.replace(/^\/?api\//, '');
    const token = localStorage.getItem('admin_token');
    const response = await fetch(apiUrl(clean), {
      headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    window.open(url, '_blank', 'noopener');
    window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
  },
  /** Sube un fichero como base64 y devuelve la ruta pública. */
  async upload(file: File, folder = 'products'): Promise<any> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(new Error('Unable to read the file.'));
      reader.onload = async () => {
        const result = String(reader.result ?? '');
        const base64 = result.includes(',') ? result.split(',')[1] : result;
        try {
          resolve(await api.adminPost('admin/upload', { filename: file.name, folder, content: base64 }));
        } catch (error) {
          reject(error);
        }
      };
      reader.readAsDataURL(file);
    });
  },
};
