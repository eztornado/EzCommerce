import { Outlet, Link, useNavigate } from 'react-router-dom';
import { Menu, Group, Container, Select, UnstyledButton, Badge } from '@mantine/core';
import { IconShoppingCart, IconUser } from '@tabler/icons-react';
import { useShop, useCart, useAuth } from '../../store';
import { useEffect, useState } from 'react';
import { api } from '../../api/client';

export default function ShopLayout() {
  const { context, changeLocale, changeCurrency } = useShop();
  const refresh = useCart((s) => s.refresh);
  const loggedIn = useAuth((s) => s.loggedIn);
  const [count, setCount] = useState(0);
  const navigate = useNavigate();

  const loadCount = async () => {
    try {
      const data = await api.get(`checkout/cart${shopCartQuery()}`);
      const cart = data?.cart ?? data;
      setCount(cart?.items_qty ?? 0);
    } catch { /* ignore */ }
  };

  useEffect(() => { refresh?.(); loadCount(); }, [refresh]);
  useEffect(() => {
    const handler = () => loadCount();
    window.addEventListener('cart-updated', handler);
    return () => window.removeEventListener('cart-updated', handler);
  });

  return (
    <>
      <div style={{ borderBottom: '1px solid #e9ecef', fontSize: 12, padding: '6px 0' }}>
        <Container size="xl" style={{ display: 'flex', justifyContent: 'space-between' }}>
          <Select
            size="xs" variant="unstyled" w={140}
            data={context?.currencies.map((c) => ({ value: c.code, label: `${c.code} - ${c.name}` })) ?? []}
            value={context?.currency ?? ''}
            onChange={(v) => v && changeCurrency(v)}
            placeholder="Currency"
          />
          <Select
            size="xs" variant="unstyled" w={120}
            data={context?.locales.map((l) => ({ value: l.code, label: l.name })) ?? []}
            value={context?.locale ?? ''}
            onChange={(v) => v && changeLocale(v)}
            placeholder="Language"
          />
        </Container>
      </div>
      <Container size="xl" py="md">
        <Group justify="space-between">
          <UnstyledButton onClick={() => navigate('/')}>
            <strong style={{ fontSize: 22 }}>EzCommerce</strong>
          </UnstyledButton>
          <Group gap="md">
            <Link to="/cart" style={{ color: 'inherit' }}>
              <Group gap={4}>
                <IconShoppingCart size={20} />
                <Badge size="sm" circle>{count}</Badge>
              </Group>
            </Link>
            <Menu shadow="md" width={160}>
              <Menu.Target>
                <UnstyledButton><IconUser size={20} /></UnstyledButton>
              </Menu.Target>
              <Menu.Dropdown>
                {loggedIn ? (
                  <>
                    <Menu.Item component="a" href="/account">My account</Menu.Item>
                    <Menu.Item onClick={async () => { await useAuth.getState().logout(); navigate('/'); }}>Sign out</Menu.Item>
                  </>
                ) : (
                  <>
                    <Menu.Item component="a" href="/login">Sign in</Menu.Item>
                    <Menu.Item component="a" href="/login">Create account</Menu.Item>
                  </>
                )}
              </Menu.Dropdown>
            </Menu>
          </Group>
        </Group>
      </Container>
      <Outlet />
    </>
  );
}

function shopCartQuery(): string {
  const params = new URLSearchParams();
  const locale = localStorage.getItem('shop_locale');
  const currency = localStorage.getItem('shop_currency');
  if (locale) params.set('locale', locale);
  if (currency) params.set('currency', currency);
  const q = params.toString();
  return q ? `?${q}` : '';
}
