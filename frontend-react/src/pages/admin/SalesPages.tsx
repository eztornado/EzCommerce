import DataGrid from '../../components/DataGrid';

export function OrdersPage() {
  return <DataGrid gridPath="admin/sales/orders" title="Orders" />;
}

export function InvoicesPage() {
  return <DataGrid gridPath="admin/sales/invoices" title="Invoices" />;
}

export function ShipmentsPage() {
  return <DataGrid gridPath="admin/sales/shipments" title="Shipments" />;
}

export function RefundsPage() {
  return <DataGrid gridPath="admin/sales/refunds" title="Refunds" />;
}
