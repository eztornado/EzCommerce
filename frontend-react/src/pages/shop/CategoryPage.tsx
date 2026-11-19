import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { api } from '../../api/client';
import { shopQuery } from '../../store';
import { useShopReload } from '../../lib/hooks';
import ProductListing from '../../components/shop/ProductListing';
import type { Category } from '../../types';

export default function CategoryPage() {
  const { id } = useParams();
  const reload = useShopReload();
  const [category, setCategory] = useState<Category | null>(null);

  useEffect(() => {
    if (!id) return;
    api
      .get(`categories/${id}${shopQuery()}`)
      .then((data) => setCategory(data))
      .catch(() => setCategory(null));
  }, [id, reload]);

  return (
    <ProductListing
      title={category?.name ?? 'Category'}
      description={category?.description ?? undefined}
      categoryId={Number(id)}
      eyebrow="Category"
    />
  );
}
