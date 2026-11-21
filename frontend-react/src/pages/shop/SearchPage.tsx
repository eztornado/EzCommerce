import { useSearchParams } from 'react-router-dom';
import ProductListing from '../../components/shop/ProductListing';
import { t } from '../../i18n';

export default function SearchPage() {
  const [params] = useSearchParams();
  const query = params.get('q') ?? '';
  return (
    <ProductListing
      title={query ? t('Results for “{query}”', { query }) : t('All products')}
      description={
        query
          ? t('Browse the products that match your search. Refine the results with the filters on the left.')
          : t('Discover the full EzCommerce collection — thoughtfully designed pieces for every day.')
      }
      search={query || undefined}
      eyebrow={query ? t('Search results') : t('Shop all')}
    />
  );
}
