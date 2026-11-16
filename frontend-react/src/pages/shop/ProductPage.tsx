import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Group, Image, Text, Select, NumberInput, Button, SimpleGrid, Stack, Card, Table, Alert, Loader, Center } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { api } from '../../api/client';
import { shopQuery } from '../../store';

interface SuperAttr { id: number; code: string; label: string; options: { id: number; admin_name: string }[] }

export default function ProductPage() {
  const { key } = useParams();
  const navigate = useNavigate();
  const [product, setProduct] = useState<any>(null);
  const [superAttrs, setSuperAttrs] = useState<SuperAttr[]>([]);
  const [selected, setSelected] = useState<Record<string, number>>({});
  const [quantity, setQuantity] = useState(1);
  const [error, setError] = useState('');
  const [adding, setAdding] = useState(false);
  const [added, setAdded] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const data = await api.get(`products/${key}${shopQuery()}`);
        setProduct(data);
        setSuperAttrs((data.super_attributes ?? []).map((attr: any) => ({
          id: attr.id, code: attr.code, label: attr.label ?? attr.code,
          options: attr.options ?? [],
        })));
      } catch { setProduct(undefined); }
    })();
  }, [key]);

  if (product === undefined) {
    return <Container py="xl"><Alert icon={<IconAlertCircle size={16} />} color="red">Product not found.</Alert></Container>;
  }
  if (!product) {
    return <Center py="xl"><Loader /></Center>;
  }

  const isConfigurable = product.type === 'configurable';
  const allSelected = !isConfigurable || superAttrs.every((attr) => selected[attr.code] !== undefined);

  const addToCart = async () => {
    setAdding(true);
    setError('');
    try {
      const body: any = { product_id: product.id, quantity, ...selected };
      const data = await api.post('checkout/cart', body);
      if (data?.cart_token) localStorage.setItem('cart_token', data.cart_token);
      window.dispatchEvent(new Event('cart-updated'));
      setAdded(true);
      setTimeout(() => setAdded(false), 2500);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setAdding(false);
    }
  };

  return (
    <Container size="xl" py="md">
      <SimpleGrid cols={{ base: 1, md: 2 }} spacing="xl">
        <Stack>
          <Image src={product.base_image?.url ?? undefined} height={400} fit="contain" radius="md"
            fallbackSrc="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='400'%3E%3Crect fill='%23f1f3f5' width='400' height='400'/%3E%3C/svg%3E" />
          <SimpleGrid cols={4}>
            {(product.images ?? []).map((image: any, i: number) => (
              <Image key={i} src={image.url} height={90} fit="contain" radius="sm" />
            ))}
          </SimpleGrid>
        </Stack>
        <Stack gap="md">
          <Text size="xl" fw={600}>{product.name}</Text>
          <Group align="baseline" gap="sm">
            <Text size="xl" fw={600} c={product.on_sale ? 'red' : undefined}>{product.formatted_price}</Text>
            {product.on_sale && product.formatted_regular_price && (
              <Text size="md" c="dimmed" td="line-through">{product.formatted_regular_price}</Text>
            )}
          </Group>
          <Text c="dimmed" size="sm">{product.in_stock ? 'In stock' : 'Out of stock'}</Text>

          {isConfigurable && superAttrs.map((attr) => (
            <Select
              key={attr.code}
              label={attr.label}
              placeholder={`Select ${attr.label}`}
              data={attr.options.map((option) => ({ value: String(option.id), label: option.admin_name }))}
              value={selected[attr.code] !== undefined ? String(selected[attr.code]) : null}
              onChange={(value) => value && setSelected((prev) => ({ ...prev, [attr.code]: Number(value) }))}
            />
          ))}

          <Group>
            <NumberInput label="Quantity" min={1} value={quantity} onChange={(v) => setQuantity(Number(v) || 1)} w={110} />
            <Button
              mt="lg" size="md" disabled={isConfigurable && !allSelected} loading={adding} onClick={addToCart}>
              Add to cart
            </Button>
          </Group>
          {error && (
            <Alert icon={<IconAlertCircle size={16} />} color="red">{error}</Alert>
          )}
          {added && (
            <Alert color="green">Added to cart. <a href="/cart">View cart</a></Alert>
          )}

          {product.short_description && (
            <Text mt="md" size="sm" c="dimmed">{product.short_description}</Text>
          )}
        </Stack>
      </SimpleGrid>

      {product.description && (
        <Card mt="xl" withBorder>
          <Text fw={500} mb="sm">Description</Text>
          <div dangerouslySetInnerHTML={{ __html: product.description }} />
        </Card>
      )}

      {(product.additional_information ?? []).length > 0 && (
        <Card mt="md" withBorder>
          <Text fw={500} mb="sm">Additional information</Text>
          <Table>
            <Table.Tbody>
              {product.additional_information.map((row: any) => (
                <Table.Tr key={row.code}>
                  <Table.Td w={180} fw={500}>{row.label}</Table.Td>
                  <Table.Td>{row.value}</Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </Card>
      )}
    </Container>
  );
}
