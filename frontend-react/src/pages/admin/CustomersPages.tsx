import DataGrid from '../../components/DataGrid';

export function CustomersPage() {
  return <DataGrid gridPath="admin/customers" title="Customers" />;
}

export function GroupsPage() {
  return <DataGrid gridPath="admin/customers/groups" title="Customer Groups" />;
}

export function ReviewsPage() {
  return <DataGrid gridPath="admin/customers/reviews" title="Reviews" />;
}
