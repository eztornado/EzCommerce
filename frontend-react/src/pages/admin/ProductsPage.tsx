import { Link, useSearchParams } from 'react-router-dom';
import { IconPlus } from '@tabler/icons-react';
import DataGrid from '../../components/admin/DataGrid';
import { AdminPageHeader } from '../../components/admin/ui';
import { t } from '../../i18n';

export default function ProductsPage() {
  const [params] = useSearchParams();
  return (
    <div>
      <AdminPageHeader
        title="Products"
        subtitle="Manage your catalog: create, edit, price and publish products."
        actions={
          <Link to="/admin/catalog/products/create" className="admin-button primary">
            <IconPlus size={15} /> {t('Create product')}
          </Link>
        }
      />
      <div className="admin-card admin-card-pad">
        <DataGrid path="admin/catalog/products" emptyLabel="No products found. Create your first product to get started." externalSearch={params.get('q') ?? ''} />
      </div>
    </div>
  );
}
