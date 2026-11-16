import DataGrid from '../../components/DataGrid';

export function ConfigurationPage() {
  return <DataGrid gridPath="admin/configuration/general.content" title="Configuration (general)" />;
}

export function DataTransferPage() {
  return <DataGrid gridPath="admin/data-transfer/products/export" title="Data Transfer" />;
}
