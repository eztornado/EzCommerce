import { useEffect, useState } from 'react';
import { SimpleGrid, Card, Text, Title, Loader, Center, Stack } from '@mantine/core';
import { api } from '../../api/client';

export default function DashboardPage() {
  const [stats, setStats] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try { setStats(await api.adminGet('admin/dashboard/stats?type=over-all')); }
      catch { /* ignore */ }
      setLoading(false);
    })();
  }, []);

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
    </Stack>
  );
}
