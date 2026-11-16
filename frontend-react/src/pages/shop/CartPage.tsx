import { useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Container, Table, Text, Button, Group, Loader, Stack, TextInput, NumberInput, Divider, Title } from '@mantine/core';
import { api } from '../../api/client';
import { shopCurrency } from '../../store';

interface CartItem { id: number; name: string; quantity: number; formatted_price: string; formatted_total: string; product_id: number }
interface Cart { items: CartItem[]; formatted_sub_total: string; formatted_tax_total: string; formatted_grand_total: string; coupon_code?: string }

export default function CartPage() {
  const [cart, setCart] = useState<Cart | null>(null);
  const [loading, setLoading] = useState(true);
  const [coupon, setCoupon] = useState('');
  const navigate = useNavigate();

  const refresh = async () => {
    try {
      const data = await api.get(`checkout/cart${cartQuery()}`);
      setCart(data && data.items ? data : null);
    } catch { /* ignore */ }
    setLoading(false);
  };

  const cartQuery = () => {
    const currency = localStorage.getItem('shop_currency');
    return currency ? `?currency=${currency}` : '';
  };

  useEffect(() => { refresh(); }, []);

  const updateQuantity = async (itemId: number, quantity: number) => {
    await api.put('checkout/cart', { qty: [{ cart_item_id: itemId, quantity }] });
    refresh();
  };

  const removeItem = async (itemId: number) => {
    const token = localStorage.getItem('cart_token');
    await fetch('/api/checkout/cart/selected', {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json', 'X-Cart-Token': token ?? '' },
      body: JSON.stringify({ item_ids: [itemId] }),
    });
    refresh();
  };

  const applyCoupon = async () => {
    if (!coupon.trim()) return;
    try { await api.post('checkout/cart/coupon', { code: coupon.trim() }); refresh(); }
    catch { /* ignore */ }
  };

  if (loading) return <Container py="xl"><Loader /></Container>;
  if (!cart || !cart.items?.length) {
    return (
      <Container py="xl" ta="center">
        <Title order={2}>Your cart is empty</Title>
        <Button mt="md" component="a" href="/">Continue shopping</Button>
      </Container>
    );
  }

  return (
    <Container size="lg" py="md">
      <Title order={2} mb="md">Cart</Title>
      <Table verticalSpacing="sm" highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Product</Table.Th><Table.Th>Price</Table.Th><Table.Th w={120}>Qty</Table.Th><Table.Th>Total</Table.Th><Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {cart.items.map((item) => (
            <Table.Tr key={item.id}>
              <Table.Td>
                <Link to={`/product/${item.product_id}`} style={{ color: 'inherit' }}>{item.name}</Link>
              </Table.Td>
              <Table.Td>{item.formatted_price}</Table.Td>
              <Table.Td>
                <NumberInput size="xs" w={90} min={1} value={item.quantity}
                  onChange={(v) => v && updateQuantity(item.id, Number(v))} />
              </Table.Td>
              <Table.Td>{item.formatted_total}</Table.Td>
              <Table.Td>
                <Button size="compact-xs" variant="subtle" color="red"
                  onClick={() => removeItem(item.id)}>Remove</Button>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
      <Divider my="md" />
      <Group justify="space-between">
        <Group>
          <TextInput placeholder="Coupon code" value={coupon} onChange={(e) => setCoupon(e.currentTarget.value)} />
          <Button variant="outline" onClick={applyCoupon}>Apply coupon</Button>
        </Group>
        <Stack gap={4} ta="right">
          <Text>Subtotal: {cart.formatted_sub_total}</Text>
          {cart.formatted_tax_total && <Text>Tax: {cart.formatted_tax_total}</Text>}
          <Title order={4}>Grand total: {cart.formatted_grand_total}</Title>
        </Stack>
      </Group>
      <Group justify="flex-end" mt="lg">
        <Button size="md" onClick={() => navigate('/checkout')}>Proceed to checkout</Button>
      </Group>
    </Container>
  );
}
