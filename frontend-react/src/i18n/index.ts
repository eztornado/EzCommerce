// Mini motor de traducciones de la interfaz (tienda + panel).
//
// Las claves son el propio texto en inglés, de modo que el código sigue siendo
// legible y cualquier cadena sin traducir cae automáticamente al inglés. El
// idioma activo lo determina la configuración general
// (`general.general.locale_options.ui_locale`) que expone `/api/core/context`.

import { useSyncExternalStore } from 'react';
import es from './es';
import ja from './ja';

export const UI_LOCALES = [
  { code: 'en', label: 'English', native: 'English' },
  { code: 'es', label: 'Spanish', native: 'Español' },
  { code: 'ja', label: 'Japanese', native: '日本語' },
] as const;

export type UiLocale = (typeof UI_LOCALES)[number]['code'];

const DICTIONARIES: Record<UiLocale, Record<string, string>> = { en: {}, es, ja };

export function isUiLocale(code?: string | null): code is UiLocale {
  return code === 'en' || code === 'es' || code === 'ja';
}

export function uiLocaleName(code?: string | null): string {
  return UI_LOCALES.find((entry) => entry.code === code)?.native ?? 'English';
}

function detectInitialLocale(): UiLocale {
  if (typeof navigator !== 'undefined') {
    const language = navigator.language.slice(0, 2).toLowerCase();
    if (isUiLocale(language)) return language;
  }
  return 'en';
}

let activeLocale: UiLocale = detectInitialLocale();
const listeners = new Set<() => void>();
if (typeof document !== 'undefined') document.documentElement.lang = activeLocale;

export function getLocale(): UiLocale {
  return activeLocale;
}

export function setLocale(code?: string | null): void {
  const next = isUiLocale(code) ? code : 'en';
  if (next === activeLocale) return;
  activeLocale = next;
  if (typeof document !== 'undefined') document.documentElement.lang = next;
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useLocale(): UiLocale {
  return useSyncExternalStore(subscribe, getLocale, getLocale);
}

/** Traduce una cadena. Admite interpolación con `{nombre}`. */
export function t(key: string, vars?: Record<string, string | number | null | undefined>): string {
  const dictionary = DICTIONARIES[activeLocale];
  let text = dictionary[key] ?? key;
  if (vars) {
    text = text.replace(/\{(\w+)\}/g, (match, name: string) => {
      const value = vars[name];
      return value === null || value === undefined ? match : String(value);
    });
  }
  return text;
}

/** Código BCP-47 para `Intl` (fechas, números y moneda). */
export function intlLocale(): string {
  return activeLocale === 'es' ? 'es-ES' : activeLocale === 'ja' ? 'ja-JP' : 'en-GB';
}
