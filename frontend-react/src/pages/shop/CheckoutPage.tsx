import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Container, Title, TextInput, Select, Button, Loader, Stack, Table, Radio, Group, Alert, Text } from '@mantine/core';
import { api } from '../../api/client';
import { shopCurrency } from '../../store';

export default function CheckoutPage() {
  const navigate = useNavigate();
  const [cart, setCart] = useState<any>(null);
  const [shippingMethods, setShippingMethods] = useState<any[]>([]);
  const [paymentMethods, setPaymentMethods] = useState<any[]>([]);
  const [shipping, setShipping] = useState('');
  const [payment, setPayment] = useState('');
  const [placing, setPlacing] = useState(false);
  const [billing, setBilling] = useState({
    first_name: '', last_name: '', email: '', address: '', city: '', postcode: '', country: 'ES', state: '', phone: '',
  });

  useEffect(() => { (async () => {
    try {
      const currency = shopCurrency();
      const q = currency ? `?currency=${currency}` : '';
      const data = await api.get(`checkout/cart${q}`);
      setCart(data && data.items ? data : null);
    } catch { /* ignore */ }
  })(); }, []);

  const saveAddresses = async () => {
    await api.post('checkout/onepage/addresses', { billing, shipping: billing });
    const methods = await api.get('checkout/onepage/shipping-methods');
    setShippingMethods(methods?.shippingMethods ?? []);
  };

  const chooseShipping = async (method: string) => {
    setShipping(method);
    const data = await api.post('checkout/onepage/shipping-methods', { shipping_method: method });
    setPaymentMethods(data?.payment_methods ?? []);
    setPayment(data?.payment_methods?.[0]?.method ?? '');
  };

  const savePayment = async () => {
    if (!payment) return;
    await api.post('checkout/onepage/payment-methods', { payment });
    loadCart();
  };

  const loadCart = async () => {
    const currency = shopCurrency();
    const q = currency ? `?currency=${currency}` : '';
    const data = await api.get(`checkout/cart${q}`);
    setCart(data);
  };

  const placeOrder = async () => {
    setPlacing(true);
    try {
      const data = await api.post('checkout/onepage/orders');
      navigate(`/checkout/success/${data.order?.increment_id ?? data.increment_id ?? ''}`);
    } finally { setPlacing(false); }
  };

  if (!cart) return <Container py="xl"><Loader /></Container>;
  if (!cart.items?.length) {
    return <Container py="xl"><Alert>Your cart is empty.</Alert></Container>;
  }

  return (
    <Container size="md" py="md">
      <Title order={2} mb="md">Checkout</Title>
      <Stack gap="lg">
        <Stack gap="sm">
          <Title order={4}>1. Billing & shipping address</Title>
          <Group grow>
            <TextInput label="First name" value={billing.first_name} onChange={(e) => setBilling({ ...billing, first_name: e.currentTarget.value })} />
            <TextInput label="Last name" value={billing.last_name} onChange={(e) => setBilling({ ...billing, last_name: e.currentTarget.value })} />
          </Group>
          <TextInput label="Email" value={billing.email} onChange={(e) => setBilling({ ...billing, email: e.currentTarget.value })} />
          <TextInput label="Address" value={billing.address} onChange={(e) => setBilling({ ...billing, address: e.currentTarget.value })} />
          <Group grow>
            <TextInput label="City" value={billing.city} onChange={(e) => setBilling({ ...billing, city: e.currentTarget.value })} />
            <TextInput label="Postcode" value={billing.postcode} onChange={(e) => setBilling({ ...billing, postcode: e.currentTarget.value })} />
          </Group>
          <Group grow>
            <TextInput label="Country (ISO)" value={billing.country} onChange={(e) => setBilling({ ...billing, country: e.currentTarget.value })} />
            <TextInput label="State" value={billing.state} onChange={(e) => setBilling({ ...billing, state: e.currentTarget.value })} />
          </Group>
          <Button onClick={saveAddresses} variant="light">Save addresses</Button>
        </Stack>

        <Stack gap="sm">
          <Title order={4}>2. Shipping method</Title>
          {shippingMethods.map((method) => (
            <Radio key={method.method} value={method.method} label={`${method.carrier_title}: ${method.base_price} EUR`}
              checked={shipping === method.method} onChange={() => chooseShipping(method.method)} />
          ))}
          {!shippingMethods.length && <Text c="dimmed" size="sm">Save the address to load shipping methods.</Text>}
        </Stack>

        <Stack gap="sm">
          <Title order={4}>3. Payment method</Title>
          {paymentMethods.map((method) => (
            <Radio key={method.method} value={method.method} label={method.method_title}
              checked={payment === method.method} onChange={() => setPayment(method.method)} />
          ))}
          {!paymentMethods.length && <Text c="dimmed" size="sm">Select a shipping method first.</Text>}
          {payment && <Button variant="light" onClick={savePayment}>Confirm payment method</Button>}
        </Stack>

        <Stack gap="sm">
          <Title order={4}>4. Review & place order</Title>
          <Table>
            <Table.Tbody>
              <Table.Tr><Table.Td>Subtotal</Table.Td><Table.Td>{cart.formatted_sub_total}</Table.Td></Table.Tr>
              <Table.Tr><Table.Td>Shipping</Table.Td><Table.Td>{cart.formatted_shipping_amount}</Table.Td></Table.Tr>
              <Table.Tr><Table.Td>Tax</Table.Td><Table.Td>{cart.formatted_tax_total}</Table.Td></Table.Tr>
              <Table.Tr><Table.Td fw={600}>Grand total</Table.Td><Table.Td fw={600}>{cart.formatted_grand_total}</Table.Td></Table.Tr>
            </Table.Tbody>
          </Table>
          <Button size="md" loading={placing} onClick={placeOrder}>Place order</Button>
        </Stack>
      </Stack>
    </Container>
  );
}
