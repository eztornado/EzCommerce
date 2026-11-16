import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { Card, Image, Text, SimpleGrid, Container, Stack } from '@mantine/core';
import { api } from '../../api/client';
import { shopQuery } from '../../store';

interface Section { type: string; name: string; options: any }

function MediaUrl(path: string): string {
  return path?.startsWith('http') ? path : `/storage/${path}`;
}

export default function HomePage() {
  const [sections, setSections] = useState<Section[]>([]);
  useEffect(() => {
    (async () => {
      try {
        const data = await api.get(`core/home${shopQuery()}`);
        setSections(Array.isArray(data) ? data : []);
      } catch { /* ignore */ }
    })();
  }, []);

  return (
    <Container size="xl" py="md">
      <Stack gap="xl">
        {sections.map((section, index) => {
          const options = section.options ?? {};
          if (section.type === 'image_carousel' && options.slides) {
            return (
              <Card key={index} shadow="sm" radius="md" withBorder>
                <SimpleGrid cols={{ base: 1, sm: 3 }} spacing="md">
                  {options.slides.map((slide: any, i: number) => (
                    <Link key={i} to={slide.link ?? '/'}>
                      <Image src={slide.image ? MediaUrl(slide.image) : undefined} height={160} fit="cover" radius="md"
                        fallbackSrc="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='160'%3E%3Crect fill='%23f1f3f5' width='400' height='160'/%3E%3C/svg%3E" />
                      <Text ta="center" fw={500} mt="xs">{slide.title}</Text>
                    </Link>
                  ))}
                </SimpleGrid>
              </Card>
            );
          }
          if (section.type === 'product_carousel' || section.type === 'category_carousel') {
            return <ProductGridSection key={index} title={options.title ?? section.name}
              path={section.type === 'product_carousel' ? 'products' : 'categories'} limit={options.limit ?? 8} />;
          }
          if (section.type === 'static_content') {
            return (
              <Card key={index} shadow="sm" radius="md" withBorder p="xl">
                <Text size="xl" fw={600}>{options.title}</Text>
                <Text c="dimmed" mt="xs">{options.subtitle}</Text>
                {options.button && (
                  <Link to={options.button.link ?? '/'} style={{ color: '#228be6', marginTop: 8, display: 'inline-block' }}>
                    {options.button.title ?? 'Shop now'}
                  </Link>
                )}
              </Card>
            );
          }
          return null;
        })}
      </Stack>
    </Container>
  );
}

function ProductGridSection({ title, path, limit }: { title: string; path: string; limit: number }) {
  const [items, setItems] = useState<any[]>([]);
  useEffect(() => {
    (async () => {
      try {
        const data = await api.get(`${path}${shopQuery({ per_page: String(limit) })}`);
        setItems(Array.isArray(data) ? data : (data?.data ?? []));
      } catch { /* ignore */ }
    })();
  }, []);
  if (!items.length) return null;
  return (
    <Card shadow="sm" radius="md" withBorder>
      <Text fw={500} mb="md">{title}</Text>
      <SimpleGrid cols={{ base: 2, sm: 4 }} spacing="md">
        {items.map((item) => (
          <Link key={item.id ?? item.slug} to={path === 'products' ? `/product/${item.url_key ?? item.id}` : `/category/${item.id}`}
            style={{ color: 'inherit', textDecoration: 'none' }}>
            <Card shadow="xs" radius="md" withBorder p="xs">
              <Image src={item.base_image?.url ?? undefined} height={140} fit="contain"
                fallbackSrc="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='140'%3E%3Crect fill='%23f1f3f5' width='200' height='140'/%3E%3C/svg%3E" />
              <Text size="sm" mt="xs" lineClamp={2}>{item.name ?? item.slug}</Text>
              <Text size="sm" fw={600}>{item.formatted_price ?? ''}</Text>
            </Card>
          </Link>
        ))}
      </SimpleGrid>
    </Card>
  );
}
