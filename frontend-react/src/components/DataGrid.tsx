import { useEffect, useState } from 'react';
import { Table, TextInput, Button, Group, Text, Loader, Stack, Pagination, ActionIcon, Tooltip } from '@mantine/core';
import { IconSearch, IconDownload, IconEye, IconPencil, IconTrash } from '@tabler/icons-react';
import { api } from '../api/client';

interface DataGridProps {
  gridPath: string;
  title: string;
  onView?: (row: any) => void;
  onEdit?: (row: any) => void;
  onDelete?: (row: any) => void;
  onAdd?: () => void;
  addLabel?: string;
  refreshKey?: number;
}

export default function DataGrid({ gridPath, title, onView, onEdit, onDelete, onAdd, addLabel, refreshKey }: DataGridProps) {
  const [records, setRecords] = useState<any[]>([]);
  const [columns, setColumns] = useState<any[]>([]);
  const [meta, setMeta] = useState<{ total: number; last_page: number }>({ total: 0, last_page: 1 });
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);

  const load = async (pageNumber = page, searchValue = search) => {
    setLoading(true);
    try {
      const data = await api.adminPost(gridPath, {
        filters: searchValue ? { all: searchValue } : {},
        pagination: { page: pageNumber, per_page: 20 },
      });
      setRecords((data?.records ?? []).map((r: any) => ({ ...r, id: r.__primary ?? r.id })));
      setColumns(data?.columns ?? []);
      setMeta({ total: data?.meta?.total ?? 0, last_page: data?.meta?.last_page ?? 1 });
    } catch { /* ignore */ }
    setLoading(false);
  };

  useEffect(() => { load(1, ''); setPage(1); }, [gridPath, refreshKey]);

  const exportCsv = async () => {
    const token = localStorage.getItem('admin_token');
    if (!token) return;
    const response = await fetch(`/api/${gridPath}?format=csv`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify({ filters: search ? { all: search } : {} }),
    });
    if (!response.ok) return;
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${gridPath.split('/').pop()}.csv`;
    anchor.click();
    URL.revokeObjectURL(url);
  };

  const visibleColumns = columns.filter((column) => column.index !== '__primary');

  return (
    <Stack p="md" gap="md">
      <Group justify="space-between">
        <Text size="xl" fw={600}>{title}</Text>
        <Group>
          <TextInput
            leftSection={<IconSearch size={14} />}
            placeholder="Search..."
            value={search}
            onChange={(e) => setSearch(e.currentTarget.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') { setPage(1); load(1, search); } }}
          />
          <Tooltip label="Export CSV"><ActionIcon variant="default" size="lg" onClick={exportCsv}><IconDownload size={16} /></ActionIcon></Tooltip>
          {onAdd && <Button onClick={onAdd}>{addLabel ?? 'New'}</Button>}
        </Group>
      </Group>
      {loading ? (
        <Center><Loader /></Center>
      ) : (
        <Table verticalSpacing="xs" highlightOnHover withTableBorder>
          <Table.Thead bg="gray.0">
            <Table.Tr>
              {visibleColumns.map((column) => (
                <Table.Th key={column.index}>{column.label}</Table.Th>
              ))}
              {(onView || onEdit || onDelete) && <Table.Th w={120}>Actions</Table.Th>}
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {records.map((row) => (
              <Table.Tr key={row.id}>
                {visibleColumns.map((column) => (
                  <Table.Td key={column.index}>{String(row[column.index] ?? '')}</Table.Td>
                ))}
                {(onView || onEdit || onDelete) && (
                  <Table.Td>
                    <Group gap={4}>
                      {onView && <ActionIcon variant="subtle" onClick={() => onView(row)}><IconEye size={16} /></ActionIcon>}
                      {onEdit && <ActionIcon variant="subtle" onClick={() => onEdit(row)}><IconPencil size={16} /></ActionIcon>}
                      {onDelete && <ActionIcon variant="subtle" color="red" onClick={() => onDelete(row)}><IconTrash size={16} /></ActionIcon>}
                    </Group>
                  </Table.Td>
                )}
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      )}
      <Group justify="space-between">
        <Text size="sm" c="dimmed">{meta.total} records</Text>
        <Pagination total={meta.last_page} value={page} onChange={(value) => { setPage(value); load(value); }} size="sm" />
      </Group>
    </Stack>
  );
}

function Center({ children }: { children: React.ReactNode }) {
  return <div style={{ display: 'flex', justifyContent: 'center', padding: 24 }}>{children}</div>;
}
