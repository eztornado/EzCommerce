import { api } from '../api/client';
import { shopQuery, useCart } from '../store';
import { notifyCart } from './hooks';

/** Añade un producto al carrito guardando el token de invitado si lo hay. */
export async function addToCart(productId: number, quantity = 1, options: Record<string, unknown> = {}) {
  const data = await api.post(`checkout/cart${shopQuery()}`, {
    product_id: productId,
    quantity,
    ...options,
  });
  if (data?.cart_token) api.setCartToken(data.cart_token);
  await useCart.getState().refresh();
  notifyCart();
  return data;
}

export async function updateCartQty(cartItemId: number, quantity: number) {
  const data = await api.put(`checkout/cart${shopQuery()}`, {
    qty: [{ cart_item_id: cartItemId, quantity }],
  });
  notifyCart();
  return data;
}

export async function removeCartItem(cartItemId: number) {
  const data = await api.delete(`checkout/cart/selected${shopQuery()}`, false, { item_ids: [cartItemId] });
  notifyCart();
  return data;
}

export const fallbackImage =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="600" height="600"><rect width="600" height="600" fill="#f5f5f4"/><g fill="none" stroke="#d6d3d1" stroke-width="10"><rect x="175" y="195" width="250" height="210" rx="16"/><circle cx="245" cy="265" r="26"/><path d="M185 380l80-90 60 60 45-40 45 70"/></g></svg>`,
  );
