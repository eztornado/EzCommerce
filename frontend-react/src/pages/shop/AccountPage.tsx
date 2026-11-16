import { useEffect, useState } from 'react';
import { Container, Title, Table, Stack, Card, Text } from '@mantine/core';
import { api } from '../../api/client';

export default function AccountPage() {
  const [orders, setOrders] = useState<any[]>([]);
  const [email, setEmail] = useState('');

  useEffect(() => {
    (async () => {
      try {
        const profile = await api.get('customer/get');
        setEmail(profile?.email ?? '');
        const data = await api.get('customer/orders');
        setOrders(Array.isArray(data) ? data : (data?.data ?? []));
      } catch { /* ignore */ }
    })();
  }, []);

  return (
    <Container size="lg" py="md">
      <Title order={2} mb="md">My account</Title>
      <Stack>
        <Card withBorder>
          <Text fw={500} mb="sm">{email}</Text>
        </Card>
        <Card withBorder>
          <Text fw={500} mb="sm">My orders</Text>
          {orders.length ? (
            <Table>
              <Table.Thead><Table.Tr><Table.Th>Order</Table.Th><Table.Th>Status</Table.Th><Table.Th>Total</Table.Th><Table.Th>Date</Table.Th></Table.Tr></Table.Thead>
              <Table.Tbody>
                {orders.map((order) => (
                  <Table.Tr key={order.id}>
                    <Table.Td>{order.increment_id}</Table.Td>
                    <Table.Td>{order.status}</Table.Td>
                    <Table.Td>{order.formatted_grand_total ?? order.base_grand_total}</Table.Td>
                    <Table.Td>{order.created_at}</Table.Td>
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          ) : <Text c="dimmed" size="sm">No orders yet.</Text>}
        </Card>
      </Stack>
    </Container>
  );
}
