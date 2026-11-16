import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:http/http.dart' as http;
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import 'admin_reporting.dart' show EzBarChart;
import '../../../core/utils/download.dart';

// ---------------------------------------------------------------------------
// Login admin
// ---------------------------------------------------------------------------

class AdminLoginPage extends StatefulWidget {
  const AdminLoginPage({super.key});

  @override
  State<AdminLoginPage> createState() => _AdminLoginPageState();
}

class _AdminLoginPageState extends State<AdminLoginPage> {
  final _email = TextEditingController();
  final _password = TextEditingController();

  @override
  Widget build(BuildContext context) {
    final admin = context.watch<AdminProvider>();
    return Scaffold(
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 480),
            child: Container(
              padding: const EdgeInsets.all(48),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                border: Border.all(color: AppColors.zinc200),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'EzCommerce Admin',
                    style: TextStyle(
                      fontFamily: 'DM Serif Display',
                      fontSize: 28,
                      color: AppColors.navyBlue,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Sign in to your admin account.',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 14,
                      color: AppColors.zinc500,
                    ),
                  ),
                  const SizedBox(height: 24),
                  if (admin.error != null)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: Text(
                        admin.error!,
                        style: const TextStyle(
                          color: AppColors.red600,
                          fontSize: 13,
                        ),
                      ),
                    ),
                  TextField(
                    controller: _email,
                    decoration: const InputDecoration(hintText: 'Email'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: _password,
                    obscureText: true,
                    decoration: const InputDecoration(hintText: 'Password'),
                  ),
                  const SizedBox(height: 20),
                  PrimaryButton(
                    label: 'Sign In',
                    expand: true,
                    isLoading: admin.loading,
                    onPressed: () async {
                      final ok = await admin.login(_email.text, _password.text);
                      if (ok && context.mounted) context.go('/admin');
                    },
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Shell admin: sidebar + contenido
// ---------------------------------------------------------------------------

class AdminShell extends StatelessWidget {
  const AdminShell({super.key, required this.child});
  final Widget child;

  static const groups = <(String, List<(String, IconData, String)>)>[
    ('', [('Dashboard', Icons.space_dashboard_outlined, '/admin')]),
    (
      'Sales',
      [
        ('Orders', Icons.receipt_long_outlined, '/admin/orders'),
        ('Invoices', Icons.request_quote_outlined, '/admin/invoices'),
        ('Shipments', Icons.local_shipping_outlined, '/admin/shipments'),
        ('Refunds', Icons.assignment_return_outlined, '/admin/refunds'),
        ('Reporting', Icons.query_stats_outlined, '/admin/reporting'),
      ],
    ),
    (
      'Catalog',
      [
        ('Products', Icons.inventory_2_outlined, '/admin/products'),
        ('Categories', Icons.account_tree_outlined, '/admin/categories'),
        ('Attributes', Icons.tune_outlined, '/admin/attributes'),
        ('Attribute Families', Icons.folder_copy_outlined, '/admin/families'),
      ],
    ),
    (
      'Customers',
      [
        ('Customers', Icons.people_outline, '/admin/customers'),
        ('Customer Groups', Icons.group_outlined, '/admin/customers/groups'),
        ('Reviews', Icons.star_border_outlined, '/admin/customers/reviews'),
      ],
    ),
    (
      'Marketing',
      [
        ('Cart Rules', Icons.local_offer_outlined, '/admin/cart-rules'),
        ('Catalog Rules', Icons.sell_outlined, '/admin/catalog-rules'),
        ('Search & SEO', Icons.travel_explore_outlined, '/admin/search-seo'),
        (
          'Communications',
          Icons.mark_email_unread_outlined,
          '/admin/communications',
        ),
      ],
    ),
    (
      'Content',
      [
        ('Pages', Icons.description_outlined, '/admin/cms'),
        ('Appearance', Icons.palette_outlined, '/admin/appearance'),
      ],
    ),
    (
      'Settings',
      [
        ('Settings', Icons.settings_outlined, '/admin/settings'),
        ('Configuration', Icons.tune, '/admin/configuration'),
        ('Data Transfer', Icons.import_export_outlined, '/admin/data-transfer'),
      ],
    ),
  ];

  static String _labelFor(String path) {
    for (final group in groups) {
      for (final item in group.$2) {
        if (item.$3 == path) return item.$1;
      }
    }
    return 'Dashboard';
  }

  @override
  Widget build(BuildContext context) {
    final admin = context.watch<AdminProvider>();
    if (!admin.isLoggedIn) {
      return const AdminLoginPage();
    }
    final isNarrow = MediaQuery.of(context).size.width < 980;
    final path = GoRouterState.of(context).uri.path;
    final title = _labelFor(path);
    final section = groups
        .where((group) => group.$2.any((item) => item.$3 == path))
        .map((group) => group.$1)
        .firstWhere((label) => label.isNotEmpty, orElse: () => 'Dashboard');

    final sidebar = _AdminSidebar(
      path: path,
      email: admin.me?['email']?.toString() ?? '',
      onLogout: () async {
        await admin.logout();
        if (context.mounted) context.go('/admin/login');
      },
    );

    return Scaffold(
      backgroundColor: const Color(0xFFF6F7F9),
      drawer: isNarrow ? Drawer(width: 260, child: sidebar) : null,
      body: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (!isNarrow) SizedBox(width: 260, child: sidebar),
          Expanded(
            child: Column(
              children: [
                Container(
                  height: 72,
                  padding: EdgeInsets.symmetric(horizontal: isNarrow ? 12 : 28),
                  decoration: const BoxDecoration(
                    color: Colors.white,
                    border: Border(
                      bottom: BorderSide(color: AppColors.zinc200),
                    ),
                  ),
                  child: Row(
                    children: [
                      if (isNarrow)
                        Builder(
                          builder: (context) => IconButton(
                            icon: const Icon(Icons.menu, size: 22),
                            onPressed: () => Scaffold.of(context).openDrawer(),
                          ),
                        ),
                      Text(
                        section == 'Dashboard' ? 'Dashboard' : section,
                        style: const TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 12,
                          color: AppColors.zinc400,
                        ),
                      ),
                      if (section != 'Dashboard') ...[
                        const Padding(
                          padding: EdgeInsets.symmetric(horizontal: 8),
                          child: Icon(
                            Icons.chevron_right,
                            size: 16,
                            color: AppColors.zinc300,
                          ),
                        ),
                        Text(
                          title,
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 13,
                            fontWeight: FontWeight.w500,
                            color: AppColors.gray900,
                          ),
                        ),
                      ],
                      const Spacer(),
                      if (!isNarrow)
                        Container(
                          width: 260,
                          height: 40,
                          padding: const EdgeInsets.symmetric(horizontal: 12),
                          decoration: BoxDecoration(
                            color: AppColors.zinc100,
                            borderRadius: BorderRadius.circular(
                              EzTheme.radiusLg,
                            ),
                          ),
                          child: const Row(
                            children: [
                              Icon(
                                Icons.search,
                                size: 18,
                                color: AppColors.zinc400,
                              ),
                              SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  'Search admin',
                                  style: TextStyle(
                                    fontFamily: EzTheme.sans,
                                    fontSize: 13,
                                    color: AppColors.zinc400,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      const SizedBox(width: 16),
                      IconButton(
                        tooltip: 'Visit store',
                        icon: const Icon(Icons.storefront_outlined, size: 20),
                        onPressed: () => context.go('/'),
                      ),
                      const SizedBox(width: 8),
                      CircleAvatar(
                        radius: 17,
                        backgroundColor: AppColors.navyBlue,
                        child: Text(
                          (admin.me?['name']?.toString() ?? 'A')
                              .characters
                              .first
                              .toUpperCase(),
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 13,
                            color: Colors.white,
                          ),
                        ),
                      ),
                      if (!isNarrow) ...[
                        const SizedBox(width: 10),
                        SizedBox(
                          width: 170,
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                admin.me?['name']?.toString() ??
                                    'Administrator',
                                overflow: TextOverflow.ellipsis,
                                style: const TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 13,
                                  fontWeight: FontWeight.w500,
                                  color: AppColors.gray900,
                                ),
                              ),
                              Text(
                                admin.me?['email']?.toString() ?? '',
                                overflow: TextOverflow.ellipsis,
                                style: const TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 11,
                                  color: AppColors.zinc500,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
                Expanded(child: child),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _AdminSidebar extends StatelessWidget {
  const _AdminSidebar({
    required this.path,
    required this.email,
    required this.onLogout,
  });

  final String path;
  final String email;
  final Future<void> Function() onLogout;

  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.white,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            height: 72,
            padding: const EdgeInsets.symmetric(horizontal: 24),
            decoration: const BoxDecoration(
              border: Border(bottom: BorderSide(color: AppColors.zinc200)),
            ),
            child: Row(
              children: [
                const Text(
                  'EzCommerce',
                  style: TextStyle(
                    fontFamily: EzTheme.serif,
                    fontSize: 22,
                    color: AppColors.navyBlue,
                  ),
                ),
                const Text(
                  '.',
                  style: TextStyle(
                    fontFamily: EzTheme.serif,
                    fontSize: 22,
                    color: AppColors.red600,
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.fromLTRB(14, 18, 14, 18),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  for (final group in AdminShell.groups) ...[
                    if (group.$1.isNotEmpty)
                      Padding(
                        padding: const EdgeInsets.fromLTRB(12, 18, 12, 8),
                        child: Text(
                          group.$1.toUpperCase(),
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 10,
                            letterSpacing: 1.3,
                            fontWeight: FontWeight.w600,
                            color: AppColors.zinc400,
                          ),
                        ),
                      ),
                    for (final item in group.$2)
                      _AdminNavItem(
                        label: item.$1,
                        icon: item.$2,
                        route: item.$3,
                        selected: path == item.$3,
                        onTap: () {
                          if (Scaffold.of(context).hasDrawer) {
                            Navigator.of(context).pop();
                          }
                          context.go(item.$3);
                        },
                      ),
                  ],
                ],
              ),
            ),
          ),
          Container(
            padding: const EdgeInsets.all(14),
            decoration: const BoxDecoration(
              border: Border(top: BorderSide(color: AppColors.zinc200)),
            ),
            child: Row(
              children: [
                const CircleAvatar(
                  radius: 16,
                  backgroundColor: AppColors.zinc100,
                  child: Icon(
                    Icons.person_outline,
                    size: 18,
                    color: AppColors.navyBlue,
                  ),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    email,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 12,
                      color: AppColors.zinc600,
                    ),
                  ),
                ),
                IconButton(
                  tooltip: 'Logout',
                  icon: const Icon(
                    Icons.logout,
                    size: 18,
                    color: AppColors.zinc500,
                  ),
                  onPressed: onLogout,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _AdminNavItem extends StatelessWidget {
  const _AdminNavItem({
    required this.label,
    required this.icon,
    required this.route,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final IconData icon;
  final String route;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 2),
      child: Material(
        color: selected ? AppColors.zinc100 : Colors.transparent,
        borderRadius: BorderRadius.circular(EzTheme.radiusLg),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(EzTheme.radiusLg),
          child: Container(
            height: 40,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(EzTheme.radiusLg),
              border: Border(
                left: BorderSide(
                  color: selected ? AppColors.navyBlue : Colors.transparent,
                  width: 3,
                ),
              ),
            ),
            child: Row(
              children: [
                Icon(
                  icon,
                  size: 19,
                  color: selected ? AppColors.navyBlue : AppColors.zinc500,
                ),
                const SizedBox(width: 11),
                Expanded(
                  child: Text(
                    label,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 13,
                      fontWeight: selected ? FontWeight.w500 : FontWeight.w400,
                      color: selected ? AppColors.navyBlue : AppColors.zinc600,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

class AdminDashboardPage extends StatefulWidget {
  const AdminDashboardPage({super.key});

  @override
  State<AdminDashboardPage> createState() => _AdminDashboardPageState();
}

class _AdminDashboardPageState extends State<AdminDashboardPage> {
  Map<String, dynamic>? _overAll;
  Map<String, dynamic>? _today;
  Map<String, dynamic>? _salesReport;
  List<Map<String, dynamic>> _topProducts = [];
  List<Map<String, dynamic>> _topCustomers = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final overAll = await admin.get(
      'admin/dashboard/stats',
      query: {'type': 'over-all'},
    );
    final today = await admin.get(
      'admin/dashboard/stats',
      query: {'type': 'today'},
    );
    final salesReport = await admin.get('admin/reporting/sales');
    final topProducts = await admin.get('admin/reporting/products');
    final topCustomers = await admin.get('admin/reporting/customers');
    if (mounted) {
      setState(() {
        _overAll = overAll is Map ? Map<String, dynamic>.from(overAll) : null;
        _today = today is Map ? Map<String, dynamic>.from(today) : null;
        _salesReport = salesReport is Map
            ? Map<String, dynamic>.from(salesReport)
            : null;
        _topProducts = decodeList(topProducts).take(5).toList();
        _topCustomers = decodeList(topCustomers).take(5).toList();
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(
        child: CircularProgressIndicator(color: AppColors.navyBlue),
      );
    }
    final stats = _overAll ?? const {};
    Map<String, dynamic> metric(String key) =>
        Map<String, dynamic>.from((stats[key] ?? const {}) as Map);
    String money(dynamic value) =>
        '€${((value as num?) ?? 0).toStringAsFixed(2)}';

    return SingleChildScrollView(
      padding: EdgeInsets.all(
        MediaQuery.of(context).size.width < 640 ? 16 : 24,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Dashboard',
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: MediaQuery.of(context).size.width < 640 ? 20 : 24,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 24),
          LayoutBuilder(
            builder: (context, constraints) {
              final columns = constraints.maxWidth >= 1080
                  ? 4
                  : (constraints.maxWidth >= 560 ? 2 : 2);
              final cardWidth =
                  (constraints.maxWidth - 16 * (columns - 1)) / columns;
              return Wrap(
                spacing: 16,
                runSpacing: 16,
                children: [
                  _MetricCard(
                    width: cardWidth,
                    title: 'Total Sales',
                    value: money(metric('total_sales')['current']),
                    progress: ((metric('total_sales')['progress'] ?? 0) as num)
                        .toDouble(),
                  ),
                  _MetricCard(
                    width: cardWidth,
                    title: 'Total Orders',
                    value: '${metric('total_orders')['current'] ?? 0}',
                    progress: ((metric('total_orders')['progress'] ?? 0) as num)
                        .toDouble(),
                  ),
                  _MetricCard(
                    width: cardWidth,
                    title: 'New Customers',
                    value: '${metric('total_customers')['current'] ?? 0}',
                    progress:
                        ((metric('total_customers')['progress'] ?? 0) as num)
                            .toDouble(),
                  ),
                  _MetricCard(
                    width: cardWidth,
                    title: 'Average Sales',
                    value: money(metric('avg_sales')['current']),
                    progress: ((metric('avg_sales')['progress'] ?? 0) as num)
                        .toDouble(),
                  ),
                ],
              );
            },
          ),
          const SizedBox(height: 32),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              border: Border.all(color: AppColors.zinc200),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Sales — last 30 days',
                  style: TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 15,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const SizedBox(height: 16),
                Builder(
                  builder: (context) {
                    final series = decodeList(_salesReport?['series']);
                    if (series.isEmpty) {
                      return const Text(
                        'No sales in this period.',
                        style: TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 12,
                          color: AppColors.zinc500,
                        ),
                      );
                    }
                    return EzBarChart(
                      bars: [
                        for (final point in series)
                          (
                            '${point['label']}',
                            (point['sales'] as num? ?? 0).toDouble(),
                          ),
                      ],
                    );
                  },
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          LayoutBuilder(
            builder: (context, constraints) {
              final stack = constraints.maxWidth < 720;
              final cards = [
                RankingCard(
                  title: 'Top products',
                  items: [
                    for (final product in _topProducts)
                      '${product['name']} — ${product['qty']} sold',
                  ],
                ),
                RankingCard(
                  title: 'Top customers',
                  items: [
                    for (final customer in _topCustomers)
                      '${customer['first_name']} ${customer['last_name']} — €${((customer['total'] as num? ?? 0)).toStringAsFixed(2)}',
                  ],
                ),
              ];
              if (stack) {
                return Column(
                  children: [
                    for (var i = 0; i < cards.length; i++) ...[
                      if (i > 0) const SizedBox(height: 16),
                      SizedBox(width: double.infinity, child: cards[i]),
                    ],
                  ],
                );
              }
              return IntrinsicHeight(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Expanded(child: cards[0]),
                    const SizedBox(width: 16),
                    Expanded(child: cards[1]),
                  ],
                ),
              );
            },
          ),
          const SizedBox(height: 16),
          // Órdenes de hoy
          if (_today != null)
            Container(
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                border: Border.all(color: AppColors.zinc200),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Text(
                        "Today's Orders",
                        style: TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 17,
                          fontWeight: FontWeight.w600,
                          color: AppColors.gray900,
                        ),
                      ),
                      const SizedBox(width: 10),
                      Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 10,
                          vertical: 3,
                        ),
                        decoration: BoxDecoration(
                          color: AppColors.zinc100,
                          borderRadius: BorderRadius.circular(30),
                        ),
                        child: Text(
                          '${_today!['orders_today'] ?? 0}',
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 12,
                            fontWeight: FontWeight.w600,
                            color: AppColors.zinc600,
                          ),
                        ),
                      ),
                      const Spacer(),
                      TextButton(
                        onPressed: () => context.go('/admin/orders'),
                        child: const Text(
                          'View all orders',
                          style: TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 13,
                            fontWeight: FontWeight.w500,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const Divider(height: 24),
                  if (((_today!['recent_orders'] as List?) ?? []).isEmpty)
                    const Padding(
                      padding: EdgeInsets.symmetric(vertical: 26),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            Icons.receipt_long_outlined,
                            size: 22,
                            color: AppColors.zinc300,
                          ),
                          SizedBox(width: 10),
                          Text(
                            'No orders have been placed today yet.',
                            style: TextStyle(
                              fontFamily: EzTheme.sans,
                              fontSize: 14,
                              color: AppColors.zinc500,
                            ),
                          ),
                        ],
                      ),
                    )
                  else
                    ...((_today!['recent_orders'] as List?) ?? []).map((order) {
                      final row = Map<String, dynamic>.from(order as Map);
                      final status = (row['status'] ?? '').toString();
                      final color = status == 'completed'
                          ? AppColors.green600
                          : status == 'processing'
                          ? AppColors.blue700
                          : AppColors.amber500;
                      return Padding(
                        padding: const EdgeInsets.symmetric(vertical: 9),
                        child: Row(
                          children: [
                            Text(
                              '#${row['increment_id']}',
                              style: const TextStyle(
                                fontFamily: EzTheme.sans,
                                fontSize: 13,
                                fontWeight: FontWeight.w600,
                                color: AppColors.navyBlue,
                              ),
                            ),
                            const SizedBox(width: 16),
                            Expanded(
                              child: Text(
                                '${row['customer_email'] ?? ''}',
                                overflow: TextOverflow.ellipsis,
                                style: const TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 13,
                                  color: AppColors.zinc600,
                                ),
                              ),
                            ),
                            Container(
                              padding: const EdgeInsets.symmetric(
                                horizontal: 10,
                                vertical: 3,
                              ),
                              decoration: BoxDecoration(
                                color: color.withValues(alpha: 0.12),
                                borderRadius: BorderRadius.circular(30),
                              ),
                              child: Text(
                                status.isEmpty
                                    ? 'pending'
                                    : status[0].toUpperCase() +
                                          status.substring(1),
                                style: TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 12,
                                  color: color,
                                ),
                              ),
                            ),
                            const SizedBox(width: 20),
                            SizedBox(
                              width: 90,
                              child: Text(
                                money(
                                  row['base_grand_total'] ?? row['grand_total'],
                                ),
                                textAlign: TextAlign.right,
                                style: const TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 13,
                                  fontWeight: FontWeight.w600,
                                  color: AppColors.gray900,
                                ),
                              ),
                            ),
                          ],
                        ),
                      );
                    }),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

class _MetricCard extends StatelessWidget {
  const _MetricCard({
    required this.title,
    required this.value,
    required this.progress,
    this.width = 240,
  });
  final String title;
  final String value;
  final double progress;
  final double width;

  @override
  Widget build(BuildContext context) {
    final positive = progress >= 0;
    return Container(
      width: width,
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: 13,
              color: AppColors.zinc500,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            value,
            style: const TextStyle(
              fontFamily: 'Poppins',
              fontSize: 24,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 8),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: const EdgeInsets.only(top: 2),
                child: Icon(
                  positive ? Icons.trending_up : Icons.trending_down,
                  size: 15,
                  color: positive ? AppColors.green600 : AppColors.red600,
                ),
              ),
              const SizedBox(width: 4),
              Expanded(
                child: Text(
                  '${positive ? '+' : ''}${progress.toStringAsFixed(1)}% vs previous period',
                  style: TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 12,
                    height: 1.3,
                    color: positive ? AppColors.green600 : AppColors.red600,
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// DataGrid genérico (contrato Bagisto)
// ---------------------------------------------------------------------------

class EzDataGrid extends StatefulWidget {
  const EzDataGrid({
    super.key,
    required this.gridPath,
    required this.title,
    this.detailBuilder,
    this.extraActions,
    this.headerActions = const [],
  });

  final String gridPath;
  final String title;
  final Widget Function(Map<String, dynamic> row)? detailBuilder;
  final List<Widget> Function(Map<String, dynamic> row)? extraActions;
  final List<Widget> headerActions;

  @override
  State<EzDataGrid> createState() => _EzDataGridState();
}

class _EzDataGridState extends State<EzDataGrid> {
  Map<String, dynamic>? _grid;
  bool _loading = true;
  String _search = '';
  int _page = 1;
  final ScrollController _gridScroll = ScrollController();

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _gridScroll.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final body = <String, dynamic>{
      'pagination': {'page': _page, 'per_page': 15},
    };
    if (_search.isNotEmpty) body['filters'] = {'all': _search};
    final grid = await admin.datagrid(widget.gridPath, body: body);
    if (mounted) {
      setState(() {
        _grid = grid;
        _loading = false;
      });
    }
  }

  /// Exporta el grid con los filtros activos (descarga en web).
  Future<void> _exportCsv() async {
    final token = ApiClient.instance.adminToken;
    if (token == null) return;
    final uri = Uri.parse(
      '${ApiClient.baseUrl}/api/${widget.gridPath}?format=csv',
    );
    final response = await http.post(
      uri,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': 'Bearer $token',
      },
      body: _search.isNotEmpty
          ? '{"filters":{"all":"$_search"}}'
          : '{"filters":{}}',
    );
    if (response.statusCode != 200 || !mounted) return;
    downloadFile('${widget.gridPath.split('/').last}.csv', response.bodyBytes);
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(const SnackBar(content: Text('Export CSV downloaded.')));
  }

  @override
  Widget build(BuildContext context) {
    final records = ((_grid?['records'] ?? const []) as List)
        .map((e) => Map<String, dynamic>.from(e as Map))
        .toList();
    final columns = ((_grid?['columns'] ?? const []) as List)
        .map((e) => Map<String, dynamic>.from(e as Map))
        .toList();
    final meta = Map<String, dynamic>.from((_grid?['meta'] ?? const {}) as Map);
    final total = (meta['total'] ?? 0) as int;
    final lastPage = (meta['last_page'] ?? 1) as int;
    final phone = MediaQuery.of(context).size.width < 640;

    return SingleChildScrollView(
      padding: EdgeInsets.fromLTRB(
        phone ? 16 : 24,
        phone ? 16 : 24,
        phone ? 16 : 24,
        phone ? 88 : 24,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          LayoutBuilder(
            builder: (context, constraints) {
              final narrow = constraints.maxWidth < 720;
              final title = Text(
                widget.title,
                style: TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: phone ? 20 : 24,
                  fontWeight: FontWeight.w500,
                ),
              );
              final search = TextField(
                decoration: const InputDecoration(
                  hintText: 'Search…',
                  prefixIcon: Icon(Icons.search, size: 20),
                  isDense: true,
                ),
                onSubmitted: (value) {
                  _search = value;
                  _page = 1;
                  _load();
                },
              );
              final actions = Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextButton.icon(
                    onPressed: _exportCsv,
                    icon: const Icon(Icons.download_outlined, size: 18),
                    label: const Text(
                      'Export CSV',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 13),
                    ),
                  ),
                  ...widget.headerActions,
                ],
              );
              if (narrow) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    title,
                    const SizedBox(height: 12),
                    search,
                    actions,
                  ],
                );
              }
              return Row(
                children: [
                  Expanded(child: title),
                  SizedBox(width: 280, child: search),
                  const SizedBox(width: 12),
                  actions,
                ],
              );
            },
          ),
          const SizedBox(height: 16),
          Container(
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              border: Border.all(color: AppColors.zinc200),
            ),
            child: _loading
                ? const SizedBox(
                    height: 300,
                    child: Center(
                      child: CircularProgressIndicator(
                        color: AppColors.navyBlue,
                      ),
                    ),
                  )
                : LayoutBuilder(
                    builder: (context, box) => Scrollbar(
                      controller: _gridScroll,
                      thumbVisibility: true,
                      scrollbarOrientation: ScrollbarOrientation.bottom,
                      child: SingleChildScrollView(
                        controller: _gridScroll,
                        scrollDirection: Axis.horizontal,
                        child: ConstrainedBox(
                          constraints: BoxConstraints(minWidth: box.maxWidth),
                          child: DataTable(
                            showCheckboxColumn: false,
                            headingRowColor: WidgetStateProperty.all(
                              AppColors.zinc100,
                            ),
                            headingTextStyle: const TextStyle(
                              fontFamily: 'Poppins',
                              fontSize: 13,
                              fontWeight: FontWeight.w600,
                            ),
                            dataTextStyle: const TextStyle(
                              fontFamily: 'Poppins',
                              fontSize: 13,
                            ),
                            columns:
                                columns
                                    .where(
                                      (column) =>
                                          column['index'] != '__primary',
                                    )
                                    .map(
                                      (column) => DataColumn(
                                        label: Text('${column['label']}'),
                                        onSort: (_, _) {},
                                      ),
                                    )
                                    .toList()
                                  ..add(const DataColumn(label: Text(''))),
                            rows: records.map((row) {
                              return DataRow(
                                onSelectChanged: widget.detailBuilder == null
                                    ? null
                                    : (_) => _openDetail(row),
                                cells: [
                                  ...columns
                                      .where(
                                        (column) =>
                                            column['index'] != '__primary',
                                      )
                                      .map(
                                        (column) => DataCell(
                                          _gridCell(
                                            '${column['index']}',
                                            row[column['index']],
                                          ),
                                        ),
                                      ),
                                  DataCell(
                                    Row(
                                      mainAxisSize: MainAxisSize.min,
                                      children: [
                                        if (widget.detailBuilder != null)
                                          IconButton(
                                            icon: const Icon(
                                              Icons.visibility_outlined,
                                              size: 18,
                                            ),
                                            onPressed: () => _openDetail(row),
                                          ),
                                        ...(widget.extraActions?.call(row) ??
                                            const []),
                                      ],
                                    ),
                                  ),
                                ],
                              );
                            }).toList(),
                          ),
                        ),
                      ),
                    ),
                  ),
          ),
          const SizedBox(height: 16),
          // Paginación
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Text(
                '$total records',
                style: TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 13,
                  color: AppColors.zinc500,
                ),
              ),
              const SizedBox(width: 24),
              IconButton(
                onPressed: _page > 1
                    ? () {
                        _page--;
                        _load();
                      }
                    : null,
                icon: const Icon(Icons.chevron_left),
              ),
              Text(
                '$_page / $lastPage',
                style: const TextStyle(fontFamily: 'Poppins', fontSize: 13),
              ),
              IconButton(
                onPressed: _page < lastPage
                    ? () {
                        _page++;
                        _load();
                      }
                    : null,
                icon: const Icon(Icons.chevron_right),
              ),
            ],
          ),
        ],
      ),
    );
  }

  /// Pills de estado para las columnas `status`/`state`; texto para el resto.
  Widget _gridCell(String index, dynamic raw) {
    final text = '${raw ?? ''}';
    if (index == 'status' || index == 'state') {
      if (text.isEmpty) return const Text('—');
      final value = text.toLowerCase();
      String label = text;
      Color color;
      if (['1', 'true', 'active', 'enabled'].contains(value)) {
        label = 'Active';
        color = AppColors.green600;
      } else if (['0', 'false', 'inactive', 'disabled'].contains(value)) {
        label = 'Inactive';
        color = AppColors.zinc500;
      } else if (['completed', 'paid', 'processing'].contains(value)) {
        color = AppColors.green600;
      } else if (['pending', 'on_hold'].contains(value)) {
        color = AppColors.amber500;
      } else if (['canceled', 'closed', 'refunded', 'fraud'].contains(value)) {
        color = AppColors.red600;
      } else {
        color = AppColors.zinc600;
      }
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
        decoration: BoxDecoration(
          color: color.withValues(alpha: .1),
          borderRadius: BorderRadius.circular(999),
        ),
        child: Text(
          label,
          style: TextStyle(
            fontFamily: EzTheme.sans,
            fontSize: 12,
            fontWeight: FontWeight.w500,
            color: color,
          ),
        ),
      );
    }
    // Importes: el backend puede mandar números planos (p. ej. el precio del
    // producto) y el grid los muestra con la moneda de la tienda.
    final lower = index.toLowerCase();
    if (lower.contains('price') ||
        lower == 'total' ||
        lower.endsWith('_total')) {
      final amount = raw is num ? raw.toDouble() : double.tryParse(text);
      if (amount != null) return Text('€${amount.toStringAsFixed(2)}');
    }
    return Text(text);
  }

  Future<void> _openDetail(Map<String, dynamic> row) async {
    final id = row['__primary'] ?? row['id'];
    final admin = context.read<AdminProvider>();
    String? path;
    if (widget.gridPath.contains('sales/orders')) {
      path = 'admin/sales/orders/view/$id';
    } else if (widget.gridPath.contains('customers')) {
      path = 'admin/customers/view/$id';
    } else if (widget.gridPath.contains('catalog/products')) {
      path = 'admin/catalog/products/edit/$id';
    }
    if (path == null) return;
    final data = await admin.get(path);
    if (!mounted || data is! Map) return;
    await showDialog(
      context: context,
      builder: (context) => AlertDialog(
        insetPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
        title: Text('Detail #$id'),
        content: SizedBox(
          width: 560,
          child:
              widget.detailBuilder?.call(Map<String, dynamic>.from(data)) ??
              Text('$data'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Pantallas de datos
// ---------------------------------------------------------------------------

class RankingCard extends StatelessWidget {
  const RankingCard({super.key, required this.title, required this.items});
  final String title;
  final List<String> items;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: const TextStyle(
              fontFamily: 'Poppins',
              fontSize: 15,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 12),
          if (items.isEmpty)
            const Text(
              'No data.',
              style: TextStyle(
                fontFamily: 'Poppins',
                fontSize: 12,
                color: AppColors.zinc500,
              ),
            )
          else
            for (final (index, item) in items.indexed)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(
                  '${index + 1}.  $item',
                  style: const TextStyle(fontFamily: 'Poppins', fontSize: 12),
                ),
              ),
        ],
      ),
    );
  }
}

class AdminProductsPage extends StatelessWidget {
  const AdminProductsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        EzDataGrid(
          gridPath: 'admin/catalog/products',
          title: 'Products',
          headerActions: [_ReindexFlatsButton(compact: true)],
          extraActions: (row) => [
            IconButton(
              icon: const Icon(Icons.edit_outlined, size: 18),
              tooltip: 'Edit',
              onPressed: () => context.go('/admin/products/edit/${row['id']}'),
            ),
            IconButton(
              icon: const Icon(Icons.copy_outlined, size: 18),
              tooltip: 'Copy product',
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                try {
                  final result = await admin.post(
                    'admin/catalog/products/copy/${row['id']}',
                  );
                  final newId = result is Map && result['data'] is Map
                      ? (result['data'] as Map)['id']
                      : null;
                  if (context.mounted && newId is int) {
                    context.go('/admin/products/edit/$newId');
                  }
                } catch (error) {
                  if (context.mounted) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(
                        content: Text('$error'),
                        backgroundColor: AppColors.red600,
                      ),
                    );
                  }
                }
              },
            ),
            IconButton(
              icon: const Icon(
                Icons.delete_outline,
                size: 18,
                color: AppColors.red600,
              ),
              onPressed: () async {
                final admin = context.read<AdminProvider>();
                final confirmed = await showDialog<bool>(
                  context: context,
                  builder: (context) => AlertDialog(
                    title: const Text('Delete product?'),
                    actions: [
                      TextButton(
                        onPressed: () => Navigator.pop(context, false),
                        child: const Text('Cancel'),
                      ),
                      TextButton(
                        onPressed: () => Navigator.pop(context, true),
                        child: const Text('Delete'),
                      ),
                    ],
                  ),
                );
                if (confirmed == true) {
                  await admin.delete('admin/catalog/products/${row['id']}');
                }
              },
            ),
          ],
        ),
        Positioned(
          right: 24,
          bottom: 24,
          child: FloatingActionButton.extended(
            backgroundColor: AppColors.navyBlue,
            foregroundColor: Colors.white,
            icon: const Icon(Icons.add),
            label: const Text('New Product'),
            onPressed: () => context.go('/admin/products/create'),
          ),
        ),
      ],
    );
  }
}

/// Regenera las filas product_flat por idioma y los índices de precio de
/// todo el catálogo (tras activar locales nuevos o importar productos).
class _ReindexFlatsButton extends StatefulWidget {
  const _ReindexFlatsButton({this.compact = false});
  final bool compact;

  @override
  State<_ReindexFlatsButton> createState() => _ReindexFlatsButtonState();
}

class _ReindexFlatsButtonState extends State<_ReindexFlatsButton> {
  bool _running = false;

  Future<void> _run() async {
    if (_running) return;
    setState(() => _running = true);
    final admin = context.read<AdminProvider>();
    try {
      final result = await admin.post('admin/catalog/products/reindex-flats');
      if (mounted) {
        final message = result is Map && result['message'] != null
            ? '${result['message']}'
            : 'Products have been reindexed.';
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(message)));
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
        );
      }
    } finally {
      if (mounted) setState(() => _running = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final icon = _running
        ? const SizedBox(
            width: 14,
            height: 14,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              color: AppColors.navyBlue,
            ),
          )
        : const Icon(Icons.refresh, size: 16);
    if (widget.compact) {
      return IconButton(
        onPressed: _running ? null : _run,
        tooltip: 'Reindex translations',
        icon: icon,
      );
    }
    return OutlinedButton.icon(
      onPressed: _running ? null : _run,
      icon: icon,
      label: const Text(
        'Reindex translations',
        style: TextStyle(fontFamily: 'Poppins', fontSize: 12),
      ),
    );
  }
}

