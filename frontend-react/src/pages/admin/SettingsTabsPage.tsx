import { useEffect, useState } from 'react';
import { Tabs, Table, Button, TextInput, Stack, NumberInput, Switch, Loader, Group, Container, ActionIcon, Title } from '@mantine/core';
import { IconTrash } from '@tabler/icons-react';
import { api } from '../../api/client';

type Row = Record<string, any>;

function CrudSection({ path, columns, renderForm, transform }: {
  path: string; columns: string[]; renderForm: () => Row | null; transform?: (row: Row) => Row;
}) {
  const [rows, setRows] = useState<Row[]>([]);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    const data = await api.adminGet(path);
    setRows(Array.isArray(data) ? data : (data?.data ?? []));
    setLoading(false);
  };
  useEffect(() => { load(); }, [path]);

  const remove = async (id: number) => {
    try { await api.adminDelete(`${path}/${id}`); await load(); } catch { /* ignore */ }
  };

  if (loading) return <Loader />;
  return (
    <Table verticalSpacing="xs" withTableBorder>
      <Table.Thead bg="gray.0"><Table.Tr>{columns.map((column) => <Table.Th key={column}>{column}</Table.Th>)}<Table.Th w={60} /></Table.Tr></Table.Thead>
      <Table.Tbody>
        {rows.map((row) => (
          <Table.Tr key={row.id}>
            {columns.map((column) => <Table.Td key={column}>{String(row[column] ?? '')}</Table.Td>)}
            <Table.Td>
              <ActionIcon variant="subtle" color="red" onClick={() => remove(row.id)}><IconTrash size={16} /></ActionIcon>
            </Table.Td>
          </Table.Tr>
        ))}
        
      </Table.Tbody>
    </Table>
  );
}

