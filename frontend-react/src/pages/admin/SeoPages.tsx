import { useState } from 'react';
import { api } from '../../api/client';
import DataGrid from '../../components/admin/DataGrid';
import GridCrudPage from '../../components/admin/GridCrudPage';
import { AdminPageHeader } from '../../components/admin/ui';

export function UrlRewritesPage() {
  return (
    <GridCrudPage
      title="URL Rewrites"
      subtitle="Map friendly storefront URLs to internal product, category or CMS paths."
      path="admin/seo/url-rewrites"
      createLabel="Create rewrite"
      createEndpoint="admin/seo/url-rewrites/store"
      fields={[
        {
          name: 'entity_type',
          label: 'Entity type',
          type: 'select',
          required: true,
          options: [
            { label: 'Product', value: 'product' },
            { label: 'Category', value: 'category' },
            { label: 'CMS page', value: 'cms' },
          ],
          defaultValue: 'product',
        },
        { name: 'request_path', label: 'Request path', required: true, placeholder: 'summer-sale' },
        { name: 'target_path', label: 'Target path', required: true, placeholder: 'category/4' },
      ]}
      hideColumn={['id']}
      emptyLabel="No URL rewrites found."
    />
  );
}

export function SearchTermsPage() {
  const [version, setVersion] = useState(0);
  const [error, setError] = useState('');

  const toggle = async (row: any) => {
    try {
      await api.adminPut(`admin/seo/search-terms/edit/${row.__primary ?? row.id}`, {
        display_in_suggested_terms: row.display_in_suggested_terms ? 0 : 1,
      });
      setVersion((value) => value + 1);
    } catch (requestError: any) {
      setError(requestError?.message ?? 'Unable to update the search term.');
    }
  };

  return (
    <div>
      <AdminPageHeader title="Search Terms" subtitle="Queries customers use in the storefront search, with their result counts." />
      {error && <div className="admin-alert danger">{error}</div>}
      <div className="admin-card admin-card-pad">
        <DataGrid
          key={version}
          path="admin/seo/search-terms"
          hideColumn={['id']}
          emptyLabel="No search terms recorded yet."
          actions={(row) => (
            <button type="button" className="admin-link" onClick={() => toggle(row)}>
              {row.display_in_suggested_terms ? 'Hide from suggestions' : 'Show in suggestions'}
            </button>
          )}
        />
      </div>
    </div>
  );
}

export function SearchSynonymsPage() {
  return (
    <GridCrudPage
      title="Search Synonyms"
      subtitle="Group related terms so a search for any of them returns the same products."
      path="admin/seo/search-synonyms"
      createLabel="Create synonym"
      createEndpoint="admin/seo/search-synonyms/store"
      fields={[
        { name: 'name', label: 'Name', required: true, placeholder: 'Sneakers' },
        { name: 'terms', label: 'Terms', required: true, placeholder: 'sneaker, trainer, running shoe' },
      ]}
      hideColumn={['id']}
      emptyLabel="No search synonyms found."
    />
  );
}
