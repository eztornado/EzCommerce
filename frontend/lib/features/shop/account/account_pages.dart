import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../catalog/product_card.dart';

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
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.lock_outline, size: 48, color: AppColors.zinc300),
          const SizedBox(height: 12),
          const Text('Please sign in to access your account.',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 16)),
          const SizedBox(height: 20),
          PrimaryButton(label: 'Sign In', onPressed: () => context.go('/login')),
        ],
      ),
    );
  }
}

class AccountScaffold extends StatelessWidget {
  const AccountScaffold({super.key, required this.title, required this.child});
  final String title;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final auth = context.watch<AuthProvider>();
    final isNarrow = MediaQuery.of(context).size.width < 900;
    if (!auth.isLoggedIn) return const SignInPrompt();

    final menu = [
      ('Profile', Icons.person_outline, '/account/profile'),
      ('Orders', Icons.receipt_long_outlined, '/account/orders'),
      ('Addresses', Icons.location_on_outlined, '/account/addresses'),
      ('Wishlist', Icons.favorite_border, '/account/wishlist'),
    ];

    final sidebar = Container(
      constraints: const BoxConstraints(maxWidth: 320),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Héroe
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              border: Border.all(color: AppColors.zinc200),
            ),
            child: Row(
              children: [
                CircleAvatar(
                  radius: 28,
                  backgroundColor: AppColors.zinc100,
                  child: const Icon(Icons.person, color: AppColors.navyBlue),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Hello, ${auth.customer!.name}',
                          style: const TextStyle(
                              fontFamily: 'Poppins', fontSize: 17, fontWeight: FontWeight.w500)),
                      Text(auth.customer!.email,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          // Menú
          Container(
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(EzTheme.radiusLg),
              border: Border.all(color: AppColors.zinc200),
            ),
            child: Column(
              children: menu.map((entry) {
                final selected = GoRouterState.of(context).uri.path == entry.$3;
                return ListTile(
                  dense: true,
                  tileColor: selected ? AppColors.zinc100 : null,
                  leading: Icon(entry.$2, size: 20, color: AppColors.zinc600),
                  title: Text(entry.$1,
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 15)),
                  trailing: const Icon(Icons.arrow_forward_ios, size: 14, color: AppColors.zinc400),
                  onTap: () => context.go(entry.$3),
                );
              }).toList(),
            ),
          ),
          const SizedBox(height: 16),
          Center(
            child: SizedBox(
              width: 240,
              child: OutlinedButton(
                onPressed: () async {
                  await auth.logout();
                  if (context.mounted) context.go('/');
                },
                style: OutlinedButton.styleFrom(
                  side: const BorderSide(color: AppColors.navyBlue),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(EzTheme.radiusLg)),
                ),
                child: const Text('Logout',
                    style: TextStyle(fontFamily: 'Poppins', color: AppColors.navyBlue)),
              ),
            ),
          ),
        ],
      ),
    );

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(title,
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
          const SizedBox(height: 24),
          isNarrow
              ? Column(children: [sidebar, const SizedBox(height: 24), child])
              : Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    sidebar,
                    const SizedBox(width: 32),
                    Expanded(child: child),
                  ],
                ),
        ],
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

  @override
  Widget build(BuildContext context) {
    return AccountScaffold(
      title: 'Orders',
      child: _loading
          ? const Center(child: CircularProgressIndicator(color: AppColors.navyBlue))
          : _orders.isEmpty
              ? const Center(child: Text('You have no orders yet.'))
              : Column(
                  children: _orders.map((order) {
                    return Container(
                      margin: const EdgeInsets.only(bottom: 16),
                      padding: const EdgeInsets.all(20),
                      decoration: BoxDecoration(
                        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                        border: Border.all(color: AppColors.zinc200),
                      ),
                      child: Row(
                        children: [
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Row(
                                  children: [
                                    Text('#${order.incrementId}',
                                        style: const TextStyle(
                                            fontFamily: 'Poppins',
                                            fontSize: 16,
                                            fontWeight: FontWeight.w600,
                                            color: AppColors.navyBlue)),
                                    const SizedBox(width: 12),
                                    Container(
                                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 3),
                                      decoration: BoxDecoration(
                                        color: _statusColor(order.status).withOpacity(0.12),
                                        borderRadius: BorderRadius.circular(20),
                                      ),
                                      child: Text(
                                        order.status,
                                        style: TextStyle(
                                            fontFamily: 'Poppins',
                                            fontSize: 12,
                                            color: _statusColor(order.status)),
                                      ),
                                    ),
                                  ],
                                ),
                                const SizedBox(height: 4),
                                Text(
                                  '${order.createdAt} · ${order.items.length} items',
                                  style: TextStyle(
                                      fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500),
                                ),
                              ],
                            ),
                          ),
                          Column(
                            crossAxisAlignment: CrossAxisAlignment.end,
                            children: [
                              Text(order.formattedTotal,
                                  style: const TextStyle(
                                      fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w600)),
                              TextButton(
                                onPressed: () => context.go('/account/orders/${order.id}'),
                                child: const Text('View',
                                    style: TextStyle(
                                        fontFamily: 'Poppins', fontSize: 13, color: AppColors.blue700)),
                              ),
                            ],
                          ),
                        ],
                      ),
                    );
                  }).toList(),
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

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final data = await apiGet('customer/orders/${widget.orderId}');
      if (data is Map && mounted) setState(() => _order = Order(Map<String, dynamic>.from(data)));
    } catch (_) {}
  }

  Future<void> _cancel() async {
    try {
      await apiPost('customer/orders/${widget.orderId}/cancel', body: {});
      await _load();
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(const SnackBar(content: Text('The order has been canceled.')));
      }
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final order = _order;
    if (order == null) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    return AccountScaffold(
      title: 'Order #${order.incrementId}',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Wrap(
            spacing: 12,
            children: [
              if (order.canCancel)
                OutlinedButton(
                  onPressed: _cancel,
                  style: OutlinedButton.styleFrom(side: const BorderSide(color: AppColors.red600)),
                  child: const Text('Cancel Order',
                      style: TextStyle(fontFamily: 'Poppins', color: AppColors.red600)),
                ),
            ],
          ),
          const SizedBox(height: 16),
          ...order.items.map((item) => Container(
                margin: const EdgeInsets.only(bottom: 12),
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                  border: Border.all(color: AppColors.zinc200),
                ),
                child: Row(
                  children: [
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(item.name,
                              style: const TextStyle(
                                  fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w500)),
                          Text('SKU: ${item.sku} · Qty: ${item.qtyOrdered}',
                              style: TextStyle(
                                  fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                        ],
                      ),
                    ),
                    Text(item.total.toStringAsFixed(2),
                        style: const TextStyle(
                            fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w600)),
                  ],
                ),
              )),
          if (order.billingAddress != null) ...[
            const SizedBox(height: 8),
            const Text('Billing Address',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
            Text(
              '${order.billingAddress!['address1'] ?? ''}, ${order.billingAddress!['city'] ?? ''} '
              '${order.billingAddress!['postcode'] ?? ''} (${order.billingAddress!['country'] ?? ''})',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.zinc500),
            ),
          ],
        ],
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
          _addresses = decodeList(data['data'] ?? data).map((e) => Address(e)).toList();
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
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          PrimaryButton(label: 'Add Address', onPressed: () => _openEditor()),
          const SizedBox(height: 24),
          if (_loading)
            const Center(child: CircularProgressIndicator(color: AppColors.navyBlue))
          else if (_addresses.isEmpty)
            const Text('No addresses saved yet.')
          else
            Wrap(
              spacing: 16,
              runSpacing: 16,
              children: _addresses.map((address) {
                return Container(
                  width: 320,
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
                          Expanded(
                            child: Text('${address.firstName} ${address.lastName}',
                                style: const TextStyle(
                                    fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w600)),
                          ),
                          if (address.isDefault)
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                              decoration: BoxDecoration(
                                color: AppColors.navyBlue,
                                borderRadius: BorderRadius.circular(12),
                              ),
                              child: const Text('Default',
                                  style: TextStyle(color: Colors.white, fontSize: 10)),
                            ),
                        ],
                      ),
                      const SizedBox(height: 6),
                      Text(
                        '${address.address1}\n${address.city}, ${address.state} ${address.postcode}\n${address.country} · ${address.phone}',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500, height: 1.5),
                      ),
                      const SizedBox(height: 12),
                      Row(
                        children: [
                          TextButton(
                            onPressed: () => _openEditor(address),
                            child: const Text('Edit',
                                style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.blue700)),
                          ),
                          TextButton(
                            onPressed: () => _delete(address.id ?? 0),
                            child: const Text('Delete',
                                style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.red600)),
                          ),
                        ],
                      ),
                    ],
                  ),
                );
              }).toList(),
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
    try {
      if (widget.address == null) {
        await apiPost('customer/addresses', body: payload);
      } else {
        await apiPut('customer/addresses/edit/${widget.address!.id}', body: payload);
      }
      if (mounted) Navigator.of(context).pop();
    } catch (error) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
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
            Row(children: [
              Expanded(child: field('First name', _firstName)),
              const SizedBox(width: 12),
              Expanded(child: field('Last name', _lastName)),
            ]),
            const SizedBox(height: 12),
            field('Address', _address),
            const SizedBox(height: 12),
            Row(children: [
              Expanded(child: field('City', _city)),
              const SizedBox(width: 12),
              Expanded(child: field('Postcode', _postcode)),
            ]),
            const SizedBox(height: 12),
            Row(children: [
              Expanded(child: field('Phone', _phone)),
              const SizedBox(width: 12),
              Expanded(
                child: DropdownButtonFormField<String>(
                  value: _country,
                  decoration: const InputDecoration(labelText: 'Country'),
                  items: const [
                    DropdownMenuItem(value: 'ES', child: Text('Spain')),
                    DropdownMenuItem(value: 'US', child: Text('United States')),
                    DropdownMenuItem(value: 'FR', child: Text('France')),
                    DropdownMenuItem(value: 'DE', child: Text('Germany')),
                    DropdownMenuItem(value: 'MX', child: Text('Mexico')),
                  ],
                  onChanged: (value) => setState(() => _country = value ?? 'ES'),
                ),
              ),
            ]),
          ],
        ),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
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

  Future<void> _moveToCart(int itemId) async {
    try {
      await apiPost('customer/wishlist/$itemId/move-to-cart', body: {});
      await context.read<CartProvider>().refresh();
      await _load();
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    return AccountScaffold(
      title: 'Wishlist',
      child: _loading
          ? const Center(child: CircularProgressIndicator(color: AppColors.navyBlue))
          : _products.isEmpty
              ? const Text('No products were added to the wishlist page.')
              : GridView.builder(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: _products.length,
                  gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                    maxCrossAxisExtent: 300,
                    mainAxisExtent: 430,
                    crossAxisSpacing: 24,
                    mainAxisSpacing: 24,
                  ),
                  itemBuilder: (context, index) {
                    final product = _products[index];
                    return Column(
                      children: [
                        Expanded(child: ProductCard(product: product)),
                        const SizedBox(height: 8),
                        SizedBox(
                          width: double.infinity,
                          child: SecondaryButton(
                            label: 'Move to Cart',
                            onPressed: () => _moveToCart(product.id ?? 0),
                          ),
                        ),
                      ],
                    );
                  },
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

  @override
  void initState() {
    super.initState();
    final customer = context.read<AuthProvider>().customer;
    if (customer != null) {
      _firstName.text = customer.firstName;
      _lastName.text = customer.lastName;
    }
  }

  @override
  Widget build(BuildContext context) {
    return AccountScaffold(
      title: 'Profile',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('First Name', style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
          TextField(controller: _firstName, decoration: const InputDecoration(fillColor: Colors.white)),
          const SizedBox(height: 12),
          const Text('Last Name', style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
          TextField(controller: _lastName, decoration: const InputDecoration(fillColor: Colors.white)),
          const SizedBox(height: 12),
          const Text('New Password (optional)',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 14)),
          TextField(
            controller: _newPassword,
            obscureText: true,
            decoration: const InputDecoration(fillColor: Colors.white),
          ),
          const SizedBox(height: 24),
          PrimaryButton(
            label: 'Save',
            onPressed: () async {
              try {
                await apiPut('customer/profile', body: {
                  'first_name': _firstName.text,
                  'last_name': _lastName.text,
                  if (_newPassword.text.isNotEmpty) 'new_password': _newPassword.text,
                });
                if (mounted) {
                  await context.read<AuthProvider>().restore();
                  ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('Your profile has been updated successfully.')));
                }
              } catch (error) {
                if (mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
                }
              }
            },
          ),
        ],
      ),
    );
  }
}
