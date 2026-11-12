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
      ('Categories', Icons.category_outlined, '/admin/categories'),
      ('Attributes', Icons.label_outline, '/admin/attributes'),
      ('Cart Rules', Icons.local_offer_outlined, '/admin/cart-rules'),
      ('Catalog Rules', Icons.sell_outlined, '/admin/catalog-rules'),
      ('CMS', Icons.description_outlined, '/admin/cms'),
      ('Appearance', Icons.palette_outlined, '/admin/appearance'),
      ('Orders', Icons.receipt_long_outlined, '/admin/orders'),
      ('Invoices', Icons.request_quote_outlined, '/admin/invoices'),
      ('Shipments', Icons.local_shipping_outlined, '/admin/shipments'),
      ('Refunds', Icons.assignment_return_outlined, '/admin/refunds'),
      ('Customers', Icons.people_outline, '/admin/customers'),
      ('Communications', Icons.mark_email_unread_outlined, '/admin/communications'),
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
      body: _search.isNotEmpty ? '{"filters":{"all":"$_search"}}' : '{"filters":{}}',
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
                                onSort: (_, _) {},
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
    return Stack(
      children: [
        EzDataGrid(
          gridPath: 'admin/catalog/products',
          title: 'Products',
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
                  final result = await admin.post('admin/catalog/products/copy/${row['id']}');
                  final newId = result is Map && result['data'] is Map
                      ? (result['data'] as Map)['id']
                      : null;
                  if (context.mounted && newId is int) {
                    context.go('/admin/products/edit/$newId');
                  }
                } catch (error) {
                  if (context.mounted) {
                    ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
                  }
                }
              },
            ),
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
        Positioned(
          top: 0,
          right: 0,
          child: _ReindexFlatsButton(),
        ),
      ],
    );
  }
}

