import { useEffect, useMemo, useState } from 'react';
import { Link, NavLink, Navigate, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { Menu } from '@mantine/core';
import {
  IconBell,
  IconBox,
  IconBuildingStore,
  IconCalendarEvent,
  IconCategory,
  IconChartBar,
  IconChevronDown,
  IconChevronRight,
  IconCurrencyDollar,
  IconDatabaseExport,
  IconDiscount2,
  IconExternalLink,
  IconFileInvoice,
  IconFileText,
  IconLayoutDashboard,
  IconLink,
  IconListDetails,
  IconLogout,
  IconMail,
  IconMenu2,
  IconPackage,
  IconPalette,
  IconReceipt,
  IconReceiptRefund,
  IconReceiptTax,
  IconSearch,
  IconSend,
  IconSettings,
  IconShield,
  IconShoppingCart,
  IconSitemap,
  IconStar,
  IconTag,
  IconTruckDelivery,
  IconUser,
  IconUserPlus,
  IconUsers,
  IconUsersGroup,
  IconWorld,
} from '@tabler/icons-react';
import { useAdminAuth } from '../../store';
import { initials } from '../../lib/format';

interface MenuChild {
  label: string;
  to: string;
}

interface MenuItem {
  label: string;
  to?: string;
  icon: React.ReactNode;
  children?: MenuChild[];
}

const ACRONYMS: Record<string, string> = {
  rma: 'RMA',
  seo: 'SEO',
  cms: 'CMS',
  gdpr: 'GDPR',
  url: 'URL',
  api: 'API',
};

const SEGMENT_LABELS: Record<string, string> = {
  templates: 'Email Templates',
  subscribers: 'Newsletter Subscribers',
};

function breadcrumbLabel(segment: string): string {
  if (SEGMENT_LABELS[segment]) return SEGMENT_LABELS[segment];
  return segment
    .split('-')
    .map((word) => ACRONYMS[word.toLowerCase()] ?? word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

const MENU: MenuItem[] = [
  { label: 'Dashboard', to: '/admin', icon: <IconLayoutDashboard size={17} /> },
  {
    label: 'Sales',
    icon: <IconShoppingCart size={17} />,
    children: [
      { label: 'Orders', to: '/admin/sales/orders' },
      { label: 'Invoices', to: '/admin/sales/invoices' },
      { label: 'Shipments', to: '/admin/sales/shipments' },
      { label: 'Refunds', to: '/admin/sales/refunds' },
    ],
  },
  {
    label: 'Catalog',
    icon: <IconBox size={17} />,
    children: [
      { label: 'Products', to: '/admin/catalog/products' },
      { label: 'Categories', to: '/admin/catalog/categories' },
      { label: 'Attributes', to: '/admin/catalog/attributes' },
      { label: 'Attribute Families', to: '/admin/catalog/families' },
    ],
  },
  {
    label: 'Customers',
    icon: <IconUsers size={17} />,
    children: [
      { label: 'Customers', to: '/admin/customers' },
      { label: 'Customer Groups', to: '/admin/customers/groups' },
      { label: 'Reviews', to: '/admin/customers/reviews' },
      { label: 'RMA', to: '/admin/customers/rma' },
      { label: 'GDPR Requests', to: '/admin/customers/gdpr' },
    ],
  },
  {
    label: 'Marketing',
    icon: <IconDiscount2 size={17} />,
    children: [
      { label: 'Cart Rules', to: '/admin/marketing/promotions/cart-rules' },
      { label: 'Catalog Rules', to: '/admin/marketing/promotions/catalog-rules' },
      { label: 'Email Templates', to: '/admin/marketing/communications/templates' },
      { label: 'Events', to: '/admin/marketing/communications/events' },
      { label: 'Campaigns', to: '/admin/marketing/communications/campaigns' },
      { label: 'URL Rewrites', to: '/admin/marketing/search-seo/url-rewrites' },
      { label: 'Search Terms', to: '/admin/marketing/search-seo/search-terms' },
      { label: 'Search Synonyms', to: '/admin/marketing/search-seo/search-synonyms' },
      { label: 'Newsletter Subscribers', to: '/admin/marketing/communications/subscribers' },
    ],
  },
  {
    label: 'CMS',
    icon: <IconFileText size={17} />,
    children: [{ label: 'Pages', to: '/admin/cms/pages' }],
  },
  { label: 'Reporting', to: '/admin/reporting/sales', icon: <IconChartBar size={17} /> },
  {
    label: 'Settings',
    icon: <IconSettings size={17} />,
    children: [
      { label: 'Locales', to: '/admin/settings/locales' },
      { label: 'Currencies', to: '/admin/settings/currencies' },
      { label: 'Exchange Rates', to: '/admin/settings/exchange-rates' },
      { label: 'Channels', to: '/admin/settings/channels' },
      { label: 'Users', to: '/admin/settings/users' },
      { label: 'Roles', to: '/admin/settings/roles' },
      { label: 'Tax Categories', to: '/admin/settings/taxes/categories' },
      { label: 'Tax Rates', to: '/admin/settings/taxes/rates' },
      { label: 'Inventory Sources', to: '/admin/settings/inventory-sources' },
    ],
  },
  { label: 'Configuration', to: '/admin/configuration', icon: <IconShield size={17} /> },
  { label: 'Data Transfer', to: '/admin/settings/data-transfer', icon: <IconDatabaseExport size={17} /> },
  { label: 'Appearance', to: '/admin/settings/appearance', icon: <IconPalette size={17} /> },
];

export default function AdminShell() {
  const location = useLocation();
  const navigate = useNavigate();
  const loggedIn = useAdminAuth((state) => state.loggedIn);
  const user = useAdminAuth((state) => state.user);
  const logout = useAdminAuth((state) => state.logout);
  const [collapsed, setCollapsed] = useState(() => localStorage.getItem('admin_collapsed') === '1');
  const [mobileOpen, setMobileOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [openGroups, setOpenGroups] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const active = MENU.find((item) => item.children?.some((child) => location.pathname.startsWith(child.to)));
    if (active) setOpenGroups((current) => ({ ...current, [active.label]: true }));
    setMobileOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    localStorage.setItem('admin_collapsed', collapsed ? '1' : '0');
  }, [collapsed]);

  const activeGroup = useMemo(
    () => MENU.find((item) => item.children?.some((child) => location.pathname.startsWith(child.to)))?.label,
    [location.pathname],
  );

  const crumbs = useMemo(() => {
    const group = MENU.find((item) =>
      item.children?.some((child) => location.pathname === child.to || location.pathname.startsWith(`${child.to}/`)),
    );
    const child = group?.children?.find(
      (entry) => location.pathname === entry.to || location.pathname.startsWith(`${entry.to}/`),
    );
    if (group && child) return [group.label, child.label];

    const segments = location.pathname.split('/').filter(Boolean).slice(1);
    const meaningful = segments.filter(
      (segment) => !['view', 'edit', 'create'].includes(segment) && !/^\d+$/.test(segment),
    );
    if (!meaningful.length) return [];
    const labels = meaningful.map(breadcrumbLabel);
    if (activeGroup && labels[0] !== activeGroup) labels.unshift(activeGroup);
    return labels;
  }, [location.pathname, activeGroup]);

  if (!loggedIn) {
    return <Navigate to="/admin/login" replace />;
  }

  const initialsValue = initials(user?.name ?? 'Admin');

  return (
    <div className="admin-shell">
      <header className="admin-header">
        <div style={{ display: 'flex', alignItems: 'center', gap: 14, minWidth: 0 }}>
          <button type="button" className="admin-icon-button admin-mobile-only" onClick={() => setMobileOpen((open) => !open)} aria-label="Toggle menu">
            <IconMenu2 size={20} />
          </button>
          <button type="button" className="admin-icon-button admin-collapse-button" onClick={() => setCollapsed((value) => !value)} aria-label="Collapse sidebar">
            <IconMenu2 size={19} />
          </button>
          <Link to="/admin" className="admin-logo">
            <span className="admin-logo-mark">E</span>
            {!collapsed && <span>EzCommerce</span>}
          </Link>
        </div>

        <form
          className="admin-search"
          onSubmit={(event) => {
            event.preventDefault();
            navigate(`/admin/catalog/products?q=${encodeURIComponent(search)}`);
          }}
        >
          <span style={{ position: 'absolute', left: 12, display: 'flex', color: '#9ca3af' }}>
            <IconSearch size={16} />
          </span>
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search products, orders, customers..."
            aria-label="Global search"
          />
          <span className="admin-kbd">⌘ K</span>
        </form>

        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <Link to="/" target="_blank" className="admin-icon-button" title="Visit store">
            <IconExternalLink size={18} />
          </Link>
          <button type="button" className="admin-icon-button" aria-label="Notifications">
            <IconBell size={18} />
            <span className="admin-notification-dot" />
          </button>
          <Menu shadow="md" width={220} position="bottom-end">
            <Menu.Target>
              <button type="button" className="admin-profile">
                <span className="admin-avatar">{initialsValue}</span>
                {!collapsed && (
                  <span className="admin-profile-copy">
                    <strong>{user?.name ?? 'Administrator'}</strong>
                    <span>{user?.email ?? 'admin@example.com'}</span>
                  </span>
                )}
                <IconChevronDown size={14} />
              </button>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Label>Signed in as {user?.email ?? 'admin@example.com'}</Menu.Label>
              <Menu.Item leftSection={<IconUser size={15} />} component={Link} to="/admin/settings/users">
                My account
              </Menu.Item>
              <Menu.Item leftSection={<IconSettings size={15} />} component={Link} to="/admin/configuration">
                Configuration
              </Menu.Item>
              <Menu.Divider />
              <Menu.Item
                color="red"
                leftSection={<IconLogout size={15} />}
                onClick={async () => {
                  await logout();
                  navigate('/admin/login');
                }}
              >
                Sign out
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </div>
      </header>

      <aside className="admin-sidebar" data-collapsed={collapsed} data-mobile-open={mobileOpen}>
        <div className="admin-sidebar-nav">
          {MENU.map((item) => {
            const isGroupActive = item.children?.some((child) => location.pathname.startsWith(child.to));
            const isOpen = openGroups[item.label] ?? isGroupActive;

            if (item.children) {
              return (
                <div className="admin-menu-group" key={item.label}>
                  <button
                    type="button"
                    className="admin-menu-link"
                    data-active={isGroupActive && !isOpen}
                    onClick={() =>
                      collapsed
                        ? navigate(item.children![0].to)
                        : setOpenGroups((current) => ({ ...current, [item.label]: !isOpen }))
                    }
                    title={item.label}
                  >
                    <span className="admin-menu-icon">{item.icon}</span>
                    {!collapsed && (
                      <>
                        <span style={{ flex: 1, textAlign: 'left' }}>{item.label}</span>
                        <IconChevronRight
                          size={14}
                          style={{ transform: isOpen ? 'rotate(90deg)' : undefined, transition: 'transform .18s' }}
                        />
                      </>
                    )}
                  </button>

                  {!collapsed && isOpen && (
                    <div className="admin-submenu" data-open={isOpen}>
                      {item.children.map((child) => (
                        <NavLink key={child.to} to={child.to} data-active={location.pathname === child.to}>
                          {child.label}
                        </NavLink>
                      ))}
                    </div>
                  )}
                </div>
              );
            }

            return (
              <div className="admin-menu-group" key={item.label}>
                <NavLink to={item.to!} end={item.to === '/admin'} className="admin-menu-link" data-active={item.to === '/admin' ? location.pathname === '/admin' : location.pathname.startsWith(item.to!)} title={item.label}>
                  <span className="admin-menu-icon">{item.icon}</span>
                  {!collapsed && <span>{item.label}</span>}
                </NavLink>
              </div>
            );
          })}
        </div>
      </aside>

      <main className="admin-main" data-collapsed={collapsed}>
        <div className="admin-breadcrumb">
          <Link to="/admin">Dashboard</Link>
          {crumbs.map((crumb) => [
            <IconChevronRight size={12} key={`${crumb}-icon`} />,
            <span key={crumb}>{crumb}</span>,
          ])}
        </div>
        <Outlet />
      </main>
    </div>
  );
}
