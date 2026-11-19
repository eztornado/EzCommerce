import { useNavigate } from 'react-router-dom';
import DataGrid from '../../components/admin/DataGrid';
import { AdminPageHeader } from '../../components/admin/ui';

function SalesGrid({
  path,
  title,
  subtitle,
  view,
}: {
  path: string;
  title: string;
  subtitle: string;
  view?: (record: any) => string;
}) {
  const navigate = useNavigate();
  return (
    <div>
      <AdminPageHeader title={title} subtitle={subtitle} />
      <div className="admin-card admin-card-pad">
        <DataGrid
          path={path}
          emptyLabel={`No ${title.toLowerCase()} found.`}
          onRowClick={view ? (record) => navigate(view(record)) : undefined}
        />
      </div>
    </div>
  );
}

export function OrdersPage() {
  return (
    <SalesGrid
      path="admin/sales/orders"
      title="Orders"
      subtitle="Track, invoice, ship and refund customer orders."
      view={(record) => `/admin/sales/orders/view/${record.__primary ?? record.id}`}
    />
  );
}

export function InvoicesPage() {
  return (
    <SalesGrid
      path="admin/sales/invoices"
      title="Invoices"
      subtitle="Every invoice raised against an order."
      view={(record) => `/admin/sales/orders/view/${record.order_id ?? ''}`}
    />
  );
}

export function ShipmentsPage() {
  return (
    <SalesGrid
      path="admin/sales/shipments"
      title="Shipments"
      subtitle="Parcels sent to customers and their tracking numbers."
      view={(record) => `/admin/sales/orders/view/${record.order_id ?? ''}`}
    />
  );
}

export function RefundsPage() {
  return (
    <SalesGrid
      path="admin/sales/refunds"
      title="Refunds"
      subtitle="Refunds issued for returned or cancelled items."
      view={(record) => `/admin/sales/orders/view/${record.order_id ?? ''}`}
    />
  );
}
