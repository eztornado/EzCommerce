import { useEffect, useState } from 'react';
import { Container, Title, SegmentedControl, Table, Stack, SimpleGrid, Card, Button, Text, Group } from '@mantine/core';
import { api } from '../../api/client';

export default function ReportingPage() {
  const [module, setModule] = useState('sales');
  const [range, setRange] = useState('last30');
  const [sales, setSales] = useState<any>(null);
  const [rows, setRows] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    setLoading(true);
    const start = range === 'year' ? `${new Date().getFullYear()}-01-01` : new Date(Date.now() - 30 * 86400000).toISOString().slice(0, 10);
    const end = new Date().toISOString().slice(0, 10);
    const query = `?start=${start}&end=${end}`;
    try {
      if (module === 'sales') {
        setSales(await api.adminGet(`admin/reporting/sales${query}&granularity=day`)); setRows([]);
      } else {
        const data = await api.adminGet(`admin/reporting/${module}${query}`);
        setRows(Array.isArray(data) ? data : data?.data ?? []);
        setSales(null);
      }
    } catch { /* ignore */ }
    setLoading(false);
  };
  useEffect(() => { load(); }, [module, range]);

  const max = Math.max(...(sales?.series ?? []).map((point: any) => point.sales), 1);

  return (
    <Container size="xl" py="md">
      <Title order={2} mb="md">Reporting</Title>
      <Group mb="md">
        <SegmentedControl value={module} onChange={setModule} data={[
          { value: 'sales', label: 'Sales' }, { value: 'products', label: 'Products' }, { value: 'customers', label: 'Customers' },
        ]} />
        <SegmentedControl value={range} onChange={setRange} data={[
          { value: 'last30', label: 'Last 30 days' }, { value: 'year', label: 'This year' },
        ]} />
        <Button variant="light" onClick={load}>Refresh</Button>
      </Group>
      {loading ? <Text>Loading...</Text> : module === 'sales' ? (
        <Stack>
          <SimpleGrid cols={{ base: 2, lg: 4 }}>
            {['total_sales', 'total_orders', 'tax_collected', 'refunds'].map((key) => (
              <Card key={key} withBorder radius="md" p="md">
                <Text size="xs" c="dimmed" tt="uppercase">{key.replace('_', ' ')}</Text>
                <Text size="lg" fw={600} mt={4}>{sales?.totals?.[key] ?? 0}</Text>
              </Card>
            ))}
          </SimpleGrid>
          <Card withBorder p="md">
            <Text fw={500} mb="sm">Sales over time</Text>
            {(sales?.series ?? []).length === 0 ? <Text c="dimmed">No data.</Text> : (
              <div style={{ display: 'flex', alignItems: 'flex-end', gap: 4, height: 180 }}>
                {(sales?.series ?? []).map((point: any) => (
                  <div key={point.label} style={{ flex: 1, textAlign: 'center' }}>
                    <div style={{ height: `${Math.max((point.sales / max) * 150, 4)}px`, background: '#228be6', borderRadius: '4px 4px 0 0' }} title={`${point.label}: ${point.sales}`} />
                    <Text size="9px" c="dimmed">{point.label.slice(5)}</Text>
                  </div>
                ))}
              </div>
            )}
          </Card>
        </Stack>
      ) : (
        <Table verticalSpacing="xs" withTableBorder>
          <Table.Thead bg="gray.0"><Table.Tr><Table.Th>{module === 'products' ? 'Product' : 'Customer'}</Table.Th><Table.Th>{module === 'products' ? 'Qty' : 'Orders'}</Table.Th><Table.Th>Revenue</Table.Th></Table.Tr></Table.Thead>
          <Table.Tbody>{rows.map((row) => (
            <Table.Tr key={row.id ?? row.product_id}>
              <Table.Td>{row.name ?? `${row.first_name} ${row.last_name}`}</Table.Td>
              <Table.Td>{row.qty ?? row.orders}</Table.Td>
              <Table.Td>{Number(row.total ?? 0).toFixed(2)} EUR</Table.Td>
            </Table.Tr>
          ))}</Table.Tbody>
        </Table>
      )}
    </Container>
  );
}
