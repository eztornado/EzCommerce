import { useSearchParams } from 'react-router-dom';
import ProductListing from '../../components/shop/ProductListing';

export default function SearchPage() {
  const [params] = useSearchParams();
  const query = params.get('q') ?? '';
  return (
    <ProductListing
      title={query ? `Results for “${query}”` : 'All products'}
      description={
        query
          ? 'Browse the products that match your search. Refine the results with the filters on the left.'
          : 'Discover the full EzCommerce collection — thoughtfully designed pieces for every day.'
      }
      search={query || undefined}
      eyebrow={query ? 'Search results' : 'Shop all'}
    />
  );
}
