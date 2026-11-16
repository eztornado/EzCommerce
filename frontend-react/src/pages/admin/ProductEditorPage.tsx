import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Tabs, TextInput, Textarea, Select, NumberInput, Button, Stack, Card, Table, Group, Loader, Checkbox, ActionIcon, Switch, Image, Text, Paper } from '@mantine/core';
import { IconTrash, IconPlus } from '@tabler/icons-react';
import { api } from '../../api/client';

const TABS = ['Settings', 'Description', 'Images', 'Inventories', 'Variations', 'Linked products', 'SEO'];

export default function ProductEditorPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [product, setProduct] = useState<any>(null);
  const [attributes, setAttributes] = useState<any>({});
  const [categories, setCategories] = useState<any[]>([]);
  const [inventories, setInventories] = useState<any[]>([]);
  const [variants, setVariants] = useState<any[]>([]);
  const [images, setImages] = useState<any[]>([]);
  const [links, setLinks] = useState<any>({ related: [], up_sells: [], cross_sells: [] });
  const [locales, setLocales] = useState<any[]>([]);
  const [locale, setLocale] = useState('en');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const [name, setName] = useState('');
  const [price, setPrice] = useState('');
  const [specialPrice, setSpecialPrice] = useState('');
  const [specialFrom, setSpecialFrom] = useState('');
  const [specialTo, setSpecialTo] = useState('');
  const [weight, setWeight] = useState('0');
  const [shortDescription, setShortDescription] = useState('');
  const [description, setDescription] = useState('');
  const [urlKey, setUrlKey] = useState('');
  const [metaTitle, setMetaTitle] = useState('');
  const [metaDescription, setMetaDescription] = useState('');
  const [selectedCategories, setSelectedCategories] = useState<number[]>([]);
  const [quantities, setQuantities] = useState<Record<string, number>>({});
  const [newVariantSku, setNewVariantSku] = useState('');
  const [newVariantColor, setNewVariantColor] = useState<string | null>(null);
  const [newVariantQty, setNewVariantQty] = useState('10');

  useEffect(() => {
    (async () => {
      try {
        const contextData = await api.adminGet('core/context');
        if (contextData?.locales) setLocales(contextData.locales);
        const data = await api.adminGet(`admin/catalog/products/edit/${id}?locale=${locale}`);
        const tree = await api.adminGet('admin/catalog/categories/tree');
        setProduct(data.product);
        setAttributes(data.attributes ?? {});
        setName(data.attributes?.name ?? '');
        setPrice(String(data.attributes?.price ?? '0'));
        setSpecialPrice(String(data.attributes?.special_price ?? ''));
        setSpecialFrom(String(data.attributes?.special_price_from ?? ''));
        setSpecialTo(String(data.attributes?.special_price_to ?? ''));
        setWeight(String(data.attributes?.weight ?? '0'));
        setShortDescription(data.attributes?.short_description ?? '');
        setDescription(data.attributes?.description ?? '');
        setUrlKey(data.attributes?.url_key ?? '');
        setMetaTitle(data.attributes?.meta_title ?? '');
        setMetaDescription(data.attributes?.meta_description ?? '');
        setInventories(data.inventories ?? []);
        setImages(data.images ?? []);
        setVariants(data.variants ?? []);
        setLinks(data.links ?? { related: [], up_sells: [], cross_sells: [] });
        setSelectedCategories((data.categories ?? []).map((row: any) => row.category_id ?? row.id));
        const treeData = Array.isArray(tree) ? tree : tree?.data ?? [];
        setCategories(treeData.flatMap((root: any) => root.children ?? [root]));
      } catch { /* ignore */ }
      setLoading(false);
    })();
  }, [id, locale]);

  const save = async () => {
    setSaving(true);
    try {
      await api.adminPut(`admin/catalog/products/edit/${id}`, {
        locale,
        sku: product?.sku,
        attributes: {
          name, price: Number(price) || 0, weight: Number(weight) || 0,
          special_price: specialPrice ? Number(specialPrice) : null,
          special_price_from: specialFrom || null, special_price_to: specialTo || null,
          short_description: shortDescription, description,
          url_key: urlKey, meta_title: metaTitle, meta_description: metaDescription,
          status: 1, visible_individually: 1,
        },
        categories: selectedCategories,
        inventories: quantities,
        links,
      });
      navigate('/admin/products');
    } finally { setSaving(false); }
  };

  const copyProduct = async () => {
    const result = await api.adminPost(`admin/catalog/products/copy/${id}`);
    if (result?.id) navigate(`/admin/products/edit/${result.id}`);
  };

  if (loading) return <Center><Loader /></Center>;

  return (
    <Stack p="md" gap="md">
      <Group justify="space-between">
        <Group gap="sm">
          <Button variant="subtle" color="gray" onClick={() => navigate('/admin/products')}>← Products</Button>
          <Text size="lg" fw={600}>Edit product #{id}</Text>
        </Group>
        <Group>
          {locales.length > 1 && (
            <Select size="sm" w={150} value={locale} data={locales.map((l) => ({ value: l.code, label: l.name }))}
              onChange={(v) => { if (v && v !== locale) { setLocale(v); setLoading(true); setTimeout(() => window.location.reload(), 0); } }} />
          )}
          <Button variant="light" onClick={copyProduct}>Copy product</Button>
          <Button loading={saving} onClick={save}>Save</Button>
        </Group>
      </Group>

      <Card withBorder p="md">
        <Tabs defaultValue="settings" keepMounted={false}>
          <Tabs.List mb="md">
            {TABS.map((tab) => <Tabs.Tab key={tab} value={tab}>{tab}</Tabs.Tab>)}
          </Tabs.List>

          <Tabs.Panel value="Settings">
            <Stack gap="sm" maw={720}>
              <TextInput label="SKU" value={product?.sku ?? ''} readOnly />
              <Group grow>
                <TextInput label="Name" value={name} onChange={(e) => setName(e.currentTarget.value)} />
                <TextInput label="URL key" value={urlKey} onChange={(e) => setUrlKey(e.currentTarget.value)} />
              </Group>
              <Group grow>
                <TextInput label="Price (EUR)" value={price} onChange={(e) => setPrice(e.currentTarget.value)} />
                <TextInput label="Weight" value={weight} onChange={(e) => setWeight(e.currentTarget.value)} />
              </Group>
              <Group grow>
                <TextInput label="Special price" value={specialPrice} onChange={(e) => setSpecialPrice(e.currentTarget.value)} />
                <TextInput label="Special from (YYYY-MM-DD)" value={specialFrom} onChange={(e) => setSpecialFrom(e.currentTarget.value)} />
                <TextInput label="Special to (YYYY-MM-DD)" value={specialTo} onChange={(e) => setSpecialTo(e.currentTarget.value)} />
              </Group>
              <Text fw={500} size="sm" mt="sm">Categories</Text>
              <Group gap="sm">
                {categories.map((category) => (
                  <Checkbox key={category.id} label={category.name ?? category.slug}
                    checked={selectedCategories.includes(category.id)}
                    onChange={(e) => setSelectedCategories(e.currentTarget.checked
                      ? [...selectedCategories, category.id]
                      : selectedCategories.filter((existing) => existing !== category.id))} />
                ))}
              </Group>
            </Stack>
          </Tabs.Panel>

          <Tabs.Panel value="Description">
            <Stack gap="sm" maw={720}>
              <TextInput label="Name" value={name} onChange={(e) => setName(e.currentTarget.value)} />
              <Textarea label="Short description" value={shortDescription} onChange={(e) => setShortDescription(e.currentTarget.value)} />
              <Textarea label="Description" minRows={8} value={description} onChange={(e) => setDescription(e.currentTarget.value)} />
            </Stack>
          </Tabs.Panel>

          <Tabs.Panel value="Images">
            <Group gap="sm">
              {images.map((image) => (
                <Image key={image.id} src={`/storage/${image.path}`} w={90} h={90} fit="contain" radius="sm" />
              ))}
            </Group>
            <Text size="sm" c="dimmed" mt="sm">Use the Flutter panel or the API to upload new images in this phase.</Text>
          </Tabs.Panel>

          <Tabs.Panel value="Inventories">
            <Stack gap="sm" maw={620}>
              {inventories.map((inventory) => (
                <Group key={inventory.inventory_source_id ?? inventory.id} justify="space-between">
                  <Text size="sm">{inventory.name ?? `Source ${inventory.inventory_source_id}`}</Text>
                  <NumberInput w={140} value={inventory.qty ?? 0}
                    onChange={(v) => setQuantities({ ...quantities, [inventory.inventory_source_id]: Number(v) || 0 })} />
                </Group>
              ))}
              <Button variant="light" onClick={async () => {
                await api.adminPut(`admin/catalog/products/edit/${id}`, { inventories: quantities });
                window.location.reload();
              }}>Update inventories</Button>
            </Stack>
          </Tabs.Panel>

          <Tabs.Panel value="Variations">
            <Stack gap="sm">
              <Table verticalSpacing="xs" withTableBorder>
                <Table.Thead><Table.Tr><Table.Th>Variant</Table.Th><Table.Th>Qty</Table.Th><Table.Th /></Table.Tr></Table.Thead>
                <Table.Tbody>
                  {variants.map((variant) => (
                    <Table.Tr key={variant.id}>
                      <Table.Td>{variant.sku ?? variant.attributes?.sku}</Table.Td>
                      <Table.Td>{variant.qty}</Table.Td>
                      <Table.Td>
                        <ActionIcon variant="subtle" color="red" onClick={async () => {
                          await api.adminDelete(`admin/catalog/products/variants/${variant.id}`);
                          setVariants(variants.filter((v) => v.id !== variant.id));
                        }}><IconTrash size={16} /></ActionIcon>
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
              <Group align="flex-end" gap="sm">
                <TextInput label="New variant SKU" value={newVariantSku} onChange={(e) => setNewVariantSku(e.currentTarget.value)} />
                <TextInput label="Color option id" value={newVariantColor ?? ''} onChange={(e) => setNewVariantColor(e.currentTarget.value)} />
                <NumberInput label="Qty" value={newVariantQty} onChange={(v) => setNewVariantQty(String(v))} w={110} />
                <Button mt={4} variant="light" onClick={async () => {
                  if (!newVariantSku.trim() || !newVariantColor) return;
                  await api.adminPost(`admin/catalog/products/edit/${id}/variants`, {
                    sku: newVariantSku.trim(),
                    attributes: { name: `${name} - ${newVariantSku}`, price: Number(price) || 0, color: Number(newVariantColor) },
                    qty: Number(newVariantQty) || 0,
                  });
                  window.location.reload();
                }}>Create variant</Button>
              </Group>
            </Stack>
          </Tabs.Panel>

          <Tabs.Panel value="Linked products">
            <Paper withBorder p="sm">
              <Text size="sm" c="dimmed">Linked products (related / up-sells / cross-sells) are preserved on save.</Text>
              <Text size="sm" mt="xs">related: {JSON.stringify(links.related ?? [])}</Text>
              <Text size="sm">up_sells: {JSON.stringify(links.up_sells ?? [])}</Text>
              <Text size="sm">cross_sells: {JSON.stringify(links.cross_sells ?? [])}</Text>
            </Paper>
          </Tabs.Panel>

          <Tabs.Panel value="SEO">
            <Stack gap="sm" maw={720}>
              <TextInput label="Meta title" value={metaTitle} onChange={(e) => setMetaTitle(e.currentTarget.value)} />
              <Textarea label="Meta description" value={metaDescription} onChange={(e) => setMetaDescription(e.currentTarget.value)} />
            </Stack>
          </Tabs.Panel>
        </Tabs>
      </Card>
    </Stack>
  );
}

function Center({ children }: { children: React.ReactNode }) {
  return <div style={{ display: 'flex', justifyContent: 'center', padding: 40 }}>{children}</div>;
}
