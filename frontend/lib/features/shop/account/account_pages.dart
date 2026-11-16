import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../shell/shop_shell.dart';

/// Área de cuenta con sidebar (Profile/Orders/Addresses/Wishlist).
class AccountPage extends StatelessWidget {
  const AccountPage({super.key});

  @override
  Widget build(BuildContext context) {
    final auth = context.watch<AuthProvider>();
    if (!auth.isLoggedIn) {
      return const SignInPrompt();
    }
    return const OrdersPage();
  }
}

class SignInPrompt extends StatelessWidget {
  const SignInPrompt({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          SizedBox(
            height: 420,
            child: Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(
                    Icons.lock_outline,
                    size: 48,
                    color: AppColors.zinc300,
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'Please sign in to access your account.',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 16),
                  ),
                  const SizedBox(height: 20),
                  PrimaryButton(
                    label: 'Sign In',
                    onPressed: () => context.go('/login'),
                  ),
                ],
              ),
            ),
          ),
          const ShopFooterSections(),
        ],
      ),
    );
  }
}

class AccountScaffold extends StatelessWidget {
  const AccountScaffold({
    super.key,
    required this.title,
    required this.child,
    this.trailing,
  });

  final String title;
  final Widget child;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final auth = context.watch<AuthProvider>();
    final width = MediaQuery.of(context).size.width;
    final isNarrow = width < 980;
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);
    if (!auth.isLoggedIn) return const SignInPrompt();

    final menu = [
      ('Profile', Icons.person_outline, '/account/profile'),
      ('Orders', Icons.receipt_long_outlined, '/account/orders'),
      ('Addresses', Icons.location_on_outlined, '/account/addresses'),
      ('Wishlist', Icons.favorite_border, '/account/wishlist'),
    ];
    final currentPath = GoRouterState.of(context).uri.path;

    final sidebar = SizedBox(
      width: isNarrow ? double.infinity : 288,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Tarjeta del cliente
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: AppColors.zinc50,
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
            ),
            child: Row(
              children: [
                CircleAvatar(
                  radius: 26,
                  backgroundColor: Colors.white,
                  child: const Icon(
                    Icons.person_outline,
                    color: AppColors.navyBlue,
                    size: 26,
                  ),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Hello, ${auth.customer!.name}',
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 16,
                          fontWeight: FontWeight.w600,
                          color: AppColors.gray900,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        auth.customer!.email,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 12,
                          color: AppColors.zinc500,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),
          // Navegación
          Container(
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(EzTheme.radiusLg),
              border: Border.all(color: AppColors.zinc200),
            ),
            clipBehavior: Clip.antiAlias,
            child: Column(
              children: [
                for (var i = 0; i < menu.length; i++) ...[
                  _AccountNavItem(
                    label: menu[i].$1,
                    icon: menu[i].$2,
                    selected: currentPath == menu[i].$3,
                    onTap: () => context.go(menu[i].$3),
                  ),
                  if (i != menu.length - 1)
                    const Divider(height: 1, color: AppColors.zinc100),
                ],
              ],
            ),
          ),
          const SizedBox(height: 18),
          SecondaryButton(
            label: 'Logout',
            expand: true,
            onPressed: () async {
              await auth.logout();
              if (context.mounted) context.go('/');
            },
          ),
        ],
      ),
    );

    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1440),
              child: Padding(
                padding: EdgeInsets.fromLTRB(horizontal, 20, horizontal, 40),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Breadcrumb
                    Row(
                      children: [
                        _Crumb(label: 'Home', onTap: () => context.go('/')),
                        const Icon(
                          Icons.chevron_right,
                          size: 16,
                          color: AppColors.zinc400,
                        ),
                        _Crumb(
                          label: 'My Account',
                          onTap: () => context.go('/account/orders'),
                        ),
                        const Icon(
                          Icons.chevron_right,
                          size: 16,
                          color: AppColors.zinc400,
                        ),
                        Text(
                          title,
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 13,
                            color: AppColors.zinc500,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 14),
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      children: [
                        Expanded(
                          child: Text(
                            title,
                            style: const TextStyle(
                              fontFamily: EzTheme.serif,
                              fontSize: 34,
                              height: 1.15,
                              color: AppColors.navyBlue,
                            ),
                          ),
                        ),
                        ?trailing,
                      ],
                    ),
                    const SizedBox(height: 26),
                    isNarrow
                        ? Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              sidebar,
                              const SizedBox(height: 28),
                              child,
                            ],
                          )
                        : Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              sidebar,
                              const SizedBox(width: 40),
                              Expanded(child: child),
                            ],
                          ),
                  ],
                ),
              ),
            ),
          ),
          const ShopFooterSections(),
        ],
      ),
    );
  }
}