class AdminOrdersPage extends StatelessWidget {
  const AdminOrdersPage({super.key});

  @override
  Widget build(BuildContext context) {
    return EzDataGrid(
      gridPath: 'admin/sales/orders',
      title: 'Orders',
      detailBuilder: (data) => OrderDetailView(order: data),
    );
  }
}

/// Detalle de pedido: cabecera, acciones, comentarios e impresión de facturas.
class OrderDetailView extends StatefulWidget {
  const OrderDetailView({super.key, required this.order});
  final Map<String, dynamic> order;

  @override
  State<OrderDetailView> createState() => _OrderDetailViewState();
}

class _OrderDetailViewState extends State<OrderDetailView> {
  final TextEditingController _comment = TextEditingController();
  bool _notify = true;
  bool _sending = false;
  List<Map<String, dynamic>> _comments = [];

  @override
  void initState() {
    super.initState();
    _comments = decodeList(widget.order['comments']);
  }

  Future<void> _addComment() async {
    if (_comment.text.trim().isEmpty || _sending) return;
    setState(() => _sending = true);
    final admin = context.read<AdminProvider>();
    try {
      await admin.post(
        'admin/sales/orders/comment/${widget.order['id']}',
        body: {'comment': _comment.text.trim(), 'customer_notified': _notify},
      );
      _comment.clear();
      final data = await admin.get(
        'admin/sales/orders/view/${widget.order['id']}',
      );
      if (data is Map && mounted) {
        setState(() => _comments = decodeList(data['comments']));
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('The comment has been added successfully.'),
          ),
        );
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
        );
      }
    } finally {
      if (mounted) setState(() => _sending = false);
    }
  }

  /// Descarga el HTML imprimible de la factura y lo abre en una pestaña.
  Future<void> _printInvoice(int invoiceId) async {
    final token = ApiClient.instance.adminToken;
    try {
      final response = await http.get(
        Uri.parse(
          '${ApiClient.baseUrl}/api/admin/sales/invoices/print/$invoiceId',
        ),
        headers: {'Authorization': 'Bearer $token'},
      );
      if (response.statusCode != 200 || !mounted) return;
      openHtmlInNewTab(response.bodyBytes);
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final data = widget.order;
    final items = (data['items'] ?? const []) as List;
    final invoices = decodeList(data['invoices']);
    final total = data['grand_total'];
    final totalLabel = total is num
        ? '€${total.toStringAsFixed(2)}'
        : '${total ?? ''}';
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Wrap(
            spacing: 16,
            children: [
              if ((data['can_invoice'] ?? false) == true)
                ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppColors.navyBlue,
                    foregroundColor: Colors.white,
                  ),
                  onPressed: () async {
                    final admin = context.read<AdminProvider>();
                    await admin.post(
                      'admin/sales/invoices/create/${data['id']}',
                      body: {},
                    );
                  },
                  child: const Text('Create Invoice'),
                ),
              if ((data['can_cancel'] ?? false) == true)
                OutlinedButton(
                  style: OutlinedButton.styleFrom(
                    side: const BorderSide(color: AppColors.red600),
                  ),
                  onPressed: () async {
                    final admin = context.read<AdminProvider>();
                    await admin.post(
                      'admin/sales/orders/${data['id']}/cancel',
                      body: {},
                    );
                  },
                  child: const Text(
                    'Cancel Order',
                    style: TextStyle(color: AppColors.red600),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 12),
          Text(
            'Status: ${data['status']} · Total: $totalLabel',
            style: const TextStyle(fontFamily: 'Poppins', fontSize: 14),
          ),
          Text(
            'Customer: ${data['customer_email']}',
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: 14,
              color: AppColors.zinc500,
            ),
          ),
          const Divider(height: 24),
          ...items.map((item) {
            final row = Map<String, dynamic>.from(item as Map);
            final itemTotal = row['total'] ?? row['base_total'];
            return Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Row(
                children: [
                  Expanded(
                    child: Text('${row['name']} ×${row['qty_ordered']}'),
                  ),
                  Text(
                    itemTotal is num
                        ? '€${itemTotal.toStringAsFixed(2)}'
                        : '${itemTotal ?? ''}',
                  ),
                ],
              ),
            );
          }),
          const Divider(height: 24),
          const Text(
            'Invoices',
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: 14,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 8),
          if (invoices.isEmpty)
            const Text(
              'No invoices yet.',
              style: TextStyle(
                fontFamily: 'Poppins',
                fontSize: 12,
                color: AppColors.zinc500,
              ),
            )
          else
            Wrap(
              spacing: 8,
              children: [
                for (final invoice in invoices)
                  ActionChip(
                    label: Text(
                      '${invoice['increment_id'] ?? invoice['id']}',
                      style: const TextStyle(
                        fontFamily: 'Poppins',
                        fontSize: 12,
                      ),
                    ),
                    onPressed: () => _printInvoice(invoice['id'] as int),
                  ),
              ],
            ),
          const Divider(height: 24),
          const Text(
            'Comments',
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: 14,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 8),
          for (final comment in _comments)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text(
                '• ${comment['comment']}'
                '  (${(comment['customer_notified'] ?? 0) == 1 ? 'notified' : 'internal'}, ${comment['created_at']})',
                style: const TextStyle(fontFamily: 'Poppins', fontSize: 12),
              ),
            ),
          if (_comments.isEmpty)
            const Text(
              'No comments yet.',
              style: TextStyle(
                fontFamily: 'Poppins',
                fontSize: 12,
                color: AppColors.zinc500,
              ),
            ),
          const SizedBox(height: 12),
          TextField(
            controller: _comment,
            maxLines: 2,
            decoration: const InputDecoration(
              labelText: 'Add a comment',
              border: OutlineInputBorder(),
            ),
          ),
          const SizedBox(height: 8),
          LayoutBuilder(
            builder: (context, constraints) {
              final compact = constraints.maxWidth < 380;
              final notify = Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Switch(
                    value: _notify,
                    activeThumbColor: AppColors.navyBlue,
                    onChanged: (value) => setState(() => _notify = value),
                  ),
                  const Text(
                    'Notify customer',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13),
                  ),
                ],
              );
              final button = PrimaryButton(
                label: 'Add comment',
                height: 44,
                isLoading: _sending,
                onPressed: _sending ? null : _addComment,
              );
              if (compact) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [notify, const SizedBox(height: 8), button],
                );
              }
              return Row(
                children: [
                  notify,
                  const Spacer(),
                  button,
                ],
              );
            },
          ),
        ],
      ),
    );
  }
}

