import { useEffect, useState } from 'react';
import { Container, Title, Table, Stack, Text, Loader, Button, Group } from '@mantine/core';
import { api } from '../../api/client';

export default function OrderDetailPage() {
  const [, setId] = useState(window.location.pathname.split('/').pop());
  const [order, setOrder] = useState<any>(null);
  const orderId = window.location.pathname.split('/').pop();

  useEffect(() => {
    (async () => {
      try { setOrder(await api.adminGet(`admin/sales/orders/view/${orderId}`)); }
      catch { /* ignore */ }
    })();
  }, [orderId]);

  if (!order) return <Container py="xl"><Loader /></Container>;
  const items = order.items ?? [];

  return (
    <Container size="lg" py="md">
      <Title order={2} mb="md">Order {order.increment_id ?? orderId}</Title>
      <Text>Status: {order.status} · Total: {order.formatted_grand_total ?? order.base_grand_total}</Text>
      <Text c="dimmed" size="sm">{order.customer_email}</Text>
      <Table verticalSpacing="xs" withTableBorder mt="md">
        <Table.Thead bg="gray.0"><Table.Tr><Table.Th>Item</Table.Th><Table.Th>Qty</Table.Th><Table.Th>Total</Table.Th></Table.Tr></Table.Thead>
        <Table.Tbody>{items.map((item: any) => (
          <Table.Tr key={item.id}><Table.Td>{item.name}</Table.Td><Table.Td>{item.qty_ordered}</Table.Td><Table.Td>{item.formatted_total ?? item.base_total}</Table.Td></Table.Tr>
        ))}</Table.Tbody>
      </Table>
      {(order.invoices ?? []).length > 0 && (
        <Group mt="md">
          {order.invoices.map((invoice: any) => (
            <Button key={invoice.id} variant="light"
              onClick={() => window.open(`/api/admin/sales/invoices/print/${invoice.id}`, '_blank')}>
              Print invoice {invoice.increment_id ?? invoice.id}
            </Button>
          ))}
        </Group>
      )}
    </Container>
  );
}
