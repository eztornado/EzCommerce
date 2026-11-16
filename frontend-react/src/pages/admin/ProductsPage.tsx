import { useEffect, useState } from 'react';
import { Table, Title, Loader, Button, Stack, TextInput, Group } from '@mantine/core';
import { api } from '../../api/client';

type ProductRecord = { [key: string]: any };

export default function ProductsPage() {
  const [records, setRecords] = useState<ProductRecord[]>([]);
  const [columns, setColumns] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');

  const load = async (searchValue = '') => {
    setLoading(true);
    try {
      const data = await api.adminPost('admin/catalog/products', {
        filters: searchValue ? { all: searchValue } : {},
        pagination: { page: 1, per_page: 20 },
      });
      setRecords((data?.records ?? []).map((r: any) => ({ ...r, id: r.__primary ?? r.id })));
      setColumns(data?.columns ?? []);
    } catch { /* ignore */ }
    setLoading(false);
  };

  useEffect(() => { load(); }, []);

  return (
    <Stack p="md">
      <Group justify="space-between">
        <Title order={2}>Products</Title>
        <TextInput placeholder="Search..." value={search} onChange={(e) => setSearch(e.currentTarget.value)}
          onKeyDown={(e) => e.key === 'Enter' && load(search)} />
        <Button onClick={() => load(search)}>Search</Button>
      </Group>
      {loading ? (
        <Center><Loader /></Center>
      ) : (
        <Table verticalSpacing="xs" highlightOnHover>
          <Table.Thead>
            <Table.Tr>{columns.filter((c) => c.index !== '__primary').map((column) => (
              <Table.Th key={column.index}>{column.label}</Table.Th>
            ))}<Table.Th /></Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {records.map((row) => (
              <Table.Tr key={row.id}>
                {columns.filter((c) => c.index !== '__primary').map((column) => (
                  <Table.Td key={column.index}>{String(row[column.index] ?? '')}</Table.Td>
                ))}
                <Table.Td>
                  <Button size="compact-xs" variant="light"
                    onClick={() => window.open(`/admin/products/edit/${row.id}`, '_blank')}>Edit</Button>
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      )}
    </Stack>
  );
}

function Center({ children }: { children: React.ReactNode }) {
  return <div style={{ display: 'flex', justifyContent: 'center', padding: 24 }}>{children}</div>;
}
