import { useEffect, useState } from 'react';

/** Re-renderiza cuando cambia el idioma o la moneda de la tienda. */
export function useShopReload(): number {
  const [key, setKey] = useState(0);
  useEffect(() => {
    const handler = () => setKey((value) => value + 1);
    window.addEventListener('shop-reload', handler);
    return () => window.removeEventListener('shop-reload', handler);
  }, []);
  return key;
}

/** Avisa al layout de que el carrito ha cambiado. */
export function notifyCart() {
  window.dispatchEvent(new Event('cart-updated'));
}

export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => (typeof window === 'undefined' ? false : window.matchMedia(query).matches));
  useEffect(() => {
    const list = window.matchMedia(query);
    const handler = () => setMatches(list.matches);
    handler();
    list.addEventListener('change', handler);
    return () => list.removeEventListener('change', handler);
  }, [query]);
  return matches;
}
