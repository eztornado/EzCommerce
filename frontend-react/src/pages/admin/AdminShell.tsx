import { useState } from 'react';
import { Outlet, NavLink, Navigate, useLocation } from 'react-router-dom';
import { AppShell, Group, Text, UnstyledButton, Stack, ThemeIcon, Box, Button } from '@mantine/core';
import {
  IconLayoutDashboard, IconPackage, IconReceipt, IconUsers,
  IconTag, IconSettings, IconLogout, IconChevronRight,
  IconChartBar, IconArrowsTransferDown, IconSettingsAutomation,
} from '@tabler/icons-react';
import { useAdminAuth } from '../../store';

const NAV: { label: string; icon: any; items?: { label: string; path: string }[]; path?: string }[] = [
  { label: 'Dashboard', icon: IconLayoutDashboard, path: '/admin' },
  {
    label: 'Sales', icon: IconReceipt,
    items: [
      { label: 'Orders', path: '/admin/orders' },
      { label: 'Invoices', path: '/admin/invoices' },
      { label: 'Shipments', path: '/admin/shipments' },
      { label: 'Refunds', path: '/admin/refunds' },
    ],
  },
  {
    label: 'Catalog', icon: IconPackage,
    items: [
      { label: 'Products', path: '/admin/products' },
      { label: 'Categories', path: '/admin/categories' },
      { label: 'Attributes', path: '/admin/attributes' },
      { label: 'Families', path: '/admin/families' },
    ],
  },
  {
    label: 'Customers', icon: IconUsers,
    items: [
      { label: 'Customers', path: '/admin/customers' },
      { label: 'Groups', path: '/admin/customers/groups' },
      { label: 'Reviews', path: '/admin/customers/reviews' },
    ],
  },
  {
    label: 'Marketing', icon: IconTag,
    items: [
      { label: 'Cart rules', path: '/admin/cart-rules' },
      { label: 'Catalog rules', path: '/admin/catalog-rules' },
      { label: 'Search SEO', path: '/admin/search-seo' },
    ],
  },
  {
    label: 'Settings', icon: IconSettings,
    items: [
      { label: 'Channels & taxes', path: '/admin/settings' },
      { label: 'Inventory sources', path: '/admin/inventory-sources' },
    ],
  },
  {
    label: 'Configuration', icon: IconSettingsAutomation,
    items: [
      { label: 'General & shipping', path: '/admin/configuration' },
      { label: 'Data transfer', path: '/admin/data-transfer' },
    ],
  },
  {
    label: 'Reporting', icon: IconChartBar,
    items: [{ label: 'Reports', path: '/admin/reporting' }],
  },
];

function NavItem({ label, path, active }: { label: string; path: string; active: boolean }) {
  return (
    <NavLink
      to={path}
      style={{
        display: 'block',
        padding: '7px 12px 7px 34px',
        borderRadius: 6,
        fontSize: 13,
        color: active ? '#111827' : '#4b4f56',
        background: active ? '#e8eef7' : 'transparent',
        fontWeight: active ? 600 : 400,
      }}
    >
      {label}
    </NavLink>
  );
}

export default function AdminShell() {
  const loggedIn = useAdminAuth((s) => s.loggedIn);
  const location = useLocation();
  const [openGroup, setOpenGroup] = useState<string | null>(
    NAV.find((group) => group.items?.some((item) => item.path === location.pathname))?.label ?? null,
  );

  if (!loggedIn) return <Navigate to="/admin/login" replace />;

  return (
    <AppShell navbar={{ width: 232, breakpoint: 'sm' }} header={{ height: 52 }}>
      <AppShell.Header bg="white" style={{ borderBottom: '1px solid #e5e7eb' }}>
        <Group h="100%" px="md" justify="space-between">
          <Text fw={700} size="lg">EzCommerce</Text>
          <Button
            variant="subtle" color="gray" size="compact-sm"
            leftSection={<IconLogout size={16} />}
            onClick={async () => {
              await useAdminAuth.getState().logout();
              location instanceof Location && (window.location.href = '/admin/login');
            }}
          >
            Sign out
          </Button>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar bg="white" style={{ borderRight: '1px solid #e5e7eb' }} p="xs">
        <Stack gap={2}>
          {NAV.map((group) => {
            const Icon = group.icon;
            const isCurrent = group.items?.some((item) => item.path === location.pathname) ?? false;
            if (!group.items) {
              const active = location.pathname === group.path;
              return (
                <UnstyledButton
                  key={group.label}
                  component="a"
                  href={group.path}
                  style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '8px 10px', borderRadius: 6,
                           background: active ? '#e8eef7' : 'transparent' }}
                >
                  <ThemeIcon variant="subtle" color="gray" size={20}><Icon size={16} /></ThemeIcon>
                  <Text size="sm" fw={500}>{group.label}</Text>
                </UnstyledButton>
              );
            }
            const open = openGroup === group.label || isCurrent;
            return (
              <Box key={group.label}>
                <UnstyledButton
                  onClick={() => setOpenGroup(openGroup === group.label ? null : group.label)}
                  style={{ display: 'flex', alignItems: 'center', gap: 10, width: '100%', padding: '8px 10px', borderRadius: 6 }}
                >
                  <ThemeIcon variant="subtle" color="gray" size={20}><Icon size={16} /></ThemeIcon>
                  <Text size="sm" fw={500} style={{ flex: 1, textAlign: 'left' }}>{group.label}</Text>
                  <IconChevronRight size={14} style={{ transform: open ? 'rotate(90deg)' : 'none' }} />
                </UnstyledButton>
                {open && (
                  <Stack gap={0} mt={2}>
                    {group.items.map((item) => (
                      <NavItem key={item.path} label={item.label} path={item.path}
                        active={location.pathname === item.path} />
                    ))}
                  </Stack>
                )}
              </Box>
            );
          })}
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main bg="#f5f6f8">
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
}