class AdminCustomersPage extends StatelessWidget {
  const AdminCustomersPage({super.key});

  @override
  Widget build(BuildContext context) {
    return EzDataGrid(
      gridPath: 'admin/customers',
      title: 'Customers',
      detailBuilder: (data) {
        final customer = Map<String, dynamic>.from(
          (data['customer'] ?? const {}) as Map,
        );
        final orders = decodeList(data['orders']);
        final reviews = decodeList(data['reviews']);
        final addresses = decodeList(data['addresses']);
        return SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                '${customer['first_name']} ${customer['last_name']}',
                style: const TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 16,
                  fontWeight: FontWeight.w500,
                ),
              ),
              Text(
                '${customer['email']} · group ${customer['customer_group_id']} · '
                '${(customer['subscribed_to_news_letter'] ?? 0) == 1 ? 'subscribed' : 'not subscribed'}',
                style: TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 12,
                  color: AppColors.zinc500,
                ),
              ),
              const Divider(height: 24),
              Text(
                'Orders (${orders.length})',
                style: const TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
              const SizedBox(height: 8),
              for (final order in orders)
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Text(
                    '${order['increment_id']} · ${order['status']} · €${order['base_grand_total']} · ${order['created_at']}',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 12),
                  ),
                ),
              const Divider(height: 24),
              Text(
                'Addresses (${addresses.length})',
                style: const TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
              const SizedBox(height: 8),
              for (final address in addresses)
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Text(
                    '${address['address1']}, ${address['city']} ${address['postcode']} (${address['country']})',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 12),
                  ),
                ),
              const Divider(height: 24),
              Text(
                'Reviews (${reviews.length}) · Wishlist: ${data['wishlist_count']} items',
                style: const TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                ),
              ),
              const SizedBox(height: 8),
              for (final review in reviews)
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Text(
                    '★${review['rating']} ${review['title']} (${review['status']})',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 12),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}