export default function SettingsTabsPage() {
  const [saving, setSaving] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [channels, setChannels] = useState<Row[]>([]);
  const [locales, setLocales] = useState<Row[]>([]);
  const [currencies, setCurrencies] = useState<Row[]>([]);
  const [taxRates, setTaxRates] = useState<Row[]>([]);
  const [inventory, setInventory] = useState<Row[]>([]);
  const [newLocale, setNewLocale] = useState('');
  const [newLocaleName, setNewLocaleName] = useState('');
  const [newCurrency, setNewCurrency] = useState('');
  const [newCurrencyName, setNewCurrencyName] = useState('');
  const [newCurrencySymbol, setNewCurrencySymbol] = useState('');

  useEffect(() => {
    (async () => {
      const channelsData = await api.adminGet('admin/settings/channels');
      const localesData = await api.adminGet('admin/settings/locales');
      const currenciesData = await api.adminGet('admin/settings/currencies');
      const taxData = await api.adminPost('admin/settings/taxes/rates', {});
      const inventoryData = await api.adminGet('admin/settings/inventory-sources');
      setChannels(Array.isArray(channelsData) ? channelsData : decode(channelsData));
      setLocales(decode(localesData)); setCurrencies(decode(currenciesData));
      setTaxRates(decode(taxData)); setInventory(decode(inventoryData));
      setLoaded(true);
    })();
  }, []);

  const decode = (data: any): Row[] => Array.isArray(data) ? data : (data?.data ?? []);

  const saveLocale = async () => {
    if (!newLocale.trim()) return;
    setSaving(true);
    await api.adminPost('admin/settings/locales', { code: newLocale.trim(), name: newLocaleName.trim() || newLocale.trim() });
    const data = await api.adminGet('admin/settings/locales');
    setLocales(decode(data)); setNewLocale(''); setNewLocaleName(''); setSaving(false);
  };

  const saveCurrency = async () => {
    if (!newCurrency.trim()) return;
    setSaving(true);
    await api.adminPost('admin/settings/currencies', { code: newCurrency.trim(), name: newCurrencyName.trim() || newCurrency.trim(), symbol: newCurrencySymbol.trim() });
    const data = await api.adminGet('admin/settings/currencies');
    setCurrencies(decode(data)); setNewCurrency(''); setNewCurrencyName(''); setNewCurrencySymbol(''); setSaving(false);
  };

  if (!loaded) return <Container><Loader /></Container>;

  return (
    <Container size="lg" py="md">
      <Title order={2} mb="md">Settings</Title>
      <Tabs defaultValue="channels">
        <Tabs.List mb="md">
          <Tabs.Tab value="channels">Channels</Tabs.Tab>
          <Tabs.Tab value="locales">Locales</Tabs.Tab>
          <Tabs.Tab value="currencies">Currencies</Tabs.Tab>
          <Tabs.Tab value="taxes">Tax rates</Tabs.Tab>
          <Tabs.Tab value="inventory">Inventory sources</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="channels">
          <Table verticalSpacing="xs" withTableBorder>
            <Table.Thead bg="gray.0"><Table.Tr><Table.Th>ID</Table.Th><Table.Th>Name</Table.Th><Table.Th>Code</Table.Th></Table.Tr></Table.Thead>
            <Table.Tbody>{channels.map((channel) => (
              <Table.Tr key={channel.id}><Table.Td>{channel.id}</Table.Td><Table.Td>{channel.name ?? channel.code}</Table.Td><Table.Td>{channel.code}</Table.Td></Table.Tr>
            ))}</Table.Tbody>
          </Table>
        </Tabs.Panel>

        <Tabs.Panel value="locales">
          <Group align="flex-end" mb="md">
            <TextInput label="Code" value={newLocale} onChange={(e) => setNewLocale(e.currentTarget.value)} />
            <TextInput label="Name" value={newLocaleName} onChange={(e) => setNewLocaleName(e.currentTarget.value)} />
            <Button onClick={saveLocale} loading={saving}>Add locale</Button>
          </Group>
          <Table verticalSpacing="xs" withTableBorder>
            <Table.Thead bg="gray.0"><Table.Tr><Table.Th>Code</Table.Th><Table.Th>Name</Table.Th><Table.Th>Direction</Table.Th></Table.Tr></Table.Thead>
            <Table.Tbody>{locales.map((locale) => (
              <Table.Tr key={locale.id}><Table.Td>{locale.code}</Table.Td><Table.Td>{locale.name}</Table.Td><Table.Td>{locale.direction}</Table.Td></Table.Tr>
            ))}</Table.Tbody>
          </Table>
        </Tabs.Panel>

        <Tabs.Panel value="currencies">
          <Group align="flex-end" mb="md">
            <TextInput label="Code" value={newCurrency} onChange={(e) => setNewCurrency(e.currentTarget.value)} />
            <TextInput label="Name" value={newCurrencyName} onChange={(e) => setNewCurrencyName(e.currentTarget.value)} />
            <TextInput label="Symbol" value={newCurrencySymbol} onChange={(e) => setNewCurrencySymbol(e.currentTarget.value)} />
            <Button onClick={saveCurrency} loading={saving}>Add currency</Button>
          </Group>
          <Table verticalSpacing="xs" withTableBorder>
            <Table.Thead bg="gray.0"><Table.Tr><Table.Th>Code</Table.Th><Table.Th>Name</Table.Th><Table.Th>Symbol</Table.Th></Table.Tr></Table.Thead>
            <Table.Tbody>{currencies.map((currency) => (
              <Table.Tr key={currency.id}><Table.Td>{currency.code}</Table.Td><Table.Td>{currency.name}</Table.Td><Table.Td>{currency.symbol}</Table.Td></Table.Tr>
            ))}</Table.Tbody>
          </Table>
        </Tabs.Panel>

        <Tabs.Panel value="taxes">
          <Table verticalSpacing="xs" withTableBorder>
            <Table.Thead bg="gray.0"><Table.Tr><Table.Th>Identifier</Table.Th><Table.Th>Country</Table.Th><Table.Th>Rate %</Table.Th></Table.Tr></Table.Thead>
            <Table.Tbody>{taxRates.map((rate) => (
              <Table.Tr key={rate.id}><Table.Td>{rate.identifier}</Table.Td><Table.Td>{rate.country}</Table.Td><Table.Td>{rate.tax_rate}</Table.Td></Table.Tr>
            ))}</Table.Tbody>
          </Table>
        </Tabs.Panel>

        <Tabs.Panel value="inventory">
          <Table verticalSpacing="xs" withTableBorder>
            <Table.Thead bg="gray.0"><Table.Tr><Table.Th>Code</Table.Th><Table.Th>Name</Table.Th><Table.Th>Status</Table.Th></Table.Tr></Table.Thead>
            <Table.Tbody>{inventory.map((source) => (
              <Table.Tr key={source.id}><Table.Td>{source.code}</Table.Td><Table.Td>{source.name}</Table.Td><Table.Td>{source.status === 1 ? 'Active' : 'Inactive'}</Table.Td></Table.Tr>
            ))}</Table.Tbody>
          </Table>
        </Tabs.Panel>
      </Tabs>
    </Container>
  );
}
