// Utilidades compartidas de formato para la tienda y el panel.

export function mediaUrl(path?: string | null): string {
  if (!path) return '';
  if (/^(https?:|data:|blob:)/.test(path)) return path;
  if (path.startsWith('/storage/')) return path;
  if (path.startsWith('storage/')) return `/${path}`;
  if (path.startsWith('/')) return path;
  return `/storage/${path}`;
}

export function stripHtml(value?: string | null): string {
  if (!value) return '';
  return value
    .replace(/<style[\s\S]*?<\/style>/gi, ' ')
    .replace(/<script[\s\S]*?<\/script>/gi, ' ')
    .replace(/<[^>]+>/g, ' ')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/\s+/g, ' ')
    .trim();
}

export function initials(name?: string | null): string {
  if (!name) return '?';
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('');
}

export function money(value?: number | string | null, currency = 'EUR'): string {
  const number = typeof value === 'string' ? Number(value) : value;
  if (number === null || number === undefined || Number.isNaN(number)) return '';
  try {
    return new Intl.NumberFormat('en-GB', {
      style: 'currency',
      currency,
      maximumFractionDigits: 2,
    }).format(number);
  } catch {
    return `${number.toFixed(2)} ${currency}`;
  }
}

export function formattedMoney(formatted?: string | null, amount?: number | string | null, currency = 'EUR'): string {
  const text = String(formatted ?? '').trim();
  if (text && /[^\d.,\s-]/.test(text)) return text;
  return money(amount, currency);
}

export function compactNumber(value?: number | string | null): string {
  const number = typeof value === 'string' ? Number(value) : value;
  if (number === null || number === undefined || Number.isNaN(number)) return '0';
  return new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 }).format(number);
}

export function formatDate(value?: string | null, withTime = false): string {
  if (!value) return '';
  const normalized = value.includes('T') ? value : value.replace(' ', 'T');
  const date = new Date(normalized.endsWith('Z') ? normalized : `${normalized}Z`);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString('en-GB', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
    ...(withTime ? { hour: '2-digit', minute: '2-digit' } : {}),
  });
}

export function statusColor(status?: string | number | null): string {
  const value = String(status ?? '').toLowerCase();
  if (['1', 'active', 'enabled', 'approved', 'completed', 'processing', 'paid', 'delivered', 'success'].includes(value)) {
    return 'green';
  }
  if (['pending', 'new', 'draft', 'on hold', 'on_hold', 'in process'].includes(value)) return 'yellow';
  if (['0', 'inactive', 'disabled', 'canceled', 'cancelled', 'closed', 'failed', 'refunded', 'disapproved', 'suspended'].includes(value)) {
    return 'red';
  }
  if (['shipped', 'invoiced', 'in transit'].includes(value)) return 'blue';
  return 'gray';
}

export function statusLabel(status?: string | number | null): string {
  if (status === 1 || status === '1') return 'Active';
  if (status === 0 || status === '0') return 'Inactive';
  return String(status ?? '—');
}

export function formatLabel(value?: string | null): string {
  if (!value) return '';
  return value
    .replace(/[_-]+/g, ' ')
    .replace(/\b\w/g, (letter) => letter.toUpperCase())
    .trim();
}

export function param(value: string | null | undefined): string {
  return encodeURIComponent(value ?? '');
}