class AdminSettingsPage extends StatefulWidget {
  const AdminSettingsPage({super.key});

  @override
  State<AdminSettingsPage> createState() => _AdminSettingsPageState();
}

class _AdminSettingsPageState extends State<AdminSettingsPage> {
  List<Map<String, dynamic>> _channels = [];
  List<Map<String, dynamic>> _currencies = [];
  List<Map<String, dynamic>> _locales = [];
  List<Map<String, dynamic>> _taxCategories = [];
  List<Map<String, dynamic>> _taxRates = [];
  List<Map<String, dynamic>> _exchangeRates = [];
  List<Map<String, dynamic>> _inventorySources = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final channels = await admin.get('admin/settings/channels');
    final currencies = await admin.get('admin/settings/currencies');
    final locales = await admin.get('admin/settings/locales');
    final taxCategories = await admin.get('admin/settings/taxes/categories');
    final taxRates = await admin.get('admin/settings/taxes/rates');
    final exchangeRates = await admin.get('admin/settings/exchange-rates');
    final inventorySources = await admin.get(
      'admin/settings/inventory-sources',
    );
    if (mounted) {
      setState(() {
        _channels = decodeList(channels);
        _currencies = decodeList(currencies);
        _locales = decodeList(locales);
        _taxCategories = decodeList(taxCategories);
        _taxRates = decodeList(taxRates);
        _exchangeRates = decodeList(exchangeRates);
        _inventorySources = decodeList(inventorySources);
        _loading = false;
      });
    }
  }

  Future<void> _inventorySourceDialog([Map<String, dynamic>? source]) async {
    final code = TextEditingController(text: '${source?['code'] ?? ''}');
    final name = TextEditingController(text: '${source?['name'] ?? ''}');
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(
          source == null ? 'New inventory source' : 'Edit inventory source',
        ),
        content: SizedBox(
          width: 420,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: code,
                enabled: source == null,
                decoration: const InputDecoration(labelText: 'Code'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: name,
                decoration: const InputDecoration(labelText: 'Name'),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          PrimaryButton(
            label: 'Save',
            onPressed: () =>
                (code.text.trim().isEmpty || name.text.trim().isEmpty)
                ? null
                : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      source == null
          ? await admin.post(
              'admin/settings/inventory-sources/store',
              body: {'code': code.text.trim(), 'name': name.text.trim()},
            )
          : await admin.put(
              'admin/settings/inventory-sources/edit/${source['id']}',
              body: {'name': name.text.trim()},
            );
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
    );
  }

  /// Ids de locale/moneda usados por algún canal: no se pueden borrar.
  Set<int> get _localesInUse => {
    for (final channel in _channels) ...decodeIds(channel['locales']),
    for (final channel in _channels) ...[
      if (channel['default_locale_id'] is int)
        channel['default_locale_id'] as int,
    ],
  };

  Set<int> get _currenciesInUse => {
    for (final channel in _channels) ...decodeIds(channel['currencies']),
    for (final channel in _channels) ...[
      if (channel['base_currency_id'] is int)
        channel['base_currency_id'] as int,
    ],
  };

  Future<void> _delete(String label, String path) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Delete $label?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      await admin.delete(path);
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _currencyDialog([Map<String, dynamic>? currency]) async {
    final code = TextEditingController(text: '${currency?['code'] ?? ''}');
    final name = TextEditingController(text: '${currency?['name'] ?? ''}');
    final symbol = TextEditingController(text: '${currency?['symbol'] ?? ''}');
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(currency == null ? 'New currency' : 'Edit currency'),
        content: SizedBox(
          width: 420,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: code,
                decoration: const InputDecoration(labelText: 'Code (ISO 4217)'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: name,
                decoration: const InputDecoration(labelText: 'Name'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: symbol,
                decoration: const InputDecoration(labelText: 'Symbol'),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          PrimaryButton(
            label: 'Save',
            onPressed: () =>
                code.text.trim().isEmpty ? null : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    final body = {
      'code': code.text.trim(),
      'name': name.text.trim(),
      'symbol': symbol.text.trim(),
      'decimal': 2,
    };
    try {
      currency == null
          ? await admin.post('admin/settings/currencies', body: body)
          : await admin.put(
              'admin/settings/currencies/edit/${currency['id']}',
              body: body,
            );
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _localeDialog([Map<String, dynamic>? locale]) async {
    final code = TextEditingController(text: '${locale?['code'] ?? ''}');
    final name = TextEditingController(text: '${locale?['name'] ?? ''}');
    String direction = '${locale?['direction'] ?? 'ltr'}';
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(locale == null ? 'New locale' : 'Edit locale'),
          content: SizedBox(
            width: 420,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: code,
                  decoration: const InputDecoration(
                    labelText: 'Code (e.g. es)',
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: name,
                  decoration: const InputDecoration(labelText: 'Name'),
                ),
                const SizedBox(height: 12),
                DropdownButtonFormField<String>(
                  initialValue: direction,
                  decoration: const InputDecoration(labelText: 'Direction'),
                  items: const [
                    DropdownMenuItem(value: 'ltr', child: Text('LTR')),
                    DropdownMenuItem(value: 'rtl', child: Text('RTL')),
                  ],
                  onChanged: (value) =>
                      setDialogState(() => direction = value ?? 'ltr'),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Cancel'),
            ),
            PrimaryButton(
              label: 'Save',
              onPressed: () => code.text.trim().isEmpty
                  ? null
                  : Navigator.pop(context, true),
            ),
          ],
        ),
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    final body = {
      'code': code.text.trim(),
      'name': name.text.trim(),
      'direction': direction,
    };
    try {
      locale == null
          ? await admin.post('admin/settings/locales', body: body)
          : await admin.put(
              'admin/settings/locales/edit/${locale['id']}',
              body: body,
            );
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _channelDialog([Map<String, dynamic>? channel]) async {
    final code = TextEditingController(text: '${channel?['code'] ?? ''}');
    final name = TextEditingController(text: '${channel?['name'] ?? ''}');
    final hostname = TextEditingController(
      text: '${channel?['hostname'] ?? ''}',
    );
    final timezone = TextEditingController(
      text: '${channel?['timezone'] ?? ''}',
    );
    int? defaultLocaleId = channel?['default_locale_id'] as int?;
    int? baseCurrencyId = channel?['base_currency_id'] as int?;
    final selectedLocales = {...decodeIds(channel?['locales'])};
    final selectedCurrencies = {...decodeIds(channel?['currencies'])};
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(channel == null ? 'New channel' : 'Edit channel'),
          content: SingleChildScrollView(
            child: SizedBox(
              width: 480,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextField(
                    controller: code,
                    decoration: const InputDecoration(labelText: 'Code'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: name,
                    decoration: const InputDecoration(labelText: 'Name'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: hostname,
                    decoration: const InputDecoration(
                      labelText: 'Hostname (optional)',
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: timezone,
                    decoration: const InputDecoration(
                      labelText: 'Timezone (optional)',
                    ),
                  ),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<int>(
                    initialValue: defaultLocaleId,
                    decoration: const InputDecoration(
                      labelText: 'Default locale',
                    ),
                    items: _locales
                        .map(
                          (locale) => DropdownMenuItem(
                            value: locale['id'] as int,
                            child: Text('${locale['name'] ?? locale['code']}'),
                          ),
                        )
                        .toList(),
                    onChanged: (value) => setDialogState(() {
                      defaultLocaleId = value;
                      if (value != null) selectedLocales.add(value);
                    }),
                  ),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<int>(
                    initialValue: baseCurrencyId,
                    decoration: const InputDecoration(
                      labelText: 'Base currency',
                    ),
                    items: _currencies
                        .map(
                          (currency) => DropdownMenuItem(
                            value: currency['id'] as int,
                            child: Text(
                              '${currency['name'] ?? currency['code']}',
                            ),
                          ),
                        )
                        .toList(),
                    onChanged: (value) => setDialogState(() {
                      baseCurrencyId = value;
                      if (value != null) selectedCurrencies.add(value);
                    }),
                  ),
                  const SizedBox(height: 16),
                  const Text(
                    'Locales',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13),
                  ),
                  Wrap(
                    spacing: 8,
                    runSpacing: 4,
                    children: [
                      for (final locale in _locales)
                        FilterChip(
                          label: Text(
                            '${locale['code']}',
                            style: const TextStyle(
                              fontFamily: 'Poppins',
                              fontSize: 12,
                            ),
                          ),
                          selected: selectedLocales.contains(locale['id']),
                          onSelected: (selected) => setDialogState(
                            () => selected
                                ? selectedLocales.add(locale['id'] as int)
                                : selectedLocales.remove(locale['id']),
                          ),
                        ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'Currencies',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13),
                  ),
                  Wrap(
                    spacing: 8,
                    runSpacing: 4,
                    children: [
                      for (final currency in _currencies)
                        FilterChip(
                          label: Text(
                            '${currency['code']}',
                            style: const TextStyle(
                              fontFamily: 'Poppins',
                              fontSize: 12,
                            ),
                          ),
                          selected: selectedCurrencies.contains(currency['id']),
                          onSelected: (selected) => setDialogState(
                            () => selected
                                ? selectedCurrencies.add(currency['id'] as int)
                                : selectedCurrencies.remove(currency['id']),
                          ),
                        ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Cancel'),
            ),
            PrimaryButton(
              label: 'Save',
              onPressed: () =>
                  (code.text.trim().isEmpty ||
                      defaultLocaleId == null ||
                      baseCurrencyId == null)
                  ? null
                  : Navigator.pop(context, true),
            ),
          ],
        ),
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    final body = {
      'code': code.text.trim(),
      'name': name.text.trim(),
      'hostname': hostname.text.trim(),
      'timezone': timezone.text.trim(),
      'default_locale_id': defaultLocaleId,
      'base_currency_id': baseCurrencyId,
      'locales': selectedLocales.toList(),
      'currencies': selectedCurrencies.toList(),
    };
    try {
      channel == null
          ? await admin.post('admin/settings/channels', body: body)
          : await admin.put(
              'admin/settings/channels/edit/${channel['id']}',
              body: body,
            );
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _taxRateDialog([Map<String, dynamic>? rate]) async {
    final identifier = TextEditingController(
      text: '${rate?['identifier'] ?? ''}',
    );
    final country = TextEditingController(text: '${rate?['country'] ?? 'ES'}');
    final state = TextEditingController(text: '${rate?['state'] ?? '*'}');
    final zipCode = TextEditingController(text: '${rate?['zip_code'] ?? '*'}');
    final zipFrom = TextEditingController(text: '${rate?['zip_from'] ?? ''}');
    final zipTo = TextEditingController(text: '${rate?['zip_to'] ?? ''}');
    final percent = TextEditingController(text: '${rate?['tax_rate'] ?? 21}');
    bool isZip = (rate?['is_zip'] ?? 0) == 1;
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(rate == null ? 'New tax rate' : 'Edit tax rate'),
          content: SingleChildScrollView(
            child: SizedBox(
              width: 440,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    controller: identifier,
                    decoration: const InputDecoration(
                      labelText: 'Identifier (e.g. IVA-21)',
                    ),
                  ),
                  const SizedBox(height: 12),
                  Row(
                    children: [
                      Expanded(
                        child: TextField(
                          controller: country,
                          decoration: const InputDecoration(
                            labelText: 'Country (ISO)',
                          ),
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: TextField(
                          controller: state,
                          decoration: const InputDecoration(
                            labelText: 'State (* = all)',
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  SwitchListTile(
                    dense: true,
                    contentPadding: EdgeInsets.zero,
                    title: const Text(
                      'Zip code range',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 13),
                    ),
                    value: isZip,
                    onChanged: (value) => setDialogState(() => isZip = value),
                  ),
                  if (isZip)
                    Row(
                      children: [
                        Expanded(
                          child: TextField(
                            controller: zipFrom,
                            decoration: const InputDecoration(
                              labelText: 'Zip from',
                            ),
                          ),
                        ),
                        const SizedBox(width: 12),
                        Expanded(
                          child: TextField(
                            controller: zipTo,
                            decoration: const InputDecoration(
                              labelText: 'Zip to',
                            ),
                          ),
                        ),
                      ],
                    )
                  else
                    TextField(
                      controller: zipCode,
                      decoration: const InputDecoration(
                        labelText: 'Zip code (* = all)',
                      ),
                    ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: percent,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(labelText: 'Rate (%)'),
                  ),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Cancel'),
            ),
            PrimaryButton(
              label: 'Save',
              onPressed: () => identifier.text.trim().isEmpty
                  ? null
                  : Navigator.pop(context, true),
            ),
          ],
        ),
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    final body = {
      'identifier': identifier.text.trim(),
      'country': country.text.trim().toUpperCase(),
      'state': state.text.trim(),
      'is_zip': isZip ? 1 : 0,
      if (isZip) ...{
        'zip_from': zipFrom.text.trim(),
        'zip_to': zipTo.text.trim(),
        'zip_code': '',
      } else ...{
        'zip_from': '',
        'zip_to': '',
        'zip_code': zipCode.text.trim(),
      },
      'tax_rate': num.tryParse(percent.text) ?? 0,
    };
    try {
      rate == null
          ? await admin.post('admin/settings/taxes/rates', body: body)
          : await admin.put(
              'admin/settings/taxes/rates/edit/${rate['id']}',
              body: body,
            );
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _taxCategoryDialog([Map<String, dynamic>? category]) async {
    final code = TextEditingController(text: '${category?['code'] ?? ''}');
    final name = TextEditingController(text: '${category?['name'] ?? ''}');
    final description = TextEditingController(
      text: '${category?['description'] ?? ''}',
    );
    final selected = {
      ...decodeList(category?['taxrates']).map((rate) => rate['id'] as int),
    };
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(
            category == null ? 'New tax category' : 'Edit tax category',
          ),
          content: SingleChildScrollView(
            child: SizedBox(
              width: 440,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextField(
                    controller: code,
                    decoration: const InputDecoration(labelText: 'Code'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: name,
                    decoration: const InputDecoration(labelText: 'Name'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: description,
                    decoration: const InputDecoration(labelText: 'Description'),
                  ),
                  const SizedBox(height: 16),
                  const Text(
                    'Tax rates',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13),
                  ),
                  Wrap(
                    spacing: 8,
                    runSpacing: 4,
                    children: [
                      for (final rate in _taxRates)
                        FilterChip(
                          label: Text(
                            '${rate['identifier']}',
                            style: const TextStyle(
                              fontFamily: 'Poppins',
                              fontSize: 12,
                            ),
                          ),
                          selected: selected.contains(rate['id']),
                          onSelected: (value) => setDialogState(
                            () => value
                                ? selected.add(rate['id'] as int)
                                : selected.remove(rate['id']),
                          ),
                        ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Cancel'),
            ),
            PrimaryButton(
              label: 'Save',
              onPressed: () => code.text.trim().isEmpty
                  ? null
                  : Navigator.pop(context, true),
            ),
          ],
        ),
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    final body = {
      'code': code.text.trim(),
      'name': name.text.trim(),
      'description': description.text.trim(),
      'tax_rates': selected.toList(),
    };
    try {
      category == null
          ? await admin.post('admin/settings/taxes/categories', body: body)
          : await admin.put(
              'admin/settings/taxes/categories/edit/${category['id']}',
              body: body,
            );
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _exchangeRateDialog([Map<String, dynamic>? exchangeRate]) async {
    int? target = exchangeRate?['target_currency'] as int?;
    final rate = TextEditingController(text: '${exchangeRate?['rate'] ?? 1}');
    final availableCurrencies = exchangeRate == null
        ? _currencies
              .where(
                (currency) => !_exchangeRates.any(
                  (row) => row['target_currency'] == currency['id'],
                ),
              )
              .toList()
        : _currencies;
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(
          exchangeRate == null ? 'New exchange rate' : 'Edit exchange rate',
        ),
        content: SizedBox(
          width: 420,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (exchangeRate == null)
                DropdownButtonFormField<int>(
                  initialValue: target,
                  decoration: const InputDecoration(
                    labelText: 'Target currency',
                  ),
                  items: availableCurrencies
                      .map(
                        (currency) => DropdownMenuItem(
                          value: currency['id'] as int,
                          child: Text(
                            '${currency['code']} (${currency['name']})',
                          ),
                        ),
                      )
                      .toList(),
                  onChanged: (value) => target = value,
                )
              else
                ListTile(
                  dense: true,
                  contentPadding: EdgeInsets.zero,
                  title: Text(
                    '${exchangeRate['code']} (${exchangeRate['name']})',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14),
                  ),
                ),
              const SizedBox(height: 12),
              TextField(
                controller: rate,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'Rate (per 1 base currency)',
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          PrimaryButton(
            label: 'Save',
            onPressed: () => (exchangeRate == null && target == null)
                ? null
                : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      exchangeRate == null
          ? await admin.post(
              'admin/settings/exchange-rates',
              body: {
                'target_currency': target,
                'rate': num.tryParse(rate.text) ?? 1,
              },
            )
          : await admin.put(
              'admin/settings/exchange-rates/edit/${exchangeRate['id']}',
              body: {'rate': num.tryParse(rate.text) ?? 1},
            );
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(
        child: CircularProgressIndicator(color: AppColors.navyBlue),
      );
    }
    final phone = MediaQuery.of(context).size.width < 640;
    return SingleChildScrollView(
      padding: EdgeInsets.all(phone ? 16 : 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Settings',
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: phone ? 20 : 24,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 24),
          _Section(
            title: 'Channels',
            actions: [
              TextButton.icon(
                onPressed: () => _channelDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text(
                  'New Channel',
                  style: TextStyle(fontFamily: 'Poppins'),
                ),
              ),
            ],
            children: [
              for (final channel in _channels)
                ListTile(
                  title: Text(
                    '${channel['name'] ?? channel['code']}',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14),
                  ),
                  subtitle: Text(
                    'code: ${channel['code']} · theme: ${channel['theme']} · '
                    'locales: ${decodeIds(channel['locales']).length} · '
                    'currencies: ${decodeIds(channel['currencies']).length}',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 12,
                      color: AppColors.zinc500,
                    ),
                  ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        tooltip: 'Edit',
                        onPressed: () => _channelDialog(channel),
                      ),
                      if (channel['id'] != 1)
                        IconButton(
                          icon: const Icon(
                            Icons.delete_outline,
                            size: 18,
                            color: AppColors.red600,
                          ),
                          tooltip: 'Delete',
                          onPressed: () => _delete(
                            'channel',
                            'admin/settings/channels/${channel['id']}',
                          ),
                        ),
                    ],
                  ),
                ),
            ],
          ),
          _Section(
            title: 'Currencies',
            actions: [
              TextButton.icon(
                onPressed: () => _currencyDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text(
                  'New Currency',
                  style: TextStyle(fontFamily: 'Poppins'),
                ),
              ),
            ],
            children: [
              for (final currency in _currencies)
                ListTile(
                  title: Text(
                    '${currency['name']} (${currency['symbol']})',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14),
                  ),
                  subtitle: Text(
                    '${currency['code']}',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 12,
                      color: AppColors.zinc500,
                    ),
                  ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        tooltip: 'Edit',
                        onPressed: () => _currencyDialog(currency),
                      ),
                      if (!_currenciesInUse.contains(currency['id']))
                        IconButton(
                          icon: const Icon(
                            Icons.delete_outline,
                            size: 18,
                            color: AppColors.red600,
                          ),
                          tooltip: 'Delete',
                          onPressed: () => _delete(
                            'currency',
                            'admin/settings/currencies/${currency['id']}',
                          ),
                        ),
                    ],
                  ),
                ),
            ],
          ),
          _Section(
            title: 'Locales',
            actions: [
              TextButton.icon(
                onPressed: () => _localeDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text(
                  'New Locale',
                  style: TextStyle(fontFamily: 'Poppins'),
                ),
              ),
            ],
            children: [
              for (final locale in _locales)
                ListTile(
                  title: Text(
                    '${locale['name']}',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14),
                  ),
                  subtitle: Text(
                    '${locale['code']} · ${locale['direction']}',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 12,
                      color: AppColors.zinc500,
                    ),
                  ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        tooltip: 'Edit',
                        onPressed: () => _localeDialog(locale),
                      ),
                      if (!_localesInUse.contains(locale['id']))
                        IconButton(
                          icon: const Icon(
                            Icons.delete_outline,
                            size: 18,
                            color: AppColors.red600,
                          ),
                          tooltip: 'Delete',
                          onPressed: () => _delete(
                            'locale',
                            'admin/settings/locales/${locale['id']}',
                          ),
                        ),
                    ],
                  ),
                ),
            ],
          ),
          _Section(
            title: 'Tax categories',
            actions: [
              TextButton.icon(
                onPressed: () => _taxCategoryDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text(
                  'New Category',
                  style: TextStyle(fontFamily: 'Poppins'),
                ),
              ),
            ],
            children: [
              for (final category in _taxCategories)
                ListTile(
                  title: Text(
                    '${category['name'] ?? category['code']}',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14),
                  ),
                  subtitle: Text(
                    'code: ${category['code']} · rates: ${decodeList(category['taxrates']).map((rate) => rate['identifier']).join(', ')}',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 12,
                      color: AppColors.zinc500,
                    ),
                  ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        tooltip: 'Edit',
                        onPressed: () => _taxCategoryDialog(category),
                      ),
                      IconButton(
                        icon: const Icon(
                          Icons.delete_outline,
                          size: 18,
                          color: AppColors.red600,
                        ),
                        tooltip: 'Delete',
                        onPressed: () => _delete(
                          'tax category',
                          'admin/settings/taxes/categories/${category['id']}',
                        ),
                      ),
                    ],
                  ),
                ),
            ],
          ),
          _Section(
            title: 'Tax rates',
            actions: [
              TextButton.icon(
                onPressed: () => _taxRateDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text(
                  'New Rate',
                  style: TextStyle(fontFamily: 'Poppins'),
                ),
              ),
            ],
            children: [
              for (final rate in _taxRates)
                ListTile(
                  title: Text(
                    '${rate['identifier']} — ${rate['tax_rate']}%',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14),
                  ),
                  subtitle: Text(
                    'country: ${rate['country']} · state: ${rate['state'] ?? '*'} · '
                    'zip: ${rate['is_zip'] == 1 ? '${rate['zip_from']}-${rate['zip_to']}' : rate['zip_code']}',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 12,
                      color: AppColors.zinc500,
                    ),
                  ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        tooltip: 'Edit',
                        onPressed: () => _taxRateDialog(rate),
                      ),
                      IconButton(
                        icon: const Icon(
                          Icons.delete_outline,
                          size: 18,
                          color: AppColors.red600,
                        ),
                        tooltip: 'Delete',
                        onPressed: () => _delete(
                          'tax rate',
                          'admin/settings/taxes/rates/${rate['id']}',
                        ),
                      ),
                    ],
                  ),
                ),
            ],
          ),
          _Section(
            title: 'Inventory sources',
            actions: [
              TextButton.icon(
                onPressed: () => _inventorySourceDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text(
                  'New Source',
                  style: TextStyle(fontFamily: 'Poppins'),
                ),
              ),
            ],
            children: [
              for (final source in _inventorySources)
                ListTile(
                  title: Text(
                    '${source['name'] ?? source['code']}',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 14,
                      color: (source['status'] ?? 1) == 1
                          ? Colors.black
                          : AppColors.zinc500,
                    ),
                  ),
                  subtitle: Text(
                    'code: ${source['code']}',
                    style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 12,
                      color: AppColors.zinc500,
                    ),
                  ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        onPressed: () => _inventorySourceDialog(source),
                      ),
                      if (source['id'] != 1)
                        IconButton(
                          icon: const Icon(
                            Icons.delete_outline,
                            size: 18,
                            color: AppColors.red600,
                          ),
                          onPressed: () => _delete(
                            'inventory source',
                            'admin/settings/inventory-sources/${source['id']}',
                          ),
                        ),
                    ],
                  ),
                ),
            ],
          ),
          _Section(
            title: 'Exchange rates',
            actions: [
              TextButton.icon(
                onPressed: () => _exchangeRateDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text(
                  'New Rate',
                  style: TextStyle(fontFamily: 'Poppins'),
                ),
              ),
            ],
            children: [
              if (_exchangeRates.isEmpty)
                const Text(
                  'No exchange rates yet.',
                  style: TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 13,
                    color: AppColors.zinc500,
                  ),
                )
              else
                for (final exchangeRate in _exchangeRates)
                  ListTile(
                    title: Text(
                      '1 EUR = ${exchangeRate['rate']} ${exchangeRate['code']}',
                      style: const TextStyle(
                        fontFamily: 'Poppins',
                        fontSize: 14,
                      ),
                    ),
                    subtitle: Text(
                      '${exchangeRate['name']}',
                      style: TextStyle(
                        fontFamily: 'Poppins',
                        fontSize: 12,
                        color: AppColors.zinc500,
                      ),
                    ),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          icon: const Icon(Icons.edit_outlined, size: 18),
                          tooltip: 'Edit',
                          onPressed: () => _exchangeRateDialog(exchangeRate),
                        ),
                        IconButton(
                          icon: const Icon(
                            Icons.delete_outline,
                            size: 18,
                            color: AppColors.red600,
                          ),
                          tooltip: 'Delete',
                          onPressed: () => _delete(
                            'exchange rate',
                            'admin/settings/exchange-rates/${exchangeRate['id']}',
                          ),
                        ),
                      ],
                    ),
                  ),
            ],
          ),
        ],
      ),
    );
  }
}

class _Section extends StatelessWidget {
  const _Section({required this.title, required this.children, this.actions});
  final String title;
  final List<Widget> children;
  final List<Widget>? actions;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  title,
                  style: const TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 16,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ),
              ...?actions,
            ],
          ),
          ...children,
        ],
      ),
    );
  }
}