class _AccountNavItem extends StatelessWidget {
  const _AccountNavItem({
    required this.label,
    required this.icon,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final IconData icon;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: selected ? AppColors.zinc50 : Colors.white,
      child: InkWell(
        onTap: onTap,
        child: Container(
          height: 52,
          padding: const EdgeInsets.symmetric(horizontal: 18),
          decoration: BoxDecoration(
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
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  label,
                  style: TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 14,
                    fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
                    color: selected ? AppColors.navyBlue : AppColors.zinc600,
                  ),
                ),
              ),
              const Icon(
                Icons.chevron_right,
                size: 18,
                color: AppColors.zinc300,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Crumb extends StatelessWidget {
  const _Crumb({required this.label, required this.onTap});
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Text(
          label,
          style: const TextStyle(
            fontFamily: EzTheme.sans,
            fontSize: 13,
            color: AppColors.zinc500,
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

class OrdersPage extends StatefulWidget {
  const OrdersPage({super.key});

  @override
  State<OrdersPage> createState() => _OrdersPageState();
}

class _OrdersPageState extends State<OrdersPage> {
  List<Order> _orders = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final data = await apiGet('customer/orders');
      if (mounted) {
        setState(() {
          _orders = decodeList(data['data']).map((e) => Order(e)).toList();
          _loading = false;
        });
      }
    } catch (_) {
      if (mounted) setState(() => _loading = false);
    }
  }

  Color _statusColor(String status) {
    switch (status) {
      case 'completed':
        return AppColors.green600;
      case 'processing':
        return AppColors.blue700;
      case 'canceled':
        return AppColors.red600;
      case 'closed':
        return AppColors.zinc600;
      default:
        return AppColors.amber500;
    }
  }

  String _formatDate(String raw) {
    final parsed = DateTime.tryParse(raw.replaceFirst(' ', 'T'));
    if (parsed == null) return raw;
    const months = [
      'Jan',
      'Feb',
      'Mar',
      'Apr',
      'May',
      'Jun',
      'Jul',
      'Aug',
      'Sep',
      'Oct',
      'Nov',
      'Dec',
    ];
    return '${parsed.day.toString().padLeft(2, '0')} '
        '${months[parsed.month - 1]} ${parsed.year}';
  }

  Widget _statusPill(String status) {
    final color = _statusColor(status);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(30),
      ),
      child: Text(
        status.isEmpty
            ? 'pending'
            : status[0].toUpperCase() + status.substring(1),
        style: TextStyle(
          fontFamily: EzTheme.sans,
          fontSize: 12,
          fontWeight: FontWeight.w500,
          color: color,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.of(context).size.width;
    final compact = width < 980;

    if (_loading) {
      return const AccountScaffold(
        title: 'Orders',
        child: Padding(
          padding: EdgeInsets.symmetric(vertical: 120),
          child: Center(
            child: CircularProgressIndicator(color: AppColors.navyBlue),
          ),
        ),
      );
    }

    if (_orders.isEmpty) {
      return AccountScaffold(
        title: 'Orders',
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 96),
          child: Column(
            children: [
              Container(
                width: 84,
                height: 84,
                decoration: const BoxDecoration(
                  shape: BoxShape.circle,
                  color: AppColors.zinc100,
                ),
                child: const Icon(
                  Icons.receipt_long_outlined,
                  size: 34,
                  color: AppColors.zinc400,
                ),
              ),
              const SizedBox(height: 18),
              const Text(
                'You have placed no orders.',
                style: TextStyle(
                  fontFamily: EzTheme.serif,
                  fontSize: 24,
                  color: AppColors.navyBlue,
                ),
              ),
              const SizedBox(height: 20),
              PrimaryButton(
                label: 'Continue Shopping',
                onPressed: () => context.go('/'),
              ),
            ],
          ),
        ),
      );
    }

    final header = Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
      color: AppColors.zinc50,
      child: Row(
        children: [
          const Expanded(
            flex: 3,
            child: Text('Order #', style: _tableHeadStyle),
          ),
          if (!compact)
            const Expanded(
              flex: 3,
              child: Text('Date', style: _tableHeadStyle),
            ),
          const Expanded(
            flex: 3,
            child: Text('Status', style: _tableHeadStyle),
          ),
          const Expanded(
            flex: 2,
            child: Text(
              'Total',
              textAlign: TextAlign.right,
              style: _tableHeadStyle,
            ),
          ),
          const SizedBox(width: 92),
        ],
      ),
    );

    return AccountScaffold(
      title: 'Orders',
      trailing: Text(
        '${_orders.length} ${_orders.length == 1 ? 'order' : 'orders'}',
        style: const TextStyle(
          fontFamily: EzTheme.sans,
          fontSize: 13,
          color: AppColors.zinc500,
        ),
      ),
      child: Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(EzTheme.radiusXl),
          border: Border.all(color: AppColors.zinc200),
        ),
        clipBehavior: Clip.antiAlias,
        child: Column(
          children: [
            header,
            for (var i = 0; i < _orders.length; i++)
              _OrderRow(
                order: _orders[i],
                date: _formatDate(_orders[i].createdAt),
                status: _statusPill(_orders[i].status),
                compact: compact,
                showBorder: i != 0,
              ),
          ],
        ),
      ),
    );
  }
}

const _tableHeadStyle = TextStyle(
  fontFamily: EzTheme.sans,
  fontSize: 12,
  fontWeight: FontWeight.w600,
  letterSpacing: .4,
  color: AppColors.zinc500,
);

class _OrderRow extends StatefulWidget {
  const _OrderRow({
    required this.order,
    required this.date,
    required this.status,
    required this.compact,
    required this.showBorder,
  });

  final Order order;
  final String date;
  final Widget status;
  final bool compact;
  final bool showBorder;

