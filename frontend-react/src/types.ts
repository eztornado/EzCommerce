export interface ProductImageModel {
  url: string;
  alt?: string | null;
  width?: number | null;
  height?: number | null;
  srcset?: string | null;
  avif_srcset?: string | null;
  mime?: string | null;
  variants?: { width: number; height: number; src?: string; avif?: string }[];
}

export interface ProductCard {
  id: number;
  name?: string | null;
  sku?: string | null;
  type?: string;
  url_key?: string | null;
  base_image?: ProductImageModel | null;
  images?: ProductImageModel[];
  formatted_price?: string;
  formatted_regular_price?: string;
  price?: number;
  special_price?: number | null;
  on_sale?: boolean;
  is_new?: boolean;
  is_featured?: boolean;
  in_stock?: boolean;
  quantity?: number;
  short_description?: string | null;
  description?: string | null;
  ratings?: { average: number; total: number };
  reviews?: { total: number };
}

export interface Category {
  id: number;
  name?: string;
  slug?: string;
  url_path?: string;
  banner_url?: string | null;
  image?: string | null;
  description?: string | null;
  children?: Category[];
  status?: number;
  parent_id?: number;
}

export interface CartItem {
  id: number;
  product_id: number;
  name: string;
  sku: string;
  quantity: number;
  base_image?: ProductImageModel | null;
  formatted_price?: string;
  formatted_total?: string;
  price?: number;
  total?: number;
  options?: { label: string; value: string }[];
}

export interface Cart {
  id: number;
  items: CartItem[];
  items_qty: number;
  items_count: number;
  formatted_sub_total?: string;
  formatted_discount_amount?: string;
  formatted_tax_amount?: string;
  formatted_shipping_amount?: string;
  formatted_grand_total?: string;
  sub_total?: number;
  discount_amount?: number;
  tax_total?: number;
  shipping_amount?: number;
  grand_total?: number;
  coupon_code?: string | null;
  have_coupon?: boolean;
}