/// Regenera las filas product_flat por idioma y los índices de precio de
/// todo el catálogo (tras activar locales nuevos o importar productos).
class _ReindexFlatsButton extends StatefulWidget {
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
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(message)));
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    } finally {
      if (mounted) setState(() => _running = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return OutlinedButton.icon(
      onPressed: _running ? null : _run,
      icon: _running
          ? const SizedBox(
              width: 14,
              height: 14,
              child: CircularProgressIndicator(strokeWidth: 2, color: AppColors.navyBlue))
          : const Icon(Icons.refresh, size: 16),
      label: const Text('Reindex translations',
          style: TextStyle(fontFamily: 'Poppins', fontSize: 12)),
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
      await admin.post('admin/sales/orders/comment/${widget.order['id']}', body: {
        'comment': _comment.text.trim(),
        'customer_notified': _notify,
      });
      _comment.clear();
      final data = await admin.get('admin/sales/orders/view/${widget.order['id']}');
      if (data is Map && mounted) {
        setState(() => _comments = decodeList(data['comments']));
        ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('The comment has been added successfully.')));
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
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
        Uri.parse('${ApiClient.baseUrl}/api/admin/sales/invoices/print/$invoiceId'),
        headers: {'Authorization': 'Bearer $token'},
      );
      if (response.statusCode != 200 || !mounted) return;
      openHtmlInNewTab(response.bodyBytes);
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final data = widget.order;
    final items = (data['items'] ?? const []) as List;
    final invoices = decodeList(data['invoices']);
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
          const Divider(height: 24),
          const Text('Invoices',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500)),
          const SizedBox(height: 8),
          if (invoices.isEmpty)
            const Text('No invoices yet.',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500))
          else
            Wrap(
              spacing: 8,
              children: [
                for (final invoice in invoices)
                  ActionChip(
                    label: Text('${invoice['increment_id'] ?? invoice['id']}',
                        style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                    onPressed: () => _printInvoice(invoice['id'] as int),
                  ),
              ],
            ),
          const Divider(height: 24),
          const Text('Comments',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500)),
          const SizedBox(height: 8),
          for (final comment in _comments)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text('• ${comment['comment']}'
                  '  (${(comment['customer_notified'] ?? 0) == 1 ? 'notified' : 'internal'}, ${comment['created_at']})',
                  style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
            ),
          if (_comments.isEmpty)
            const Text('No comments yet.',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
          const SizedBox(height: 12),
          TextField(
            controller: _comment,
            maxLines: 2,
            decoration: const InputDecoration(
                labelText: 'Add a comment',
                border: OutlineInputBorder()),
          ),
          const SizedBox(height: 8),
          Row(children: [
            Switch(
              value: _notify,
              activeColor: AppColors.navyBlue, // ignore: deprecated_member_use
              onChanged: (value) => setState(() => _notify = value),
            ),
            const Text('Notify customer',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
            const Spacer(),
            PrimaryButton(
              label: 'Add comment',
              isLoading: _sending,
              onPressed: _sending ? null : _addComment,
            ),
          ]),
        ],
      ),
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
  List<Map<String, dynamic>> _taxCategories = [];
  List<Map<String, dynamic>> _taxRates = [];
  List<Map<String, dynamic>> _exchangeRates = [];
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
    if (mounted) {
      setState(() {
        _channels = decodeList(channels);
        _currencies = decodeList(currencies);
        _locales = decodeList(locales);
        _taxCategories = decodeList(taxCategories);
        _taxRates = decodeList(taxRates);
        _exchangeRates = decodeList(exchangeRates);
        _loading = false;
      });
    }
  }

  void _showError(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
  }

  /// Ids de locale/moneda usados por algún canal: no se pueden borrar.
  Set<int> get _localesInUse => {
        for (final channel in _channels)
          ...decodeIds(channel['locales']),
        for (final channel in _channels) ...[
          if (channel['default_locale_id'] is int) channel['default_locale_id'] as int,
        ],
      };

  Set<int> get _currenciesInUse => {
        for (final channel in _channels)
          ...decodeIds(channel['currencies']),
        for (final channel in _channels) ...[
          if (channel['base_currency_id'] is int) channel['base_currency_id'] as int,
        ],
      };

  Future<void> _delete(String label, String path) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Delete $label?'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          TextButton(onPressed: () => Navigator.pop(context, true), child: const Text('Delete')),
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
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            TextField(controller: code, decoration: const InputDecoration(labelText: 'Code (ISO 4217)')),
            const SizedBox(height: 12),
            TextField(controller: name, decoration: const InputDecoration(labelText: 'Name')),
            const SizedBox(height: 12),
            TextField(controller: symbol, decoration: const InputDecoration(labelText: 'Symbol')),
          ]),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          PrimaryButton(
            label: 'Save',
            onPressed: () => code.text.trim().isEmpty ? null : Navigator.pop(context, true),
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
          : await admin.put('admin/settings/currencies/edit/${currency['id']}', body: body);
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
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              TextField(controller: code, decoration: const InputDecoration(labelText: 'Code (e.g. es)')),
              const SizedBox(height: 12),
              TextField(controller: name, decoration: const InputDecoration(labelText: 'Name')),
              const SizedBox(height: 12),
              DropdownButtonFormField<String>(
                initialValue: direction,
                decoration: const InputDecoration(labelText: 'Direction'),
                items: const [
                  DropdownMenuItem(value: 'ltr', child: Text('LTR')),
                  DropdownMenuItem(value: 'rtl', child: Text('RTL')),
                ],
                onChanged: (value) => setDialogState(() => direction = value ?? 'ltr'),
              ),
            ]),
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
            PrimaryButton(
              label: 'Save',
              onPressed: () => code.text.trim().isEmpty ? null : Navigator.pop(context, true),
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
          : await admin.put('admin/settings/locales/edit/${locale['id']}', body: body);
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _channelDialog([Map<String, dynamic>? channel]) async {
    final code = TextEditingController(text: '${channel?['code'] ?? ''}');
    final name = TextEditingController(text: '${channel?['name'] ?? ''}');
    final hostname = TextEditingController(text: '${channel?['hostname'] ?? ''}');
    final timezone = TextEditingController(text: '${channel?['timezone'] ?? ''}');
    int? defaultLocaleId = channel?['default_locale_id'] as int?;
    int? baseCurrencyId = channel?['base_currency_id'] as int?;
    final selectedLocales = {
      ...decodeIds(channel?['locales']),
    };
    final selectedCurrencies = {
      ...decodeIds(channel?['currencies']),
    };
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(channel == null ? 'New channel' : 'Edit channel'),
          content: SingleChildScrollView(
            child: SizedBox(
              width: 480,
              child: Column(mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start, children: [
                TextField(controller: code, decoration: const InputDecoration(labelText: 'Code')),
                const SizedBox(height: 12),
                TextField(controller: name, decoration: const InputDecoration(labelText: 'Name')),
                const SizedBox(height: 12),
                TextField(controller: hostname, decoration: const InputDecoration(labelText: 'Hostname (optional)')),
                const SizedBox(height: 12),
                TextField(controller: timezone, decoration: const InputDecoration(labelText: 'Timezone (optional)')),
                const SizedBox(height: 12),
                DropdownButtonFormField<int>(
                  initialValue: defaultLocaleId,
                  decoration: const InputDecoration(labelText: 'Default locale'),
                  items: _locales
                      .map((locale) => DropdownMenuItem(
                            value: locale['id'] as int,
                            child: Text('${locale['name'] ?? locale['code']}'),
                          ))
                      .toList(),
                  onChanged: (value) => setDialogState(() {
                    defaultLocaleId = value;
                    if (value != null) selectedLocales.add(value);
                  }),
                ),
                const SizedBox(height: 12),
                DropdownButtonFormField<int>(
                  initialValue: baseCurrencyId,
                  decoration: const InputDecoration(labelText: 'Base currency'),
                  items: _currencies
                      .map((currency) => DropdownMenuItem(
                            value: currency['id'] as int,
                            child: Text('${currency['name'] ?? currency['code']}'),
                          ))
                      .toList(),
                  onChanged: (value) => setDialogState(() {
                    baseCurrencyId = value;
                    if (value != null) selectedCurrencies.add(value);
                  }),
                ),
                const SizedBox(height: 16),
                const Text('Locales', style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                Wrap(
                  spacing: 8,
                  runSpacing: 4,
                  children: [
                    for (final locale in _locales)
                      FilterChip(
                        label: Text('${locale['code']}',
                            style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                        selected: selectedLocales.contains(locale['id']),
                        onSelected: (selected) => setDialogState(() => selected
                            ? selectedLocales.add(locale['id'] as int)
                            : selectedLocales.remove(locale['id'])),
                      ),
                  ],
                ),
                const SizedBox(height: 12),
                const Text('Currencies', style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                Wrap(
                  spacing: 8,
                  runSpacing: 4,
                  children: [
                    for (final currency in _currencies)
                      FilterChip(
                        label: Text('${currency['code']}',
                            style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                        selected: selectedCurrencies.contains(currency['id']),
                        onSelected: (selected) => setDialogState(() => selected
                            ? selectedCurrencies.add(currency['id'] as int)
                            : selectedCurrencies.remove(currency['id'])),
                      ),
                  ],
                ),
              ]),
            ),
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
            PrimaryButton(
              label: 'Save',
              onPressed: () => (code.text.trim().isEmpty ||
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
          : await admin.put('admin/settings/channels/edit/${channel['id']}', body: body);
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _taxRateDialog([Map<String, dynamic>? rate]) async {
    final identifier = TextEditingController(text: '${rate?['identifier'] ?? ''}');
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
              child: Column(mainAxisSize: MainAxisSize.min, children: [
                TextField(
                    controller: identifier,
                    decoration: const InputDecoration(labelText: 'Identifier (e.g. IVA-21)')),
                const SizedBox(height: 12),
                Row(children: [
                  Expanded(
                      child: TextField(
                          controller: country,
                          decoration: const InputDecoration(labelText: 'Country (ISO)'))),
                  const SizedBox(width: 12),
                  Expanded(
                      child: TextField(
                          controller: state,
                          decoration: const InputDecoration(labelText: 'State (* = all)'))),
                ]),
                const SizedBox(height: 12),
                SwitchListTile(
                  dense: true,
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Zip code range', style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                  value: isZip,
                  onChanged: (value) => setDialogState(() => isZip = value),
                ),
                if (isZip)
                  Row(children: [
                    Expanded(
                        child: TextField(
                            controller: zipFrom,
                            decoration: const InputDecoration(labelText: 'Zip from'))),
                    const SizedBox(width: 12),
                    Expanded(
                        child: TextField(
                            controller: zipTo,
                            decoration: const InputDecoration(labelText: 'Zip to'))),
                  ])
                else
                  TextField(
                      controller: zipCode,
                      decoration: const InputDecoration(labelText: 'Zip code (* = all)')),
                const SizedBox(height: 12),
                TextField(
                    controller: percent,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(labelText: 'Rate (%)')),
              ]),
            ),
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
            PrimaryButton(
              label: 'Save',
              onPressed: () =>
                  identifier.text.trim().isEmpty ? null : Navigator.pop(context, true),
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
          : await admin.put('admin/settings/taxes/rates/edit/${rate['id']}', body: body);
      await _load();
    } catch (error) {
      _showError(error);
    }
  }

  Future<void> _taxCategoryDialog([Map<String, dynamic>? category]) async {
    final code = TextEditingController(text: '${category?['code'] ?? ''}');
    final name = TextEditingController(text: '${category?['name'] ?? ''}');
    final description = TextEditingController(text: '${category?['description'] ?? ''}');
    final selected = {
      ...decodeList(category?['taxrates']).map((rate) => rate['id'] as int),
    };
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(category == null ? 'New tax category' : 'Edit tax category'),
          content: SingleChildScrollView(
            child: SizedBox(
              width: 440,
              child: Column(mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start, children: [
                TextField(controller: code, decoration: const InputDecoration(labelText: 'Code')),
                const SizedBox(height: 12),
                TextField(controller: name, decoration: const InputDecoration(labelText: 'Name')),
                const SizedBox(height: 12),
                TextField(controller: description, decoration: const InputDecoration(labelText: 'Description')),
                const SizedBox(height: 16),
                const Text('Tax rates', style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                Wrap(
                  spacing: 8,
                  runSpacing: 4,
                  children: [
                    for (final rate in _taxRates)
                      FilterChip(
                        label: Text('${rate['identifier']}',
                            style: const TextStyle(fontFamily: 'Poppins', fontSize: 12)),
                        selected: selected.contains(rate['id']),
                        onSelected: (value) => setDialogState(() => value
                            ? selected.add(rate['id'] as int)
                            : selected.remove(rate['id'])),
                      ),
                  ],
                ),
              ]),
            ),
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
            PrimaryButton(
              label: 'Save',
              onPressed: () => code.text.trim().isEmpty ? null : Navigator.pop(context, true),
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
          : await admin.put('admin/settings/taxes/categories/edit/${category['id']}', body: body);
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
            .where((currency) =>
                !_exchangeRates.any((row) => row['target_currency'] == currency['id']))
            .toList()
        : _currencies;
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(exchangeRate == null ? 'New exchange rate' : 'Edit exchange rate'),
        content: SizedBox(
          width: 420,
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            if (exchangeRate == null)
              DropdownButtonFormField<int>(
                initialValue: target,
                decoration: const InputDecoration(labelText: 'Target currency'),
                items: availableCurrencies
                    .map((currency) => DropdownMenuItem(
                          value: currency['id'] as int,
                          child: Text('${currency['code']} (${currency['name']})'),
                        ))
                    .toList(),
                onChanged: (value) => target = value,
              )
            else
              ListTile(
                dense: true,
                contentPadding: EdgeInsets.zero,
                title: Text('${exchangeRate['code']} (${exchangeRate['name']})',
                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
              ),
            const SizedBox(height: 12),
            TextField(
                controller: rate,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(labelText: 'Rate (per 1 base currency)')),
          ]),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context, false), child: const Text('Cancel')),
          PrimaryButton(
            label: 'Save',
            onPressed: () =>
                (exchangeRate == null && target == null) ? null : Navigator.pop(context, true),
          ),
        ],
      ),
    );
    if (saved != true || !mounted) return;
    final admin = context.read<AdminProvider>();
    try {
      exchangeRate == null
          ? await admin.post('admin/settings/exchange-rates', body: {
              'target_currency': target,
              'rate': num.tryParse(rate.text) ?? 1,
            })
          : await admin.put('admin/settings/exchange-rates/edit/${exchangeRate['id']}',
              body: {'rate': num.tryParse(rate.text) ?? 1});
      await _load();
    } catch (error) {
      _showError(error);
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
            actions: [
              TextButton.icon(
                onPressed: () => _channelDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('New Channel', style: TextStyle(fontFamily: 'Poppins')),
              ),
            ],
            children: [
              for (final channel in _channels)
                ListTile(
                  title: Text('${channel['name'] ?? channel['code']}',
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                  subtitle: Text(
                      'code: ${channel['code']} · theme: ${channel['theme']} · '
                      'locales: ${decodeIds(channel['locales']).length} · '
                      'currencies: ${decodeIds(channel['currencies']).length}',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                  trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                    IconButton(
                      icon: const Icon(Icons.edit_outlined, size: 18),
                      tooltip: 'Edit',
                      onPressed: () => _channelDialog(channel),
                    ),
                    if (channel['id'] != 1)
                      IconButton(
                        icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                        tooltip: 'Delete',
                        onPressed: () =>
                            _delete('channel', 'admin/settings/channels/${channel['id']}'),
                      ),
                  ]),
                ),
            ],
          ),
          _Section(
            title: 'Currencies',
            actions: [
              TextButton.icon(
                onPressed: () => _currencyDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('New Currency', style: TextStyle(fontFamily: 'Poppins')),
              ),
            ],
            children: [
              for (final currency in _currencies)
                ListTile(
                  title: Text('${currency['name']} (${currency['symbol']})',
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                  subtitle: Text('${currency['code']}',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                  trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                    IconButton(
                      icon: const Icon(Icons.edit_outlined, size: 18),
                      tooltip: 'Edit',
                      onPressed: () => _currencyDialog(currency),
                    ),
                    if (!_currenciesInUse.contains(currency['id']))
                      IconButton(
                        icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                        tooltip: 'Delete',
                        onPressed: () =>
                            _delete('currency', 'admin/settings/currencies/${currency['id']}'),
                      ),
                  ]),
                ),
            ],
          ),
          _Section(
            title: 'Locales',
            actions: [
              TextButton.icon(
                onPressed: () => _localeDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('New Locale', style: TextStyle(fontFamily: 'Poppins')),
              ),
            ],
            children: [
              for (final locale in _locales)
                ListTile(
                  title: Text('${locale['name']}',
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                  subtitle: Text('${locale['code']} · ${locale['direction']}',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                  trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                    IconButton(
                      icon: const Icon(Icons.edit_outlined, size: 18),
                      tooltip: 'Edit',
                      onPressed: () => _localeDialog(locale),
                    ),
                    if (!_localesInUse.contains(locale['id']))
                      IconButton(
                        icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                        tooltip: 'Delete',
                        onPressed: () =>
                            _delete('locale', 'admin/settings/locales/${locale['id']}'),
                      ),
                  ]),
                ),
            ],
          ),
          _Section(
            title: 'Tax categories',
            actions: [
              TextButton.icon(
                onPressed: () => _taxCategoryDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('New Category', style: TextStyle(fontFamily: 'Poppins')),
              ),
            ],
            children: [
              for (final category in _taxCategories)
                ListTile(
                  title: Text('${category['name'] ?? category['code']}',
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                  subtitle: Text(
                      'code: ${category['code']} · rates: ${decodeList(category['taxrates']).map((rate) => rate['identifier']).join(', ')}',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                  trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                    IconButton(
                      icon: const Icon(Icons.edit_outlined, size: 18),
                      tooltip: 'Edit',
                      onPressed: () => _taxCategoryDialog(category),
                    ),
                    IconButton(
                      icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                      tooltip: 'Delete',
                      onPressed: () =>
                          _delete('tax category', 'admin/settings/taxes/categories/${category['id']}'),
                    ),
                  ]),
                ),
            ],
          ),
          _Section(
            title: 'Tax rates',
            actions: [
              TextButton.icon(
                onPressed: () => _taxRateDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('New Rate', style: TextStyle(fontFamily: 'Poppins')),
              ),
            ],
            children: [
              for (final rate in _taxRates)
                ListTile(
                  title: Text('${rate['identifier']} — ${rate['tax_rate']}%',
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                  subtitle: Text(
                      'country: ${rate['country']} · state: ${rate['state'] ?? '*'} · '
                      'zip: ${rate['is_zip'] == 1 ? '${rate['zip_from']}-${rate['zip_to']}' : rate['zip_code']}',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                  trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                    IconButton(
                      icon: const Icon(Icons.edit_outlined, size: 18),
                      tooltip: 'Edit',
                      onPressed: () => _taxRateDialog(rate),
                    ),
                    IconButton(
                      icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                      tooltip: 'Delete',
                      onPressed: () =>
                          _delete('tax rate', 'admin/settings/taxes/rates/${rate['id']}'),
                    ),
                  ]),
                ),
            ],
          ),
          _Section(
            title: 'Exchange rates',
            actions: [
              TextButton.icon(
                onPressed: () => _exchangeRateDialog(),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('New Rate', style: TextStyle(fontFamily: 'Poppins')),
              ),
            ],
            children: [
              if (_exchangeRates.isEmpty)
                const Text('No exchange rates yet.',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500))
              else
                for (final exchangeRate in _exchangeRates)
                  ListTile(
                    title: Text('1 EUR = ${exchangeRate['rate']} ${exchangeRate['code']}',
                        style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                    subtitle: Text('${exchangeRate['name']}',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                    trailing: Row(mainAxisSize: MainAxisSize.min, children: [
                      IconButton(
                        icon: const Icon(Icons.edit_outlined, size: 18),
                        tooltip: 'Edit',
                        onPressed: () => _exchangeRateDialog(exchangeRate),
                      ),
                      IconButton(
                        icon: const Icon(Icons.delete_outline, size: 18, color: AppColors.red600),
                        tooltip: 'Delete',
                        onPressed: () => _delete(
                            'exchange rate', 'admin/settings/exchange-rates/${exchangeRate['id']}'),
                      ),
                    ]),
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
                child: Text(title,
                    style: const TextStyle(
                        fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
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
