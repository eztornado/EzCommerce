import { Link } from 'react-router-dom';
import DataGrid from '../../components/admin/DataGrid';
import { AdminPageHeader } from '../../components/admin/ui';
import { t } from '../../i18n';

export function CartRulesPage() {
  return (
    <div>
      <AdminPageHeader
        title="Cart Rules"
        subtitle="Automatic discounts and coupons applied to the shopping cart."
        actions={
          <Link to="/admin/marketing/promotions/cart-rules/create" className="admin-button primary">
            {t('Create cart rule')}
          </Link>
        }
      />
      <div className="admin-card admin-card-pad">
        <DataGrid
          path="admin/marketing/cart-rules"
          emptyLabel="No cart rules found."
          hideActions={['Edit']}
          actions={(record) => (
            <Link className="admin-link" to={`/admin/marketing/promotions/cart-rules/edit/${record.__primary ?? record.id}`}>
              {t('Edit')}
            </Link>
          )}
        />
      </div>
    </div>
  );
}

export function CatalogRulesPage() {
  return (
    <div>
      <AdminPageHeader
        title="Catalog Rules"
        subtitle="Price rules applied automatically to products in the catalog."
        actions={
          <Link to="/admin/marketing/promotions/catalog-rules/create" className="admin-button primary">
            {t('Create catalog rule')}
          </Link>
        }
      />
      <div className="admin-card admin-card-pad">
        <DataGrid
          path="admin/marketing/catalog-rules"
          emptyLabel="No catalog rules found."
          actions={(record) => (
            <Link className="admin-link" to={`/admin/marketing/promotions/catalog-rules/edit/${record.__primary ?? record.id}`}>
              {t('Edit')}
            </Link>
          )}
        />
      </div>
    </div>
  );
}
