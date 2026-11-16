import DataGrid from '../../components/DataGrid';

export function CartRulesPage() {
  return <DataGrid gridPath="admin/marketing/cart-rules" title="Cart Rules" />;
}

export function CatalogRulesPage() {
  return <DataGrid gridPath="admin/marketing/catalog-rules" title="Catalog Rules" />;
}

export function SearchSeoPage() {
  return <DataGrid gridPath="admin/seo/search-terms" title="Search Terms" />;
}

export function CommunicationsPage() {
  return <DataGrid gridPath="admin/communications/campaigns" title="Campaigns" />;
}
