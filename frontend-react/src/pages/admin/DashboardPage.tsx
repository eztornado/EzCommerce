import { useEffect, useState } from 'react';
import { SimpleGrid, Card, Text, Title, Loader, Center, Stack } from '@mantine/core';
import { api } from '../../api/client';

export default function DashboardPage() {
  const [stats, setStats] = useState<any>(null);
  const [series, setSeries] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try { setStats(await api.adminGet('admin/dashboard/stats?type=over-all')); }
      catch { /* ignore */ }
      try {
        const report = await api.adminGet('admin/reporting/sales');
        setSeries(report?.series ?? []);
      } catch { /* ignore */ }
      setLoading(false);
    })();
  }, []);

  const max = Math.max(...series.map((point) => point.sales ?? 0), 1);

  if (loading) return <Center py="xl"><Loader /></Center>;

  const metric = (key: string) => stats?.[key]?.current ?? 0;
  const progress = (key: string) => Number(stats?.[key]?.progress ?? 0);

  return (
    <Stack p="md">
      <Title order={2}>Dashboard</Title>
      <SimpleGrid cols={{ base: 1, sm: 2, lg: 4 }} spacing="md">
        {['total_sales', 'total_orders', 'total_customers', 'avg_sales'].map((key) => (
          <Card key={key} withBorder radius="md" p="md">
            <Text size="xs" c="dimmed" tt="uppercase">{key.replace('_', ' ')}</Text>
            <Text size="xl" fw={600} mt={4}>{metric(key)}</Text>
            <Text size="xs" c={progress(key) >= 0 ? 'teal' : 'red'} mt={4}>
              {progress(key) >= 0 ? '+' : ''}{progress(key).toFixed(1)}% vs last period
            </Text>
          </Card>
        ))}
      </SimpleGrid>
          <Card withBorder radius="md" p="md" mt="md">
            <Text size="sm" fw={500} mb="md">Sales — last 30 days</Text>
            {series.length === 0 ? (
              <Text size="sm" c="dimmed">No sales in this period.</Text>
            ) : (
              <div style={{ display: 'flex', alignItems: 'flex-end', gap: 4, height: 180 }}>
                {series.map((point) => (
                  <div key={point.label} style={{ flex: 1, textAlign: 'center' }}>
                    <div style={{ height: `${Math.max((point.sales / max) * 150, 4)}px`, background: '#228be6', borderRadius: '4px 4px 0 0' }} title={`${point.label}: ${point.sales}`} />
                    <Text size="9px" c="dimmed">{String(point.label).slice(5)}</Text>
                  </div>
                ))}
              </div>
            )}
          </Card>
    </Stack>
  );
}
