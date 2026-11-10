import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:http/http.dart' as http;
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
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
                  const Text('EzCommerce Admin',
                      style: TextStyle(
                          fontFamily: 'DM Serif Display', fontSize: 28, color: AppColors.navyBlue)),
                  const SizedBox(height: 4),
                  Text('Sign in to your admin account.',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.zinc500)),
                  const SizedBox(height: 24),
                  if (admin.error != null)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: Text(admin.error!,
                          style: const TextStyle(color: AppColors.red600, fontSize: 13)),
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

  @override
  Widget build(BuildContext context) {
    final admin = context.watch<AdminProvider>();
    if (!admin.isLoggedIn) {
      // Renderiza el login como contenido; el router redirige desde pantallas.
      return const AdminLoginPage();
    }
    final isNarrow = MediaQuery.of(context).size.width < 900;
    final items = [
      ('Dashboard', Icons.dashboard_outlined, '/admin'),
      ('Products', Icons.inventory_2_outlined, '/admin/products'),
      ('Orders', Icons.receipt_long_outlined, '/admin/orders'),
      ('Customers', Icons.people_outline, '/admin/customers'),
      ('Settings', Icons.settings_outlined, '/admin/settings'),
    ];

    return Scaffold(
      backgroundColor: AppColors.zinc50,
      body: Row(
        children: [
          if (!isNarrow)
            Container(
              width: 240,
              color: AppColors.navyBlue,
              child: Column(
                children: [
                  const SizedBox(height: 24),
                  const Text('EzCommerce',
                      style: TextStyle(
                          fontFamily: 'DM Serif Display', fontSize: 22, color: Colors.white)),
                  Text('Admin Panel',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: Colors.white70)),
                  const SizedBox(height: 24),
                  ...items.map((entry) {
                    final selected = GoRouterState.of(context).uri.path == entry.$3;
                    return ListTile(
                      leading: Icon(entry.$2,
                          size: 20, color: selected ? Colors.white : Colors.white70),
                      title: Text(entry.$1,
                          style: TextStyle(
                              fontFamily: 'Poppins',
                              fontSize: 14,
                              color: selected ? Colors.white : Colors.white70,
                              fontWeight: selected ? FontWeight.w600 : FontWeight.w400)),
                      selected: selected,
                      onTap: () => context.go(entry.$3),
                    );
                  }),
                  const Spacer(),
                  ListTile(
                    leading: const Icon(Icons.logout, size: 20, color: Colors.white70),
                    title: const Text('Logout',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: Colors.white70)),
                    onTap: () async {
                      await admin.logout();
                      if (context.mounted) context.go('/admin/login');
                    },
                  ),
                  const SizedBox(height: 16),
                ],
              ),
            ),
          Expanded(
            child: Column(
              children: [
                Container(
                  color: Colors.white,
                  padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                  child: Row(
                    children: [
                      if (isNarrow)
                        PopupMenuButton<String>(
                          icon: const Icon(Icons.menu),
                          itemBuilder: (context) =>
                              items.map((e) => PopupMenuItem(value: e.$3, child: Text(e.$1))).toList(),
                          onSelected: (route) => context.go(route),
                        ),
                      const Spacer(),
                      Text(
                        admin.me?['email'] ?? '',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc600),
                      ),
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
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final admin = context.read<AdminProvider>();
    final overAll = await admin.get('admin/dashboard/stats', query: {'type': 'over-all'});
    final today = await admin.get('admin/dashboard/stats', query: {'type': 'today'});
    if (mounted) {
      setState(() {
        _overAll = overAll is Map ? Map<String, dynamic>.from(overAll) : null;
        _today = today is Map ? Map<String, dynamic>.from(today) : null;
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    final stats = _overAll ?? const {};
    Map<String, dynamic> metric(String key) =>
        Map<String, dynamic>.from((stats[key] ?? const {}) as Map);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Dashboard',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
          const SizedBox(height: 24),
          Wrap(
            spacing: 16,
            runSpacing: 16,
            children: [
              _MetricCard(
                title: 'Total Sales',
                value: (metric('total_sales')['current'] ?? 0).toString(),
                progress: ((metric('total_sales')['progress'] ?? 0) as num).toDouble(),
              ),
              _MetricCard(
                title: 'Total Orders',
                value: '${metric('total_orders')['current'] ?? 0}',
                progress: ((metric('total_orders')['progress'] ?? 0) as num).toDouble(),
              ),
              _MetricCard(
                title: 'New Customers',
                value: '${metric('total_customers')['current'] ?? 0}',
                progress: ((metric('total_customers')['progress'] ?? 0) as num).toDouble(),
              ),
              _MetricCard(
                title: 'Average Sales',
                value: (metric('avg_sales')['current'] ?? 0).toString(),
                progress: ((metric('avg_sales')['progress'] ?? 0) as num).toDouble(),
              ),
            ],
          ),
          const SizedBox(height: 32),
          // Recent orders de hoy
          if (_today != null && _today!['recent_orders'] is List)
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
                  Text('Today\'s Orders (${_today!['orders_today'] ?? 0})',
                      style: const TextStyle(
                          fontFamily: 'Poppins', fontSize: 18, fontWeight: FontWeight.w500)),
                  const SizedBox(height: 16),
                  ...((_today!['recent_orders'] as List?) ?? []).map((order) {
                    final row = Map<String, dynamic>.from(order as Map);
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: Row(
                        children: [
                          Text('#${row['increment_id']}',
                              style: const TextStyle(
                                  fontFamily: 'Poppins',
                                  fontWeight: FontWeight.w600,
                                  color: AppColors.navyBlue)),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Text('${row['customer_email'] ?? ''}',
                                style: TextStyle(
                                    fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500)),
                          ),
                          Text('${row['status']}',
                              style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                          const SizedBox(width: 16),
                          Text('${row['base_grand_total']}',
                              style: const TextStyle(
                                  fontFamily: 'Poppins', fontSize: 13, fontWeight: FontWeight.w600)),
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
  const _MetricCard({required this.title, required this.value, required this.progress});
  final String title;
  final String value;
  final double progress;

  @override
  Widget build(BuildContext context) {
    final positive = progress >= 0;
    return Container(
      width: 240,
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(title, style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500)),
          const SizedBox(height: 8),
          Text(value,
              style: const TextStyle(
                  fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w600)),
          const SizedBox(height: 8),
          Row(
            children: [
              Icon(
                positive ? Icons.trending_up : Icons.trending_down,
                size: 16,
                color: positive ? AppColors.green600 : AppColors.red600,
              ),
              const SizedBox(width: 4),
              Text(
                '${positive ? '+' : ''}${progress.toStringAsFixed(1)}% vs previous period',
                style: TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 12,
                    color: positive ? AppColors.green600 : AppColors.red600),
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
  });

  final String gridPath;
  final String title;
  final Widget Function(Map<String, dynamic> row)? detailBuilder;
  final List<Widget> Function(Map<String, dynamic> row)? extraActions;

  @override
  State<EzDataGrid> createState() => _EzDataGridState();
}

class _EzDataGridState extends State<EzDataGrid> {
  Map<String, dynamic>? _grid;
  bool _loading = true;
  String _search = '';
  int _page = 1;

  @override
  void initState() {
    super.initState();
    _load();
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
    final uri = Uri.parse('${ApiClient.baseUrl}/api/${widget.gridPath}?format=csv');
    final response = await http.post(
      uri,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': 'Bearer $token',
      },
      body: '{"filters": ${_search.isNotEmpty ? '{"all": "${_search}"}' : '{}'}}',
    );
    if (response.statusCode != 200 || !mounted) return;
    downloadFile('${widget.gridPath.split('/').last}.csv', response.bodyBytes);
    ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Export CSV downloaded.')));
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

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(widget.title,
                    style: const TextStyle(
                        fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
              ),
              SizedBox(
                width: 280,
                child: TextField(
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
                ),
              ),
              const SizedBox(width: 12),
              TextButton.icon(
                onPressed: _exportCsv,
                icon: const Icon(Icons.download_outlined, size: 18),
                label: const Text('Export CSV',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
              ),
            ],
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
                    child: Center(child: CircularProgressIndicator(color: AppColors.navyBlue)))
                : SizedBox(
                    width: double.infinity,
                    child: DataTable(
                      headingRowColor: WidgetStateProperty.all(AppColors.zinc100),
                      headingTextStyle: const TextStyle(
                          fontFamily: 'Poppins', fontSize: 13, fontWeight: FontWeight.w600),
                      dataTextStyle: const TextStyle(fontFamily: 'Poppins', fontSize: 13),
                      columns: columns
                          .where((column) => column['index'] != '__primary')
                          .map((column) => DataColumn(
                                label: Text('${column['label']}'),
                                onSort: (_, __) {},
                              ))
                          .toList()
                        ..add(const DataColumn(label: Text(''))),
                      rows: records.map((row) {
                        return DataRow(
                          cells: [
                            ...columns
                                .where((column) => column['index'] != '__primary')
                                .map((column) => DataCell(
                                      Text('${row[column['index']] ?? ''}'),
                                    )),
                            DataCell(
                              Row(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  if (widget.detailBuilder != null)
                                    IconButton(
                                      icon: const Icon(Icons.visibility_outlined, size: 18),
                                      onPressed: () => _openDetail(row),
                                    ),
                                  ...(widget.extraActions?.call(row) ?? const []),
                                ],
                              ),
                            ),
                          ],
                        );
                      }).toList(),
                    ),
                  ),
          ),
          const SizedBox(height: 16),
          // Paginación
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Text('$total records',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500)),
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
              Text('$_page / $lastPage',
                  style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
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
        title: Text('Detail #$id'),
        content: SizedBox(
          width: 560,
          child: widget.detailBuilder?.call(Map<String, dynamic>.from(data)) ??
              Text('$data'),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context), child: const Text('Close')),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Pantallas de datos
// ---------------------------------------------------------------------------

class AdminProductsPage extends StatelessWidget {
  const AdminProductsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return EzDataGrid(
      gridPath: 'admin/catalog/products',
      title: 'Products',
      extraActions: (row) => [
        IconButton(
          icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
          onPressed: () async {
            final admin = context.read<AdminProvider>();
            final confirmed = await showDialog<bool>(
              context: context,
              builder: (context) => AlertDialog(
                title: const Text('Delete product?'),
                actions: [
                  TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
                  TextButton(onPressed: () => Navigator.pop(context, true), child: const Text('Delete')),
                ],
              ),
            );
            if (confirmed == true) {
              await admin.delete('admin/catalog/products/${row['id']}');
            }
          },
        ),
      ],
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
      detailBuilder: (data) {
        final items = (data['items'] ?? const []) as List;
        return SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Wrap(
                spacing: 16,
                children: [
                  if ((data['can_invoice'] ?? false) == true)
                    ElevatedButton(
                      style: ElevatedButton.styleFrom(backgroundColor: AppColors.navyBlue),
                      onPressed: () async {
                        final admin = context.read<AdminProvider>();
                        await admin.post('admin/sales/invoices/create/${data['id']}', body: {});
                      },
                      child: const Text('Create Invoice'),
                    ),
                  if ((data['can_cancel'] ?? false) == true)
                    OutlinedButton(
                      style: OutlinedButton.styleFrom(
                          side: const BorderSide(color: AppColors.red600)),
                      onPressed: () async {
                        final admin = context.read<AdminProvider>();
                        await admin.post('admin/sales/orders/${data['id']}/cancel', body: {});
                      },
                      child: const Text('Cancel Order',
                          style: TextStyle(color: AppColors.red600)),
                    ),
                ],
              ),
              const SizedBox(height: 12),
              Text('Status: ${data['status']} · Total: ${data['grand_total']}',
                  style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
              Text('Customer: ${data['customer_email']}',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.zinc500)),
              const Divider(height: 24),
              ...items.map((item) {
                final row = Map<String, dynamic>.from(item as Map);
                return Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Row(
                    children: [
                      Expanded(child: Text('${row['name']} ×${row['qty_ordered']}')),
                      Text('${row['base_total']}'),
                    ],
                  ),
                );
              }),
            ],
          ),
        );
      },
    );
  }
}

class AdminCustomersPage extends StatelessWidget {
  const AdminCustomersPage({super.key});

  @override
  Widget build(BuildContext context) {
    return EzDataGrid(gridPath: 'admin/customers', title: 'Customers');
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
    if (mounted) {
      setState(() {
        _channels = decodeList(channels);
        _currencies = decodeList(currencies);
        _locales = decodeList(locales);
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Settings',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
          const SizedBox(height: 24),
          _Section(
            title: 'Channels',
            children: _channels
                .map((channel) => ListTile(
                      title: Text('${channel['name'] ?? channel['code']}',
                          style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                      subtitle: Text('code: ${channel['code']} · theme: ${channel['theme']}',
                          style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                    ))
                .toList(),
          ),
          _Section(
            title: 'Currencies',
            children: _currencies
                .map((currency) => ListTile(
                      title: Text('${currency['name']} (${currency['symbol']})',
                          style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                      subtitle: Text('${currency['code']}',
                          style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                    ))
                .toList(),
          ),
          _Section(
            title: 'Locales',
            children: _locales
                .map((locale) => ListTile(
                      title: Text('${locale['name']}',
                          style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                      subtitle: Text('${locale['code']} · ${locale['direction']}',
                          style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                    ))
                .toList(),
          ),
        ],
      ),
    );
  }
}

class _Section extends StatelessWidget {
  const _Section({required this.title, required this.children});
  final String title;
  final List<Widget> children;

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
          Text(title,
              style: const TextStyle(
                  fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
          ...children,
        ],
      ),
    );
  }
}
