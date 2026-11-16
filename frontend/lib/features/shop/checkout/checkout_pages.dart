import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Checkout one-page: Address → Shipping → Payment → Review con resumen.
class CheckoutPage extends StatefulWidget {
  const CheckoutPage({super.key});

  @override
  State<CheckoutPage> createState() => _CheckoutPageState();
}

class _CheckoutPageState extends State<CheckoutPage> {
  int _step = 0;
  bool _saving = false;

  final _formKey = GlobalKey<FormState>();
  final _firstName = TextEditingController();
  final _lastName = TextEditingController();
  final _email = TextEditingController();
  final _address = TextEditingController();
  final _city = TextEditingController();
  final _postcode = TextEditingController();
  final _phone = TextEditingController();
  String _country = 'ES';
  final _stateCode = TextEditingController();
  List<Map<String, String>> _countries = const [];
  Map<String, List<Map<String, String>>> _statesByCountry = const {};

  List<ShippingRate> _rates = [];
  List<PaymentMethod> _paymentMethods = [];
  String? _selectedShipping;
  String? _selectedPayment;

  Cart? get cart => context.read<CartProvider>().cart;

  @override
  void initState() {
    super.initState();
    final auth = context.read<AuthProvider>();
    if (auth.isLoggedIn) {
      _firstName.text = auth.customer!.firstName;
      _lastName.text = auth.customer!.lastName;
      _email.text = auth.customer!.email;
    }
    _loadCountriesAndStates();
  }

  /// Listado real de países y provincias (como en el checkout de Bagisto).
  Future<void> _loadCountriesAndStates() async {
    final countries = await apiGet('core/countries');
    final states = await apiGet('core/states');
    if (!mounted) return;
    setState(() {
      if (countries is List) {
        _countries = countries
            .map((e) => Map<String, dynamic>.from(e as Map))
            .map(
              (row) => {
                'code': '${row['code'] ?? ''}',
                'name': '${row['name'] ?? row['code'] ?? ''}',
              },
            )
            .where((row) => row['code']!.isNotEmpty)
            .toList();
      }
      if (states is Map) {
        final grouped = <String, List<Map<String, String>>>{};
        states.forEach((country, list) {
          if (list is! List) return;
          final seen = <String>{};
          final options = <Map<String, String>>[];
          for (final item in list) {
            final row = Map<String, dynamic>.from(item as Map);
            final code = '${row['code'] ?? ''}';
            if (code.isEmpty || !seen.add(code)) continue;
            options.add({
              'code': code,
              'name': '${row['default_name'] ?? code}',
            });
          }
          options.sort((a, b) => a['name']!.compareTo(b['name']!));
          grouped['$country'] = options;
        });
        _statesByCountry = grouped;
      }
    });
  }

