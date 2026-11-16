import DataGrid from '../../components/DataGrid';

export function CartRulesPage() {
  return <DataGrid gridPath="admin/marketing/cart-rules" title="Cart Rules" />;
}

export function CatalogRulesPage() {
  return <DataGrid gridPath="admin/marketing/catalog-rules" title="Catalog Rules" />;
}

export function CategoriesPage() {
  return <DataGrid gridPath="admin/catalog/categories" title="Categories" />;
}

export function AttributesPage() {
  return <DataGrid gridPath="admin/catalog/attributes" title="Attributes" />;
}

export function FamiliesPage() {
  return <DataGrid gridPath="admin/catalog/families" title="Attribute Families" />;
}

export function CustomersPage() {
  return <DataGrid gridPath="admin/customers" title="Customers" />;
}

export function GroupsPage() {
  return <DataGrid gridPath="admin/customers/groups" title="Customer Groups" />;
}

export function ReviewsPage() {
  return <DataGrid gridPath="admin/customers/reviews" title="Reviews" />;
}

export function InventorySourcesPage() {
  return <DataGrid gridPath="admin/settings/inventory-sources" title="Inventory Sources" />;
}

export function ReportingPage() {
  return <DataGrid gridPath="admin/reporting/products" title="Reporting (top products)" />;
}
