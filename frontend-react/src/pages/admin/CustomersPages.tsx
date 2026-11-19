import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { IconCheck, IconX } from '@tabler/icons-react';
import DataGrid from '../../components/admin/DataGrid';
import ResourcePage from '../../components/admin/ResourcePage';
import { AdminPageHeader } from '../../components/admin/ui';
import { api } from '../../api/client';

export function CustomersPage() {
  const navigate = useNavigate();
  return (
    <div>
      <AdminPageHeader
        title="Customers"
        subtitle="Manage customer accounts, groups and their order history."
      />
      <div className="admin-card admin-card-pad">
        <DataGrid
          path="admin/customers"
          hideColumn={['id']}
          emptyLabel="No customers found."
          onRowClick={(record) => navigate(`/admin/customers/view/${record.__primary ?? record.id}`)}
        />
      </div>
    </div>
  );
}

export function GroupsPage() {
  return (
    <ResourcePage
      title="Customer Groups"
      subtitle="Group customers to target promotions, pricing and campaigns."
      endpoint="admin/customers/groups"
      createLabel="Create group"
      fields={[
        { name: 'id', label: 'ID', formHidden: true },
        { name: 'code', label: 'Code', required: true, placeholder: 'vip' },
        { name: 'name', label: 'Name', required: true, placeholder: 'VIP customers' },
        {
          name: 'is_user_defined',
          label: 'System',
          type: 'checkbox',
          hideInTable: false,
          render: (value) => (
            <span className={`status-badge ${value ? 'gray' : 'blue'}`}>{value ? 'User defined' : 'System'}</span>
          ),
        },
      ]}
      columns={['id', 'code', 'name', 'is_user_defined']}
      createEndpoint="admin/customers/groups/store"
      updateEndpoint="admin/customers/groups/edit/{id}"
      deleteEndpoint="admin/customers/groups/{id}"
      transform={(values, editing) => ({
        code: values.code,
        name: values.name,
        ...(editing ? {} : { is_user_defined: 1 }),
      })}
      emptyLabel="No customer groups found."
    />
  );
}

export function ReviewsPage() {
  const [version, setVersion] = useState(0);

  const moderate = async (id: number, status: string) => {
    try {
      await api.adminPut(`admin/customers/reviews/${id}/status`, { status });
      setVersion((value) => value + 1);
    } catch (error: any) {
      window.alert(error?.message ?? 'Unable to update the review.');
    }
  };

  return (
    <div>
      <AdminPageHeader
        title="Reviews"
        subtitle="Moderate product reviews before they appear in the storefront."
      />
      <div className="admin-card admin-card-pad">
        <DataGrid
          key={version}
          path="admin/customers/reviews"
          emptyLabel="No reviews found."
          actions={(record) => (
            <>
              <button type="button" className="admin-icon-action" title="Approve" onClick={() => moderate(record.__primary ?? record.id, 'approved')}>
                <IconCheck size={15} />
              </button>
              <button type="button" className="admin-icon-action danger" title="Reject" onClick={() => moderate(record.__primary ?? record.id, 'rejected')}>
                <IconX size={15} />
              </button>
            </>
          )}
        />
      </div>
    </div>
  );
}