  @override
  State<_OrderRow> createState() => _OrderRowState();
}

class _OrderRowState extends State<_OrderRow> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final order = widget.order;
    final items = order.totalItemCount ?? order.items.length;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: () => context.go('/account/orders/${order.id}'),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 18),
          decoration: BoxDecoration(
            color: _hovering ? AppColors.zinc50 : Colors.white,
            border: widget.showBorder
                ? const Border(top: BorderSide(color: AppColors.zinc100))
                : null,
          ),
          child: Row(
            children: [
              Expanded(
                flex: 3,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '#${order.incrementId}',
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: AppColors.navyBlue,
                      ),
                    ),
                    const SizedBox(height: 3),
                    Text(
                      '$items ${items == 1 ? 'item' : 'items'}',
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 12,
                        color: AppColors.zinc500,
                      ),
                    ),
                  ],
                ),
              ),
              if (!widget.compact)
                Expanded(
                  flex: 3,
                  child: Text(
                    widget.date,
                    style: const TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 13,
                      color: AppColors.zinc600,
                    ),
                  ),
                ),
              Expanded(
                flex: 3,
                child: Align(
                  alignment: Alignment.centerLeft,
                  child: widget.status,
                ),
              ),
              Expanded(
                flex: 2,
                child: Text(
                  order.formattedTotal,
                  textAlign: TextAlign.right,
                  style: const TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: AppColors.gray900,
                  ),
                ),
              ),
              const SizedBox(width: 20),
              SizedBox(
                width: 72,
                child: TextButton(
                  onPressed: () => context.go('/account/orders/${order.id}'),
                  style: TextButton.styleFrom(
                    foregroundColor: AppColors.navyBlue,
                    padding: const EdgeInsets.symmetric(horizontal: 10),
                  ),
                  child: const Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      Text(
                        'View',
                        style: TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 13,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                      SizedBox(width: 2),
                      Icon(Icons.chevron_right, size: 16),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class OrderDetailPage extends StatefulWidget {
  const OrderDetailPage({super.key, required this.orderId});
  final String orderId;

  @override
  State<OrderDetailPage> createState() => _OrderDetailPageState();
}

class _OrderDetailPageState extends State<OrderDetailPage> {
  Order? _order;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final data = await apiGet('customer/orders/${widget.orderId}');
      if (data is Map && mounted) {
        setState(() => _order = Order(Map<String, dynamic>.from(data)));
      }
    } catch (_) {}
  }

  Future<void> _cancel() async {
    setState(() => _busy = true);
    try {
      await apiPost('customer/orders/${widget.orderId}/cancel', body: {});
      await _load();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('The order has been canceled.')),
        );
      }
    } catch (_) {
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _reorder() async {
    setState(() => _busy = true);
    final cart = context.read<CartProvider>();
    try {
      await apiPost('customer/orders/${widget.orderId}/reorder', body: {});
      await cart.refresh();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('The products have been added to your cart.'),
          ),
        );
        context.go('/cart');
      }
    } catch (_) {
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  String _date(String raw) {
    final parsed = DateTime.tryParse(raw.replaceFirst(' ', 'T'));
    if (parsed == null) return raw;
    const months = [
      'Jan',
      'Feb',
      'Mar',
      'Apr',
      'May',
      'Jun',
      'Jul',
      'Aug',
      'Sep',
      'Oct',
      'Nov',
      'Dec',
    ];
    return '${parsed.day} ${months[parsed.month - 1]} ${parsed.year}';
  }

  Color _statusColor(String status) {
    switch (status) {
      case 'completed':
        return AppColors.green600;
      case 'canceled':
      case 'closed':
        return AppColors.red600;
      default:
        return AppColors.blue700;
    }
  }

  Widget _statusPill(String status) {
    final color = _statusColor(status);
    final label = status.isEmpty
        ? 'Pending'
        : status[0].toUpperCase() + status.substring(1);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
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

  Widget _addressCard(String title, Map<String, dynamic>? address) {
    final lines = <String>[];
    if (address != null) {
      final name =
          '${address['first_name'] ?? ''} ${address['last_name'] ?? ''}'.trim();
      if (name.isNotEmpty) lines.add(name);
      final street = [
        address['address1'],
        address['address2'],
      ].where((value) => (value ?? '').toString().trim().isNotEmpty).join(', ');
      if (street.isNotEmpty) lines.add(street);
      final cityLine = [
        address['city'],
        address['postcode'],
        address['country'],
      ].where((value) => (value ?? '').toString().trim().isNotEmpty).join(' ');
      if (cityLine.isNotEmpty) lines.add(cityLine);
      final phone = (address['phone'] ?? '').toString();
      if (phone.isNotEmpty) lines.add(phone);
      final email = (address['email'] ?? '').toString();
      if (email.isNotEmpty) lines.add(email);
    }
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                title.startsWith('Billing')
                    ? Icons.receipt_long_outlined
                    : Icons.local_shipping_outlined,
                size: 18,
                color: AppColors.navyBlue,
              ),
              const SizedBox(width: 8),
              Text(
                title,
                style: const TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: 15,
                  fontWeight: FontWeight.w600,
                  color: AppColors.gray900,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          if (lines.isEmpty)
            const Text(
              '—',
              style: TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 13,
                color: AppColors.zinc500,
              ),
            )
          else
            for (final line in lines)
              Padding(
                padding: const EdgeInsets.only(bottom: 3),
                child: Text(
                  line,
                  style: const TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 13,
                    height: 1.5,
                    color: AppColors.zinc600,
                  ),
                ),
              ),
        ],
      ),
    );
  }

  Widget _row(String label, String value, {bool strong = false}) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 5),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: strong ? 14 : 13,
                fontWeight: strong ? FontWeight.w600 : FontWeight.w400,
                color: strong ? AppColors.gray900 : AppColors.zinc500,
              ),
            ),
          ),
          Text(
            value,
            style: TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: strong ? 16 : 13,
              fontWeight: strong ? FontWeight.w600 : FontWeight.w500,
              color: AppColors.gray900,
            ),
          ),
        ],
      ),
    );
  }

  Widget _itemRow(BuildContext context, OrderItem item, bool narrow) {
    final image = apiUrlRelative(item.image);
    final thumb = ClipRRect(
      borderRadius: BorderRadius.circular(EzTheme.radiusLg),
      child: SizedBox(
        width: 64,
        height: 64,
        child: Container(
          color: AppColors.zinc100,
          child: image != null
              ? Image.network(
                  image,
                  fit: BoxFit.cover,
                  errorBuilder: (_, _, _) => const Icon(
                    Icons.image_outlined,
                    size: 22,
                    color: AppColors.zinc300,
                  ),
                )
              : const Icon(
                  Icons.image_outlined,
                  size: 22,
                  color: AppColors.zinc300,
                ),
        ),
      ),
    );
    final name = Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: item.productId == null
              ? null
              : () => context.go('/product/${item.productId}'),
          child: Text(
            item.name,
            style: const TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 14,
              fontWeight: FontWeight.w500,
              color: AppColors.gray900,
            ),
          ),
        ),
        const SizedBox(height: 3),
        Text(
          'SKU: ${item.sku}',
          style: const TextStyle(
            fontFamily: EzTheme.sans,
            fontSize: 12,
            color: AppColors.zinc500,
          ),
        ),
      ],
    );

    if (narrow) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 12),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            thumb,
            const SizedBox(width: 14),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  name,
                  const SizedBox(height: 8),
                  Text(
                    '${item.formattedPrice} × ${item.qtyOrdered}',
                    style: const TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 13,
                      color: AppColors.zinc500,
                    ),
                  ),
                ],
              ),
            ),
            Text(
              item.formattedTotal,
              style: const TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 14,
                fontWeight: FontWeight.w600,
                color: AppColors.gray900,
              ),
            ),
          ],
        ),
      );
    }

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 14),
      child: Row(
        children: [
          thumb,
          const SizedBox(width: 16),
          Expanded(flex: 4, child: name),
          Expanded(
            child: Text(
              item.formattedPrice,
              style: const TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 13,
                color: AppColors.zinc600,
              ),
            ),
          ),
          Expanded(
            child: Text(
              '${item.qtyOrdered}',
              style: const TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 13,
                color: AppColors.zinc600,
              ),
            ),
          ),
          Expanded(
            child: Align(
              alignment: Alignment.centerRight,
              child: Text(
                item.formattedTotal,
                style: const TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: AppColors.gray900,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final order = _order;
    if (order == null) {
      return const Center(
        child: CircularProgressIndicator(color: AppColors.navyBlue),
      );
    }
    return AccountScaffold(
      title: 'Order #${order.incrementId}',
      trailing: Wrap(
        spacing: 10,
        runSpacing: 8,
        children: [
          if (order.canReorder)
            SecondaryButton(
              label: 'Reorder',
              height: 44,
              onPressed: _busy ? null : _reorder,
            ),
          if (order.canCancel)
            OutlinedButton(
              onPressed: _busy ? null : _cancel,
              style: OutlinedButton.styleFrom(
                minimumSize: const Size(0, 44),
                padding: const EdgeInsets.symmetric(horizontal: 18),
                side: const BorderSide(color: AppColors.zinc300),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                ),
              ),
              child: const Text(
                'Cancel Order',
                style: TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: 14,
                  color: AppColors.red600,
                ),
              ),
            ),
        ],
      ),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final narrow = constraints.maxWidth < 720;
          final items = order.items;
          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  _statusPill(order.status),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      'Placed on ${_date(order.createdAt)}'
                      '  ·  ${items.length} ${items.length == 1 ? 'item' : 'items'}',
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 13,
                        color: AppColors.zinc500,
                      ),
                    ),
                  ),
                  if (!narrow)
                    Text(
                      order.formattedTotal,
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                        color: AppColors.gray900,
                      ),
                    ),
                ],
              ),
              const SizedBox(height: 18),
              Container(
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                  border: Border.all(color: AppColors.zinc200),
                ),
                padding: const EdgeInsets.symmetric(
                  horizontal: 20,
                  vertical: 4,
                ),
                child: Column(
                  children: [
                    if (!narrow)
                      Container(
                        padding: const EdgeInsets.symmetric(vertical: 12),
                        decoration: const BoxDecoration(
                          border: Border(
                            bottom: BorderSide(color: AppColors.zinc100),
                          ),
                        ),
                        child: Row(
                          children: const [
                            Expanded(
                              flex: 4,
                              child: Text(
                                'Product',
                                style: TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 12,
                                  fontWeight: FontWeight.w600,
                                  color: AppColors.zinc500,
                                ),
                              ),
                            ),
                            Expanded(
                              child: Text(
                                'Price',
                                style: TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 12,
                                  fontWeight: FontWeight.w600,
                                  color: AppColors.zinc500,
                                ),
                              ),
                            ),
                            Expanded(
                              child: Text(
                                'Qty',
                                style: TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 12,
                                  fontWeight: FontWeight.w600,
                                  color: AppColors.zinc500,
                                ),
                              ),
                            ),
                            Expanded(
                              child: Align(
                                alignment: Alignment.centerRight,
                                child: Text(
                                  'Subtotal',
                                  style: TextStyle(
                                    fontFamily: EzTheme.sans,
                                    fontSize: 12,
                                    fontWeight: FontWeight.w600,
                                    color: AppColors.zinc500,
                                  ),
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                    for (var i = 0; i < items.length; i++) ...[
                      if (i > 0)
                        const Divider(height: 1, color: AppColors.zinc100),
                      _itemRow(context, items[i], narrow),
                    ],
                  ],
                ),
              ),
              const SizedBox(height: 20),
              if (narrow) ...[
                Container(
                  width: double.infinity,
                  padding: const EdgeInsets.all(20),
                  decoration: BoxDecoration(
                    color: AppColors.zinc50,
                    borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                    border: Border.all(color: AppColors.zinc200),
                  ),
                  child: Column(
                    children: [
                      _row('Sub Total', order.formattedSubTotal),
                      if (order.shippingTitle.isNotEmpty ||
                          order.shippingDescription.isNotEmpty)
                        _row('Shipping', order.formattedShipping),
                      if (order.formattedTax.isNotEmpty &&
                          order.formattedTax != '€0.00')
                        _row('Tax', order.formattedTax),
                      if (order.formattedDiscount.isNotEmpty &&
                          order.formattedDiscount != '€0.00' &&
                          order.formattedDiscount != '0.00')
                        _row('Discount', '-${order.formattedDiscount}'),
                      const Divider(height: 20),
                      _row('Grand Total', order.formattedTotal, strong: true),
                    ],
                  ),
                ),
                const SizedBox(height: 16),
              ] else
                Align(
                  alignment: Alignment.centerRight,
                  child: Container(
                    width: 360,
                    padding: const EdgeInsets.all(20),
                    decoration: BoxDecoration(
                      color: AppColors.zinc50,
                      borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                      border: Border.all(color: AppColors.zinc200),
                    ),
                    child: Column(
                      children: [
                        _row('Sub Total', order.formattedSubTotal),
                        if (order.shippingTitle.isNotEmpty ||
                            order.shippingDescription.isNotEmpty)
                          _row('Shipping', order.formattedShipping),
                        if (order.formattedTax.isNotEmpty &&
                            order.formattedTax != '€0.00')
                          _row('Tax', order.formattedTax),
                        if (order.formattedDiscount.isNotEmpty &&
                            order.formattedDiscount != '€0.00' &&
                            order.formattedDiscount != '0.00')
                          _row('Discount', '-${order.formattedDiscount}'),
                        const Divider(height: 20),
                        _row('Grand Total', order.formattedTotal, strong: true),
                      ],
                    ),
                  ),
                ),
              const SizedBox(height: 20),
              Wrap(
                spacing: 16,
                runSpacing: 16,
                children: [
                  SizedBox(
                    width: narrow
                        ? double.infinity
                        : (constraints.maxWidth - 16) / 2,
                    child: _addressCard(
                      'Billing Address',
                      order.billingAddress,
                    ),
                  ),
                  SizedBox(
                    width: narrow
                        ? double.infinity
                        : (constraints.maxWidth - 16) / 2,
                    child: _addressCard(
                      'Shipping Address',
                      order.shippingAddress,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Wrap(
                spacing: 16,
                runSpacing: 16,
                children: [
                  SizedBox(
                    width: narrow
                        ? double.infinity
                        : (constraints.maxWidth - 16) / 2,
                    child: _addressCard('Shipping Method', {
                      'first_name': order.shippingTitle.isEmpty
                          ? 'Flat Rate'
                          : order.shippingTitle,
                      'address1': order.shippingDescription,
                    }),
                  ),
                  SizedBox(
                    width: narrow
                        ? double.infinity
                        : (constraints.maxWidth - 16) / 2,
                    child: _addressCard('Payment Method', {
                      'first_name':
                          order.payment?['method_title']?.toString() ??
                          'Cash On Delivery',
                    }),
                  ),
                ],
              ),
            ],
          );
        },
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Addresses
// ---------------------------------------------------------------------------

class AddressesPage extends StatefulWidget {
  const AddressesPage({super.key});

  @override
  State<AddressesPage> createState() => _AddressesPageState();
}

class _AddressesPageState extends State<AddressesPage> {
  List<Address> _addresses = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final data = await apiGet('customer/addresses');
      if (mounted) {
        setState(() {
          _addresses = decodeList(
            data['data'] ?? data,
          ).map((e) => Address(e)).toList();
          _loading = false;
        });
      }
    } catch (_) {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<void> _delete(int id) async {
    try {
      await apiDelete('customer/addresses/$id');
      await _load();
    } catch (_) {}
  }

  Future<void> _openEditor([Address? address]) async {
    await showDialog(
      context: context,
      builder: (context) => AddressEditorDialog(address: address),
    );
    _load();
  }

  @override
  Widget build(BuildContext context) {
    return AccountScaffold(
      title: 'Addresses',
      trailing: PrimaryButton(
        label: 'Add Address',
        height: 44,
        icon: Icons.add,
        onPressed: () => _openEditor(),
      ),
      child: _loading
          ? const Padding(
              padding: EdgeInsets.symmetric(vertical: 110),
              child: Center(
                child: CircularProgressIndicator(color: AppColors.navyBlue),
              ),
            )
          : _addresses.isEmpty
          ? Container(
              padding: const EdgeInsets.symmetric(vertical: 70, horizontal: 24),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                border: Border.all(color: AppColors.zinc200),
              ),
              child: Column(
                children: [
                  const Icon(
                    Icons.location_on_outlined,
                    size: 40,
                    color: AppColors.zinc300,
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'You have no saved addresses yet.',
                    style: TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 15,
                      color: AppColors.zinc500,
                    ),
                  ),
                  const SizedBox(height: 18),
                  SecondaryButton(
                    label: 'Add your first address',
                    onPressed: () => _openEditor(),
                  ),
                ],
              ),
            )
          : LayoutBuilder(
              builder: (context, constraints) {
                final columns = constraints.maxWidth >= 900
                    ? 2
                    : (constraints.maxWidth >= 560 ? 2 : 1);
                const gap = 20.0;
                final cardWidth =
                    (constraints.maxWidth - gap * (columns - 1)) / columns;
                return Wrap(
                  spacing: gap,
                  runSpacing: gap,
                  children: _addresses.map((address) {
                    return SizedBox(
                      width: cardWidth,
                      child: _AddressCard(
                        address: address,
                        onEdit: () => _openEditor(address),
                        onDelete: () => _delete(address.id ?? 0),
                      ),
                    );
                  }).toList(),
                );
              },
            ),
    );
  }
}

class _AddressCard extends StatelessWidget {
  const _AddressCard({
    required this.address,
    required this.onEdit,
    required this.onDelete,
  });

  final Address address;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(22),
      decoration: BoxDecoration(
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
                  '${address.firstName} ${address.lastName}',
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: AppColors.gray900,
                  ),
                ),
              ),
              if (address.isDefault)
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 3,
                  ),
                  decoration: BoxDecoration(
                    color: AppColors.navyBlue.withValues(alpha: 0.08),
                    borderRadius: BorderRadius.circular(30),
                  ),
                  child: const Text(
                    'Default',
                    style: TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 11,
                      fontWeight: FontWeight.w500,
                      color: AppColors.navyBlue,
                    ),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 12),
          _addressLine(Icons.home_outlined, address.address1),
          _addressLine(
            Icons.location_city_outlined,
            [
              if (address.city.isNotEmpty) address.city,
              if (address.state.isNotEmpty) address.state,
              if (address.postcode.isNotEmpty) address.postcode,
            ].join(', '),
          ),
          _addressLine(Icons.public_outlined, address.country),
          _addressLine(Icons.phone_outlined, address.phone),
          const SizedBox(height: 16),
          const Divider(height: 1, color: AppColors.zinc100),
          const SizedBox(height: 8),
          Row(
            children: [
              TextButton.icon(
                onPressed: onEdit,
                icon: const Icon(
                  Icons.edit_outlined,
                  size: 17,
                  color: AppColors.navyBlue,
                ),
                label: const Text(
                  'Edit',
                  style: TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 13,
                    fontWeight: FontWeight.w500,
                    color: AppColors.navyBlue,
                  ),
                ),
              ),
              const SizedBox(width: 6),
              TextButton.icon(
                onPressed: onDelete,
                icon: const Icon(
                  Icons.delete_outline,
                  size: 17,
                  color: AppColors.red600,
                ),
                label: const Text(
                  'Delete',
                  style: TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 13,
                    fontWeight: FontWeight.w500,
                    color: AppColors.red600,
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _addressLine(IconData icon, String value) {
    if (value.trim().isEmpty) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 16, color: AppColors.zinc400),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 13,
                height: 1.45,
                color: AppColors.zinc600,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class AddressEditorDialog extends StatefulWidget {
  const AddressEditorDialog({super.key, this.address});
  final Address? address;

  @override
  State<AddressEditorDialog> createState() => _AddressEditorDialogState();
}

class _AddressEditorDialogState extends State<AddressEditorDialog> {
  late final TextEditingController _firstName;
  late final TextEditingController _lastName;
  late final TextEditingController _address;
  late final TextEditingController _city;
  late final TextEditingController _postcode;
  late final TextEditingController _phone;
  String _country = 'ES';

  @override
  void initState() {
    super.initState();
    final a = widget.address;
    _firstName = TextEditingController(text: a?.firstName ?? '');
    _lastName = TextEditingController(text: a?.lastName ?? '');
    _address = TextEditingController(text: a?.address1 ?? '');
    _city = TextEditingController(text: a?.city ?? '');
    _postcode = TextEditingController(text: a?.postcode ?? '');
    _phone = TextEditingController(text: a?.phone ?? '');
    _country = a?.country ?? 'ES';
  }

  Future<void> _save() async {
    final payload = {
      'first_name': _firstName.text,
      'last_name': _lastName.text,
      'address': _address.text,
      'city': _city.text,
      'postcode': _postcode.text,
      'phone': _phone.text,
      'country': _country,
    };
    final messenger = ScaffoldMessenger.of(context);
    try {
      if (widget.address == null) {
        await apiPost('customer/addresses', body: payload);
      } else {
        await apiPut(
          'customer/addresses/edit/${widget.address!.id}',
          body: payload,
        );
      }
      if (mounted) Navigator.of(context).pop();
    } catch (error) {
      messenger.showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    Widget field(String label, TextEditingController controller) => TextField(
      controller: controller,
      decoration: InputDecoration(labelText: label, fillColor: Colors.white),
    );
    return AlertDialog(
      title: Text(widget.address == null ? 'Add Address' : 'Edit Address'),
      content: SizedBox(
        width: 480,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                Expanded(child: field('First name', _firstName)),
                const SizedBox(width: 12),
                Expanded(child: field('Last name', _lastName)),
              ],
            ),
            const SizedBox(height: 12),
            field('Address', _address),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(child: field('City', _city)),
                const SizedBox(width: 12),
                Expanded(child: field('Postcode', _postcode)),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(child: field('Phone', _phone)),
                const SizedBox(width: 12),
                Expanded(
                  child: DropdownButtonFormField<String>(
                    initialValue: _country,
                    decoration: const InputDecoration(labelText: 'Country'),
                    items: const [
                      DropdownMenuItem(value: 'ES', child: Text('Spain')),
                      DropdownMenuItem(
                        value: 'US',
                        child: Text('United States'),
                      ),
                      DropdownMenuItem(value: 'FR', child: Text('France')),
                      DropdownMenuItem(value: 'DE', child: Text('Germany')),
                      DropdownMenuItem(value: 'MX', child: Text('Mexico')),
                    ],
                    onChanged: (value) =>
                        setState(() => _country = value ?? 'ES'),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        PrimaryButton(label: 'Save', onPressed: _save),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Wishlist
// ---------------------------------------------------------------------------

class WishlistPage extends StatefulWidget {
  const WishlistPage({super.key});

  @override
  State<WishlistPage> createState() => _WishlistPageState();
}

class _WishlistPageState extends State<WishlistPage> {
  List<Product> _products = [];
  bool _loading = true;
  final Set<int> _busy = {};

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final data = await apiGet('customer/wishlist');
      if (mounted) {
        setState(() {
          _products = decodeList(data['data']).map((e) => Product(e)).toList();
          _loading = false;
        });
      }
    } catch (_) {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<void> _moveToCart(Product product) async {
    final itemId = product.wishlistId ?? product.id ?? 0;
    final messenger = ScaffoldMessenger.of(context);
    setState(() => _busy.add(itemId));
    try {
      await apiPost('customer/wishlist/$itemId/move-to-cart', body: {});
      if (!mounted) return;
      await context.read<CartProvider>().refresh();
      await _load();
      messenger.showSnackBar(
        const SnackBar(content: Text('Product moved to cart.')),
      );
    } catch (_) {
      messenger.showSnackBar(
        const SnackBar(content: Text('Unable to move the product to cart.')),
      );
    } finally {
      if (mounted) setState(() => _busy.remove(itemId));
    }
  }

  Future<void> _remove(Product product) async {
    final itemId = product.wishlistId ?? product.id ?? 0;
    setState(() => _busy.add(itemId));
    try {
      await apiDelete('customer/wishlist/$itemId');
      await _load();
    } catch (_) {
    } finally {
      if (mounted) setState(() => _busy.remove(itemId));
    }
  }

  Future<void> _removeAll() async {
    try {
      await apiDelete('customer/wishlist/all');
      await _load();
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const AccountScaffold(
        title: 'Wishlist',
        child: Padding(
          padding: EdgeInsets.symmetric(vertical: 120),
          child: Center(
            child: CircularProgressIndicator(color: AppColors.navyBlue),
          ),
        ),
      );
    }

    if (_products.isEmpty) {
      return AccountScaffold(
        title: 'Wishlist',
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 96),
          child: Column(
            children: [
              Container(
                width: 84,
                height: 84,
                decoration: const BoxDecoration(
                  shape: BoxShape.circle,
                  color: AppColors.zinc100,
                ),
                child: const Icon(
                  Icons.favorite_border,
                  size: 34,
                  color: AppColors.zinc400,
                ),
              ),
              const SizedBox(height: 18),
              const Text(
                'Your wishlist is empty.',
                style: TextStyle(
                  fontFamily: EzTheme.serif,
                  fontSize: 24,
                  color: AppColors.navyBlue,
                ),
              ),
              const SizedBox(height: 6),
              const Text(
                'Save the products you love and come back to them later.',
                textAlign: TextAlign.center,
                style: TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: 14,
                  color: AppColors.zinc500,
                ),
              ),
              const SizedBox(height: 22),
              PrimaryButton(
                label: 'Continue Shopping',
                onPressed: () => context.go('/'),
              ),
            ],
          ),
        ),
      );
    }

    return AccountScaffold(
      title: 'Wishlist',
      trailing: SecondaryButton(
        label: 'Delete All',
        height: 42,
        onPressed: _removeAll,
      ),
      child: Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(EzTheme.radiusXl),
          border: Border.all(color: AppColors.zinc200),
        ),
        clipBehavior: Clip.antiAlias,
        child: Column(
          children: [
            for (var i = 0; i < _products.length; i++)
              _WishlistRow(
                product: _products[i],
                busy: _busy.contains(
                  _products[i].wishlistId ?? _products[i].id ?? 0,
                ),
                showBorder: i != 0,
                onMove: () => _moveToCart(_products[i]),
                onRemove: () => _remove(_products[i]),
              ),
          ],
        ),
      ),
    );
  }
}

class _WishlistRow extends StatelessWidget {
  const _WishlistRow({
    required this.product,
    required this.busy,
    required this.showBorder,
    required this.onMove,
    required this.onRemove,
  });

  final Product product;
  final bool busy;
  final bool showBorder;
  final VoidCallback onMove;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    final image = apiUrlRelative(product.imageUrl);
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        border: showBorder
            ? const Border(top: BorderSide(color: AppColors.zinc100))
            : null,
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          GestureDetector(
            onTap: () => context.go('/product/${product.id}'),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(EzTheme.radiusLg),
              child: SizedBox(
                width: 104,
                height: 112,
                child: Container(
                  color: AppColors.zinc100,
                  child: image != null
                      ? Image.network(
                          image,
                          fit: BoxFit.cover,
                          errorBuilder: (_, _, _) => const Icon(
                            Icons.image_outlined,
                            size: 30,
                            color: AppColors.zinc300,
                          ),
                        )
                      : const Icon(
                          Icons.image_outlined,
                          size: 30,
                          color: AppColors.zinc300,
                        ),
                ),
              ),
            ),
          ),
          const SizedBox(width: 20),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                GestureDetector(
                  onTap: () => context.go('/product/${product.id}'),
                  child: Text(
                    product.name,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 16,
                      fontWeight: FontWeight.w500,
                      color: AppColors.gray900,
                    ),
                  ),
                ),
                const SizedBox(height: 6),
                Row(
                  children: [
                    Text(
                      product.formattedPrice,
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: AppColors.gray900,
                      ),
                    ),
                    if (product.onSale) ...[
                      const SizedBox(width: 8),
                      Text(
                        product.formattedRegularPrice,
                        style: const TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 12,
                          color: AppColors.zinc400,
                          decoration: TextDecoration.lineThrough,
                        ),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: 6),
                Row(
                  children: [
                    Container(
                      width: 7,
                      height: 7,
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: product.inStock
                            ? AppColors.emerald500
                            : AppColors.red500,
                      ),
                    ),
                    const SizedBox(width: 6),
                    Text(
                      product.inStock ? 'In Stock' : 'Out of Stock',
                      style: TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 12,
                        color: product.inStock
                            ? AppColors.emerald600
                            : AppColors.red600,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(width: 16),
          SizedBox(
            width: 168,
            child: PrimaryButton(
              label: 'Move to Cart',
              height: 44,
              isLoading: busy,
              onPressed: busy ? null : onMove,
            ),
          ),
          const SizedBox(width: 6),
          IconButton(
            tooltip: 'Remove from wishlist',
            icon: const Icon(
              Icons.delete_outline,
              size: 20,
              color: AppColors.zinc500,
            ),
            onPressed: busy ? null : onRemove,
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

class ProfilePage extends StatefulWidget {
  const ProfilePage({super.key});

  @override
  State<ProfilePage> createState() => _ProfilePageState();
}

class _ProfilePageState extends State<ProfilePage> {
  final _firstName = TextEditingController();
  final _lastName = TextEditingController();
  final _newPassword = TextEditingController();
  bool _hydrated = false;
  bool _saving = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (_hydrated) return;
    final customer = context.read<AuthProvider>().customer;
    if (customer != null) {
      _firstName.text = customer.firstName;
      _lastName.text = customer.lastName;
      _hydrated = true;
    }
  }

  @override
  void dispose() {
    _firstName.dispose();
    _lastName.dispose();
    _newPassword.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final authProvider = context.read<AuthProvider>();
    final messenger = ScaffoldMessenger.of(context);
    setState(() => _saving = true);
    try {
      await apiPut(
        'customer/profile',
        body: {
          'first_name': _firstName.text,
          'last_name': _lastName.text,
          if (_newPassword.text.isNotEmpty) 'new_password': _newPassword.text,
        },
      );
      await authProvider.restore();
      _newPassword.clear();
      messenger.showSnackBar(
        const SnackBar(
          content: Text('Your profile has been updated successfully.'),
        ),
      );
    } catch (error) {
      messenger.showSnackBar(
        SnackBar(content: Text('$error'), backgroundColor: AppColors.red600),
      );
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final customer = context.watch<AuthProvider>().customer;
    final wide = MediaQuery.of(context).size.width >= 1200;

    Widget field(String label, Widget input) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: const TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 13,
              fontWeight: FontWeight.w500,
              color: AppColors.gray900,
            ),
          ),
          const SizedBox(height: 7),
          input,
        ],
      );
    }

    final firstName = field('First Name', TextField(controller: _firstName));
    final lastName = field('Last Name', TextField(controller: _lastName));

    return AccountScaffold(
      title: 'Profile',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 680),
            child: Container(
              width: double.infinity,
              padding: const EdgeInsets.all(28),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                border: Border.all(color: AppColors.zinc200),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'Profile Information',
                    style: TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 17,
                      fontWeight: FontWeight.w600,
                      color: AppColors.gray900,
                    ),
                  ),
                  const SizedBox(height: 22),
                  if (wide)
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Expanded(child: firstName),
                        const SizedBox(width: 18),
                        Expanded(child: lastName),
                      ],
                    )
                  else ...[
                    firstName,
                    const SizedBox(height: 16),
                    lastName,
                  ],
                  const SizedBox(height: 16),
                  field(
                    'Email',
                    Container(
                      height: 54,
                      alignment: Alignment.centerLeft,
                      padding: const EdgeInsets.symmetric(horizontal: 18),
                      decoration: BoxDecoration(
                        color: AppColors.zinc50,
                        borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                        border: Border.all(color: AppColors.zinc200),
                      ),
                      child: Text(
                        customer?.email ?? '',
                        style: const TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 14,
                          color: AppColors.zinc500,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(height: 26),
                  const Divider(height: 1, color: AppColors.zinc100),
                  const SizedBox(height: 24),
                  const Text(
                    'Change Password',
                    style: TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 17,
                      fontWeight: FontWeight.w600,
                      color: AppColors.gray900,
                    ),
                  ),
                  const SizedBox(height: 6),
                  const Text(
                    'Leave this blank unless you want to set a new password.',
                    style: TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 13,
                      color: AppColors.zinc500,
                    ),
                  ),
                  const SizedBox(height: 16),
                  field(
                    'New Password',
                    TextField(
                      controller: _newPassword,
                      obscureText: true,
                      decoration: const InputDecoration(
                        hintText: 'At least 6 characters',
                      ),
                    ),
                  ),
                  const SizedBox(height: 26),
                  PrimaryButton(
                    label: 'Save Changes',
                    height: 48,
                    isLoading: _saving,
                    onPressed: _saving ? null : _save,
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