  Future<void> _saveAddresses() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _saving = true);
    try {
      await apiPost(
        'checkout/onepage/addresses',
        body: {
          'billing': {
            'first_name': _firstName.text,
            'last_name': _lastName.text,
            'email': _email.text,
            'address': _address.text,
            'country': _country,
            'state': _stateCode.text,
            'city': _city.text,
            'postcode': _postcode.text,
            'phone': _phone.text,
            'use_for_shipping': true,
          },
        },
      );
      await _loadShippingRates();
      if (mounted) setState(() => _step = 1);
    } catch (error) {
      _showError('$error');
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _loadShippingRates() async {
    final data = await apiGet('checkout/onepage/shipping-methods');
    if (data is Map && mounted) {
      setState(() {
        _rates = ((data['shippingMethods'] ?? const []) as List)
            .map((e) => ShippingRate(Map<String, dynamic>.from(e as Map)))
            .toList();
        if (_rates.isNotEmpty) _selectedShipping = _rates.first.method;
      });
    }
  }

  Future<void> _saveShipping() async {
    if (_selectedShipping == null) return;
    setState(() => _saving = true);
    try {
      final data = await apiPost(
        'checkout/onepage/shipping-methods',
        body: {'shipping_method': _selectedShipping},
      );
      if (mounted) {
        // El carrito recalcula el envío: refresca el resumen lateral.
        await context.read<CartProvider>().refresh();
      }
      if (data is Map && mounted) {
        setState(() {
          _paymentMethods = ((data['payment_methods'] ?? const []) as List)
              .map((e) => PaymentMethod(Map<String, dynamic>.from(e as Map)))
              .toList();
          if (_paymentMethods.isNotEmpty) {
            _selectedPayment = _paymentMethods.first.method;
          }
          _step = 2;
        });
      }
    } catch (error) {
      _showError('$error');
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _savePayment() async {
    if (_selectedPayment == null) return;
    setState(() => _saving = true);
    try {
      await apiPost(
        'checkout/onepage/payment-methods',
        body: {'payment_method': _selectedPayment},
      );
      if (mounted) setState(() => _step = 3);
    } catch (error) {
      _showError('$error');
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _placeOrder() async {
    final cartProvider = context.read<CartProvider>();
    setState(() => _saving = true);
    try {
      final data = await apiPost('checkout/onepage/orders', body: {});
      final order = data is Map ? data['order'] : null;
      await cartProvider.refresh();
      if (!mounted) return;
      final incrementId = order is Map
          ? (order['increment_id'] ?? order['id'] ?? '').toString()
          : '';
      context.go('/checkout/success/$incrementId');
    } catch (error) {
      _showError('$error');
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), backgroundColor: AppColors.red600),
    );
  }

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<CartProvider>();
    final cart = provider.cart;
    final width = MediaQuery.of(context).size.width;
    final isNarrow = width < 1060;
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);
    ShippingRate? selectedRate;
    for (final rate in _rates) {
      if (rate.method == _selectedShipping) selectedRate = rate;
    }

    return Scaffold(
      backgroundColor: Colors.white,
      body: Column(
        children: [
          _CheckoutHeader(onBack: () => context.go('/cart')),
          Expanded(
            child: SingleChildScrollView(
              child: Center(
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 1440),
                  child: Padding(
                    padding: EdgeInsets.symmetric(horizontal: horizontal),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const SizedBox(height: 20),
                        Row(
                          children: [
                            GestureDetector(
                              onTap: () => context.go('/'),
                              child: const Text(
                                'Home',
                                style: TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 13,
                                  color: AppColors.zinc500,
                                ),
                              ),
                            ),
                            const Icon(
                              Icons.chevron_right,
                              size: 16,
                              color: AppColors.zinc400,
                            ),
                            GestureDetector(
                              onTap: () => context.go('/cart'),
                              child: const Text(
                                'Shopping Cart',
                                style: TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 13,
                                  color: AppColors.zinc500,
                                ),
                              ),
                            ),
                            const Icon(
                              Icons.chevron_right,
                              size: 16,
                              color: AppColors.zinc400,
                            ),
                            const Text(
                              'Checkout',
                              style: TextStyle(
                                fontFamily: EzTheme.sans,
                                fontSize: 13,
                                color: AppColors.zinc500,
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 12),
                        const Text(
                          'Checkout',
                          style: TextStyle(
                            fontFamily: EzTheme.serif,
                            fontSize: 34,
                            height: 1.15,
                            color: AppColors.navyBlue,
                          ),
                        ),
                        const SizedBox(height: 26),
                        if (cart == null || cart.items.isEmpty)
                          Padding(
                            padding: const EdgeInsets.symmetric(vertical: 90),
                            child: Center(
                              child: Column(
                                children: [
                                  const Icon(
                                    Icons.shopping_bag_outlined,
                                    size: 48,
                                    color: AppColors.zinc300,
                                  ),
                                  const SizedBox(height: 14),
                                  const Text(
                                    'Your cart is empty.',
                                    style: TextStyle(
                                      fontFamily: EzTheme.serif,
                                      fontSize: 24,
                                      color: AppColors.navyBlue,
                                    ),
                                  ),
                                  const SizedBox(height: 18),
                                  PrimaryButton(
                                    label: 'Go Shopping',
                                    onPressed: () => context.go('/'),
                                  ),
                                ],
                              ),
                            ),
                          )
                        else
                          Flex(
                            direction: isNarrow
                                ? Axis.vertical
                                : Axis.horizontal,
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Expanded(
                                flex: isNarrow ? 0 : 2,
                                child: Column(
                                  children: [
                                    _CheckoutStep(
                                      index: 0,
                                      title: '1. Address',
                                      currentStep: _step,
                                      child: _addressForm(),
                                    ),
                                    _CheckoutStep(
                                      index: 1,
                                      title: '2. Shipping Method',
                                      currentStep: _step,
                                      child: Column(
                                        crossAxisAlignment:
                                            CrossAxisAlignment.start,
                                        children: [
                                          _shippingOptions(),
                                          const SizedBox(height: 16),
                                          _saving
                                              ? const CircularProgressIndicator(
                                                  color: AppColors.navyBlue,
                                                )
                                              : PrimaryButton(
                                                  label: 'Save & Continue',
                                                  onPressed: _saveShipping,
                                                ),
                                        ],
                                      ),
                                    ),
                                    _CheckoutStep(
                                      index: 2,
                                      title: '3. Payment Method',
                                      currentStep: _step,
                                      child: Column(
                                        crossAxisAlignment:
                                            CrossAxisAlignment.start,
                                        children: [
                                          _paymentOptions(),
                                          const SizedBox(height: 16),
                                          _saving
                                              ? const CircularProgressIndicator(
                                                  color: AppColors.navyBlue,
                                                )
                                              : PrimaryButton(
                                                  label: 'Save & Continue',
                                                  onPressed: _savePayment,
                                                ),
                                        ],
                                      ),
                                    ),
                                    _CheckoutStep(
                                      index: 3,
                                      title: '4. Review & Place Order',
                                      currentStep: _step,
                                      child: _review(),
                                    ),
                                  ],
                                ),
                              ),
                              SizedBox(
                                width: isNarrow ? 0 : 40,
                                height: isNarrow ? 28 : 0,
                              ),
                              SizedBox(
                                width: isNarrow ? double.infinity : 400,
                                child: Container(
                                  padding: const EdgeInsets.all(24),
                                  decoration: BoxDecoration(
                                    color: AppColors.zinc50,
                                    borderRadius: BorderRadius.circular(
                                      EzTheme.radiusXl,
                                    ),
                                    border: Border.all(
                                      color: AppColors.zinc200,
                                    ),
                                  ),
                                  child: _Summary(
                                    cart: cart,
                                    shippingMethod: _selectedShipping,
                                    shippingRate: selectedRate,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        const SizedBox(height: 64),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  static const _fallbackCountries = <Map<String, String>>[
    {'code': 'ES', 'name': 'Spain'},
    {'code': 'US', 'name': 'United States'},
    {'code': 'FR', 'name': 'France'},
    {'code': 'DE', 'name': 'Germany'},
    {'code': 'GB', 'name': 'United Kingdom'},
    {'code': 'MX', 'name': 'Mexico'},
    {'code': 'AR', 'name': 'Argentina'},
  ];

  Widget _addressForm() {
    final states = _statesByCountry[_country] ?? const <Map<String, String>>[];
    return Form(
      key: _formKey,
      child: Column(
        children: [
          Row(
            children: [
              Expanded(
                child: _field('First name', _firstName, requiredField: true),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: _field('Last name', _lastName, requiredField: true),
              ),
            ],
          ),
          const SizedBox(height: 12),
          _field('Email', _email, requiredField: true),
          const SizedBox(height: 12),
          _field('Address', _address, requiredField: true),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Country',
                      style: TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: AppColors.gray900,
                      ),
                    ),
                    const SizedBox(height: 7),
                    DropdownButtonFormField<String>(
                      initialValue: _country,
                      isExpanded: true,
                      decoration: const InputDecoration(
                        fillColor: Colors.white,
                      ),
                      items: [
                        for (final country
                            in _countries.isEmpty
                                ? _fallbackCountries
                                : _countries)
                          DropdownMenuItem(
                            value: country['code'],
                            child: Text(
                              country['name']!,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                      ],
                      onChanged: (value) => setState(() {
                        _country = value ?? 'ES';
                        _stateCode.clear();
                      }),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'State',
                      style: TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: AppColors.gray900,
                      ),
                    ),
                    const SizedBox(height: 7),
                    if (states.isEmpty)
                      _field(
                        'State',
                        _stateCode,
                        requiredField: false,
                        hideLabel: true,
                      )
                    else
                      DropdownButtonFormField<String>(
                        key: ValueKey('state-$_country'),
                        initialValue:
                            states.any(
                              (state) => state['code'] == _stateCode.text,
                            )
                            ? _stateCode.text
                            : null,
                        isExpanded: true,
                        decoration: const InputDecoration(
                          fillColor: Colors.white,
                          hintText: 'State',
                        ),
                        items: [
                          for (final state in states)
                            DropdownMenuItem(
                              value: state['code'],
                              child: Text(
                                state['name']!,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                        ],
                        onChanged: (value) =>
                            setState(() => _stateCode.text = value ?? ''),
                      ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(child: _field('City', _city, requiredField: true)),
              const SizedBox(width: 12),
              Expanded(
                child: _field('Postcode', _postcode, requiredField: true),
              ),
            ],
          ),
          const SizedBox(height: 12),
          _field('Phone', _phone, requiredField: false),
          const SizedBox(height: 16),
          _saving
              ? const CircularProgressIndicator(color: AppColors.navyBlue)
              : PrimaryButton(
                  label: 'Save & Continue',
                  onPressed: _saveAddresses,
                ),
        ],
      ),
    );
  }

  Widget _field(
    String hint,
    TextEditingController controller, {
    bool requiredField = true,
    bool hideLabel = false,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (!hideLabel) ...[
          Text(
            hint,
            style: const TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 13,
              fontWeight: FontWeight.w500,
              color: AppColors.gray900,
            ),
          ),
          const SizedBox(height: 7),
        ],
        TextFormField(
          controller: controller,
          decoration: InputDecoration(hintText: hint, fillColor: Colors.white),
          validator: requiredField
              ? (value) => (value ?? '').trim().isEmpty
                    ? 'This field is required.'
                    : null
              : null,
        ),
      ],
    );
  }

  Widget _shippingOptions() {
    if (_rates.isEmpty) {
      return const Text('No shipping methods available.');
    }
    return Wrap(
      spacing: 16,
      runSpacing: 16,
      children: _rates.map((rate) {
        return GestureDetector(
          onTap: () => setState(() => _selectedShipping = rate.method),
          child: Container(
            width: 220,
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              border: Border.all(
                color: _selectedShipping == rate.method
                    ? AppColors.navyBlue
                    : AppColors.zinc200,
                width: 2,
              ),
            ),
            child: Row(
              children: [
                const Icon(
                  Icons.local_shipping_outlined,
                  size: 32,
                  color: AppColors.navyBlue,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        rate.methodTitle,
                        style: const TextStyle(
                          fontFamily: 'Poppins',
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                      Text(
                        '€${rate.price.toStringAsFixed(2)}',
                        style: const TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 17,
                          fontWeight: FontWeight.w600,
                          color: AppColors.gray900,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      }).toList(),
    );
  }

  Widget _paymentOptions() {
    return RadioGroup<String>(
      groupValue: _selectedPayment,
      onChanged: (value) => setState(() => _selectedPayment = value),
      child: Column(
        children: _paymentMethods
            .map(
              (method) => RadioListTile<String>(
                value: method.method,
                title: Text(
                  method.methodTitle,
                  style: const TextStyle(fontFamily: 'Poppins', fontSize: 15),
                ),
                subtitle: Text(
                  method.description,
                  style: TextStyle(
                    fontFamily: 'Poppins',
                    fontSize: 12,
                    color: AppColors.zinc500,
                  ),
                ),
              ),
            )
            .toList(),
      ),
    );
  }

  Widget _reviewBlock({
    required String title,
    required List<String> lines,
    required int step,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 14),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: const TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                    color: AppColors.zinc500,
                  ),
                ),
                const SizedBox(height: 5),
                for (final line in lines)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 2),
                    child: Text(
                      line,
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 14,
                        height: 1.45,
                        color: AppColors.gray900,
                      ),
                    ),
                  ),
              ],
            ),
          ),
          TextButton(
            onPressed: () => setState(() => _step = step),
            style: TextButton.styleFrom(
              foregroundColor: AppColors.blue700,
              padding: EdgeInsets.zero,
              minimumSize: const Size(0, 0),
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            child: const Text(
              'Edit',
              style: TextStyle(fontFamily: EzTheme.sans, fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }

  Widget _review() {
    ShippingRate? rate;
    for (final item in _rates) {
      if (item.method == _selectedShipping) rate = item;
    }
    PaymentMethod? payment;
    for (final item in _paymentMethods) {
      if (item.method == _selectedPayment) payment = item;
    }
    final name = '${_firstName.text} ${_lastName.text}'.trim();
    final cityLine = '${_city.text} ${_postcode.text}'.trim();
    final addressLines = <String>[
      if (name.isNotEmpty) name,
      if (_address.text.trim().isNotEmpty) _address.text.trim(),
      if (cityLine.isNotEmpty) cityLine,
      if (_country.trim().isNotEmpty) _country,
      if (_phone.text.trim().isNotEmpty) _phone.text.trim(),
    ];
    final shippingLines = <String>[
      rate == null
          ? (_selectedShipping ?? '—')
          : '${rate.carrierTitle} — €${rate.price.toStringAsFixed(2)}',
    ];
    final paymentLines = <String>[
      payment?.methodTitle ?? _selectedPayment ?? '—',
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 4),
          decoration: BoxDecoration(
            color: AppColors.zinc50,
            borderRadius: BorderRadius.circular(EzTheme.radiusXl),
            border: Border.all(color: AppColors.zinc200),
          ),
          child: Column(
            children: [
              _reviewBlock(
                title: 'Shipping Address',
                lines: addressLines,
                step: 0,
              ),
              const Divider(height: 1, color: AppColors.zinc200),
              _reviewBlock(
                title: 'Shipping Method',
                lines: shippingLines,
                step: 1,
              ),
              const Divider(height: 1, color: AppColors.zinc200),
              _reviewBlock(
                title: 'Payment Method',
                lines: paymentLines,
                step: 2,
              ),
            ],
          ),
        ),
        const SizedBox(height: 20),
        PrimaryButton(
          label: 'Place Order',
          height: 52,
          isLoading: _saving,
          onPressed: _saving ? null : _placeOrder,
        ),
      ],
    );
  }
}

class _CheckoutHeader extends StatelessWidget {
  const _CheckoutHeader({required this.onBack});
  final VoidCallback onBack;

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.of(context).size.width;
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);
    return Container(
      height: 76,
      padding: EdgeInsets.symmetric(horizontal: horizontal),
      decoration: const BoxDecoration(
        color: Colors.white,
        border: Border(bottom: BorderSide(color: AppColors.zinc200)),
      ),
      child: Row(
        children: [
          GestureDetector(
            onTap: () => context.go('/'),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.baseline,
              textBaseline: TextBaseline.alphabetic,
              children: const [
                Text(
                  'EzCommerce',
                  style: TextStyle(
                    fontFamily: EzTheme.serif,
                    fontSize: 26,
                    fontWeight: FontWeight.w700,
                    color: AppColors.navyBlue,
                  ),
                ),
                Text(
                  '.',
                  style: TextStyle(
                    fontFamily: EzTheme.serif,
                    fontSize: 26,
                    fontWeight: FontWeight.w700,
                    color: AppColors.red600,
                  ),
                ),
              ],
            ),
          ),
          const Spacer(),
          TextButton.icon(
            onPressed: onBack,
            icon: const Icon(Icons.arrow_back, size: 16),
            label: Text(
              width >= 560 ? 'Back to cart' : 'Cart',
              style: const TextStyle(fontFamily: EzTheme.sans, fontSize: 13),
            ),
          ),
          if (width >= 720) ...[
            const SizedBox(width: 18),
            const Row(
              children: [
                Icon(Icons.lock_outline, size: 16, color: AppColors.zinc500),
                SizedBox(width: 6),
                Text(
                  'Secure checkout',
                  style: TextStyle(
                    fontFamily: EzTheme.sans,
                    fontSize: 13,
                    color: AppColors.zinc500,
                  ),
                ),
              ],
            ),
          ],
        ],
      ),
    );
  }
}

class _CheckoutStep extends StatelessWidget {
  const _CheckoutStep({
    required this.index,
    required this.title,
    required this.currentStep,
    required this.child,
  });

  final int index;
  final String title;
  final int currentStep;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final active = currentStep == index;
    final done = currentStep > index;
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(
          color: active ? AppColors.navyBlue : AppColors.zinc200,
        ),
      ),
      child: Theme(
        data: Theme.of(context).copyWith(dividerColor: Colors.transparent),
        child: ExpansionTile(
          key: ValueKey('checkout-step-$index-$active'),
          initiallyExpanded: active,
          maintainState: true,
          onExpansionChanged: (expanded) {},
          title: Row(
            children: [
              if (done)
                const Icon(
                  Icons.check_circle,
                  color: AppColors.green600,
                  size: 20,
                ),
              const SizedBox(width: 8),
              Text(
                title,
                style: TextStyle(
                  fontFamily: 'Poppins',
                  fontSize: 18,
                  fontWeight: active ? FontWeight.w600 : FontWeight.w400,
                ),
              ),
            ],
          ),
          children: [Padding(padding: const EdgeInsets.all(16), child: child)],
        ),
      ),
    );
  }
}

class _Summary extends StatelessWidget {
  const _Summary({required this.cart, this.shippingMethod, this.shippingRate});
  final Cart cart;
  final String? shippingMethod;
  final ShippingRate? shippingRate;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Order Summary',
          style: TextStyle(
            fontFamily: EzTheme.serif,
            fontSize: 22,
            color: AppColors.navyBlue,
          ),
        ),
        const SizedBox(height: 18),
        ...cart.items.map(
          (item) => Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Row(
              children: [
                ClipRRect(
                  borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                  child: SizedBox(
                    width: 56,
                    height: 56,
                    child: Container(
                      color: AppColors.zinc100,
                      child: apiUrlRelative(item.imageUrl) != null
                          ? Image.network(
                              apiUrlRelative(item.imageUrl)!,
                              fit: BoxFit.cover,
                              errorBuilder: (_, _, _) => const Icon(
                                Icons.image_outlined,
                                color: AppColors.zinc300,
                                size: 22,
                              ),
                            )
                          : const Icon(
                              Icons.image_outlined,
                              color: AppColors.zinc300,
                              size: 22,
                            ),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        item.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 14,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                      Text(
                        '${item.formattedPrice} × ${item.quantity.toInt()}',
                        style: TextStyle(
                          fontFamily: 'Poppins',
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
        ),
        const Divider(),
        _row('Sub Total', cart.formattedSubTotal),
        if (cart.formattedDiscount.isNotEmpty && cart.formattedDiscount != '0')
          _row('Discount', '-${cart.formattedDiscount}'),
        _row('Tax', cart.formattedTaxTotal),
        if (shippingMethod != null || cart.shippingMethod != null)
          _row(
            'Shipping',
            '€${(shippingMethod != null ? (shippingRate?.price ?? cart.shippingAmount) : cart.shippingAmount).toStringAsFixed(2)}',
          ),
        const Divider(),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text(
              'Grand Total',
              style: TextStyle(
                fontFamily: 'Poppins',
                fontWeight: FontWeight.w600,
              ),
            ),
            Text(
              '€${(cart.grandTotal - cart.shippingAmount + (shippingMethod != null ? (shippingRate?.price ?? cart.shippingAmount) : cart.shippingAmount)).toStringAsFixed(2)}',
              style: const TextStyle(
                fontFamily: 'Poppins',
                fontSize: 17,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _row(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            label,
            style: TextStyle(
              fontFamily: 'Poppins',
              fontSize: 13,
              color: AppColors.zinc600,
            ),
          ),
          Text(
            value,
            style: const TextStyle(fontFamily: 'Poppins', fontSize: 13),
          ),
        ],
      ),
    );
  }
}

/// Pantalla de éxito (thank-you + nº pedido).
class CheckoutSuccessPage extends StatelessWidget {
  const CheckoutSuccessPage({super.key, required this.incrementId});
  final String incrementId;

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.of(context).size.width;
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);
    return Scaffold(
      backgroundColor: Colors.white,
      body: SingleChildScrollView(
        child: Column(
          children: [
            Container(
              height: 76,
              padding: EdgeInsets.symmetric(horizontal: horizontal),
              decoration: const BoxDecoration(
                border: Border(bottom: BorderSide(color: AppColors.zinc200)),
              ),
              child: Row(
                children: const [
                  Text(
                    'EzCommerce',
                    style: TextStyle(
                      fontFamily: EzTheme.serif,
                      fontSize: 26,
                      fontWeight: FontWeight.w700,
                      color: AppColors.navyBlue,
                    ),
                  ),
                  Text(
                    '.',
                    style: TextStyle(
                      fontFamily: EzTheme.serif,
                      fontSize: 26,
                      fontWeight: FontWeight.w700,
                      color: AppColors.red600,
                    ),
                  ),
                ],
              ),
            ),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: horizontal),
              child: Center(
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 640),
                  child: Padding(
                    padding: EdgeInsets.symmetric(vertical: width < 640 ? 48 : 80),
                    child: Column(
                      children: [
                        Container(
                          width: 88,
                          height: 88,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            color: AppColors.emerald500.withValues(alpha: .12),
                          ),
                          child: const Icon(
                            Icons.check_rounded,
                            size: 44,
                            color: AppColors.emerald600,
                          ),
                        ),
                        const SizedBox(height: 26),
                        Text(
                          'Thank you for your order',
                          textAlign: TextAlign.center,
                          style: TextStyle(
                            fontFamily: EzTheme.serif,
                            fontSize: width < 640 ? 28 : 34,
                            height: 1.15,
                            color: AppColors.navyBlue,
                          ),
                        ),
                        const SizedBox(height: 12),
                        Text(
                          'Your order #$incrementId has been placed successfully. '
                          'We have emailed a confirmation with all the details.',
                          textAlign: TextAlign.center,
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 15,
                            height: 1.6,
                            color: AppColors.zinc600,
                          ),
                        ),
                        const SizedBox(height: 34),
                        Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 24,
                            vertical: 20,
                          ),
                          decoration: BoxDecoration(
                            color: AppColors.zinc50,
                            borderRadius: BorderRadius.circular(
                              EzTheme.radiusXl,
                            ),
                            border: Border.all(color: AppColors.zinc200),
                          ),
                          child: Row(
                            children: [
                              const Icon(
                                Icons.receipt_long_outlined,
                                size: 22,
                                color: AppColors.navyBlue,
                              ),
                              const SizedBox(width: 12),
                              const Expanded(
                                child: Text(
                                  'Order number',
                                  style: TextStyle(
                                    fontFamily: EzTheme.sans,
                                    fontSize: 14,
                                    color: AppColors.zinc600,
                                  ),
                                ),
                              ),
                              Text(
                                '#$incrementId',
                                style: const TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 15,
                                  fontWeight: FontWeight.w600,
                                  color: AppColors.navyBlue,
                                ),
                              ),
                            ],
                          ),
                        ),
                        const SizedBox(height: 30),
                        if (width < 640)
                          Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              PrimaryButton(
                                label: 'Continue Shopping',
                                expand: true,
                                onPressed: () => context.go('/'),
                              ),
                              const SizedBox(height: 12),
                              SecondaryButton(
                                label: 'View My Orders',
                                expand: true,
                                onPressed: () => context.go('/account/orders'),
                              ),
                            ],
                          )
                        else
                          Wrap(
                            spacing: 14,
                            runSpacing: 12,
                            alignment: WrapAlignment.center,
                            children: [
                              PrimaryButton(
                                label: 'Continue Shopping',
                                onPressed: () => context.go('/'),
                              ),
                              SecondaryButton(
                                label: 'View My Orders',
                                onPressed: () => context.go('/account/orders'),
                              ),
                            ],
                          ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
