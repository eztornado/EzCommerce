import { Outlet, NavLink, Navigate } from 'react-router-dom';
import { AppShell, Group, Text, UnstyledButton, useMantineColorScheme } from '@mantine/core';
import { useAdminAuth } from '../../store';

const NAV = [
  { label: 'Dashboard', path: '/admin' },
  { label: 'Products', path: '/admin/products' },
];

export default function AdminShell() {
  const loggedIn = useAdminAuth((s) => s.loggedIn);
  if (!loggedIn) return <Navigate to="/admin/login" replace />;

  return (
    <AppShell navbar={{ width: 240, breakpoint: 'sm' }}>
      <AppShell.Navbar p="md">
        <Text fw={700} size="lg" mb="xl">EzCommerce Admin</Text>
        {NAV.map((item) => (
          <NavLink key={item.path} to={item.path} style={{ borderRadius: 8, marginBottom: 4 }}>
            {item.label}
          </NavLink>
        ))}
        <UnstyledButton mt="auto" onClick={async () => { await useAdminAuth.getState().logout(); location.href = '/admin/login'; }}>
          <Text size="sm">Sign out</Text>
        </UnstyledButton>
      </AppShell.Navbar>
      <AppShell.Main><Outlet /></AppShell.Main>
    </AppShell>
  );
}
