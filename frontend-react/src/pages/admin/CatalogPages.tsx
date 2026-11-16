import DataGrid from '../../components/DataGrid';

export function CategoriesPage() {
  return <DataGrid gridPath="admin/catalog/categories" title="Categories" />;
}

export function AttributesPage() {
  return <DataGrid gridPath="admin/catalog/attributes" title="Attributes" />;
}

export function FamiliesPage() {
  return <DataGrid gridPath="admin/catalog/families" title="Attribute Families" />;
}
