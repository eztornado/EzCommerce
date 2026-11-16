import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { Container, SimpleGrid, Card, Image, Text, Loader, Center, RangeSlider, Group, Button, Select } from '@mantine/core';
import { Link } from 'react-router-dom';
import { api } from '../../api/client';
import { shopQuery } from '../../store';

export default function CategoryPage() {
  const { id } = useParams();
  const [name, setName] = useState('');
  const [products, setProducts] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [sort, setSort] = useState('newest');

  useEffect(() => {
    (async () => {
      try {
        const data = await api.get(`categories/${id}${shopQuery()}`);
        setName(data.name ?? data.slug ?? '');
      } catch { /* ignore */ }
    })();
  }, [id]);

  useEffect(() => {
    (async () => {
      setLoading(true);
      try {
        const data = await api.get(`products${shopQuery({ category_id: id ?? '', sort, per_page: '20' })}`);
        setProducts(data?.data ?? []);
      } catch { /* ignore */ }
      setLoading(false);
    })();
  }, [id, sort]);

  return (
    <Container size="xl" py="md">
      <Text size="xl" fw={600} mb="md">{name}</Text>
      <Group justify="flex-end" mb="md">
        <Select size="sm" w={180} value={sort} onChange={(v) => setSort(v ?? 'newest')}
          data={[
            { value: 'newest', label: 'Newest first' },
            { value: 'price_asc', label: 'Price: low to high' },
            { value: 'price_desc', label: 'Price: high to low' },
            { value: 'name_asc', label: 'Name A-Z' },
          ]} />
      </Group>
      {loading ? (
        <Center py="xl"><Loader /></Center>
      ) : products.length === 0 ? (
        <Center py="xl"><Text c="dimmed">No products found.</Text></Center>
      ) : (
        <SimpleGrid cols={{ base: 2, sm: 3, lg: 4 }} spacing="md">
          {products.map((product) => (
            <Link key={product.id} to={`/product/${product.url_key ?? product.id}`} style={{ color: 'inherit', textDecoration: 'none' }}>
              <Card shadow="sm" radius="md" withBorder>
                <Card.Section>
                  <Image src={product.base_image?.url ?? undefined} height={180} fit="contain"
                    fallbackSrc="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='180'%3E%3Crect fill='%23f1f3f5' width='200' height='180'/%3E%3C/svg%3E" />
                </Card.Section>
                <Text fw={500} size="sm" mt="sm" lineClamp={2}>{product.name}</Text>
                <Text fw={600} size="sm" mt="xs">{product.formatted_price}</Text>
              </Card>
            </Link>
          ))}
        </SimpleGrid>
      )}
    </Container>
  );
}
