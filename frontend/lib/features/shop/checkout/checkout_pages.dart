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
  final _stateCode = TextEditingController(text: 'MD');

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
  }

  Future<void> _saveAddresses() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _saving = true);
    try {
      await apiPost('checkout/onepage/addresses', body: {
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
      });
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
      final data = await apiPost('checkout/onepage/shipping-methods',
          body: {'shipping_method': _selectedShipping});
      if (data is Map && mounted) {
        setState(() {
          _paymentMethods = ((data['payment_methods'] ?? const []) as List)
              .map((e) => PaymentMethod(Map<String, dynamic>.from(e as Map)))
              .toList();
          if (_paymentMethods.isNotEmpty) _selectedPayment = _paymentMethods.first.method;
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
      await apiPost('checkout/onepage/payment-methods', body: {'payment_method': _selectedPayment});
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
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(
        content: Text(message), backgroundColor: AppColors.red600));
  }

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<CartProvider>();
    final cart = provider.cart;
    final isNarrow = MediaQuery.of(context).size.width < 1060;

    if (cart == null || cart.items.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text('Your cart is empty.',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 18)),
            const SizedBox(height: 16),
            PrimaryButton(label: 'Go Shopping', onPressed: () => context.go('/')),
          ],
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(title: const Text('Checkout')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Flex(
          direction: isNarrow ? Axis.vertical : Axis.horizontal,
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
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        _shippingOptions(),
                        const SizedBox(height: 16),
                        _saving
                            ? const CircularProgressIndicator(color: AppColors.navyBlue)
                            : PrimaryButton(label: 'Save & Continue', onPressed: _saveShipping),
                      ],
                    ),
                  ),
                  _CheckoutStep(
                    index: 2,
                    title: '3. Payment Method',
                    currentStep: _step,
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        _paymentOptions(),
                        const SizedBox(height: 16),
                        _saving
                            ? const CircularProgressIndicator(color: AppColors.navyBlue)
                            : PrimaryButton(label: 'Save & Continue', onPressed: _savePayment),
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
            const SizedBox(width: 32, height: 24),
            Container(
              constraints: const BoxConstraints(maxWidth: 360),
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                border: Border.all(color: AppColors.zinc200),
              ),
              child: _Summary(cart: cart),
            ),
          ],
        ),
      ),
    );
  }

  Widget _addressForm() {
    return Form(
      key: _formKey,
      child: Column(
        children: [
          Row(children: [
            Expanded(child: _field('First name', _firstName, requiredField: true)),
            const SizedBox(width: 12),
            Expanded(child: _field('Last name', _lastName, requiredField: true)),
          ]),
          const SizedBox(height: 12),
          _field('Email', _email, requiredField: true),
          const SizedBox(height: 12),
          _field('Address', _address, requiredField: true),
          const SizedBox(height: 12),
          Row(children: [
            Expanded(
              child: DropdownButtonFormField<String>(
                initialValue: _country,
                decoration: const InputDecoration(hintText: 'Country'),
                items: const [
                  DropdownMenuItem(value: 'ES', child: Text('Spain')),
                  DropdownMenuItem(value: 'US', child: Text('United States')),
                  DropdownMenuItem(value: 'FR', child: Text('France')),
                  DropdownMenuItem(value: 'DE', child: Text('Germany')),
                  DropdownMenuItem(value: 'GB', child: Text('United Kingdom')),
                  DropdownMenuItem(value: 'MX', child: Text('Mexico')),
                  DropdownMenuItem(value: 'AR', child: Text('Argentina')),
                ],
                onChanged: (value) => setState(() => _country = value ?? 'ES'),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(child: _field('State', _stateCode, requiredField: false)),
          ]),
          const SizedBox(height: 12),
          Row(children: [
            Expanded(child: _field('City', _city, requiredField: true)),
            const SizedBox(width: 12),
            Expanded(child: _field('Postcode', _postcode, requiredField: true)),
          ]),
          const SizedBox(height: 12),
          _field('Phone', _phone, requiredField: false),
          const SizedBox(height: 16),
          _saving
              ? const CircularProgressIndicator(color: AppColors.navyBlue)
              : PrimaryButton(label: 'Save & Continue', onPressed: _saveAddresses),
        ],
      ),
    );
  }

  Widget _field(String hint, TextEditingController controller, {bool requiredField = true}) {
    return TextFormField(
      controller: controller,
      decoration: InputDecoration(hintText: hint, fillColor: Colors.white,
          border: const OutlineInputBorder()),
      validator: requiredField
          ? (value) => (value ?? '').trim().isEmpty ? 'This field is required.' : null
          : null,
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
                color: _selectedShipping == rate.method ? AppColors.navyBlue : AppColors.zinc200,
                width: 2,
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.local_shipping_outlined, size: 32, color: AppColors.navyBlue),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(rate.methodTitle,
                          style: const TextStyle(fontFamily: 'Poppins', fontWeight: FontWeight.w500)),
                      Text('\$${rate.price}',
                          style: const TextStyle(
                              fontFamily: 'Poppins', fontSize: 18, fontWeight: FontWeight.w600)),
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
            .map((method) => RadioListTile<String>(
                  value: method.method,
                  title: Text(method.methodTitle,
                      style: const TextStyle(fontFamily: 'Poppins', fontSize: 15)),
                  subtitle: Text(method.description,
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                ))
            .toList(),
      ),
    );
  }

  Widget _review() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Shipping: ${_selectedShipping ?? '-'}',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.zinc600)),
        Text('Payment: ${_selectedPayment ?? '-'}',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.zinc600)),
        const SizedBox(height: 16),
        PrimaryButton(
          label: 'Place Order',
          isLoading: _saving,
          onPressed: _saving ? null : _placeOrder,
        ),
      ],
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
        border: Border.all(color: active ? AppColors.navyBlue : AppColors.zinc200),
      ),
      child: Theme(
        data: Theme.of(context).copyWith(dividerColor: Colors.transparent),
        child: ExpansionTile(
          initiallyExpanded: active,
          maintainState: true,
          onExpansionChanged: (expanded) {},
          title: Row(
            children: [
              if (done) const Icon(Icons.check_circle, color: AppColors.green600, size: 20),
              const SizedBox(width: 8),
              Text(title,
                  style: TextStyle(
                      fontFamily: 'Poppins',
                      fontSize: 18,
                      fontWeight: active ? FontWeight.w600 : FontWeight.w400)),
            ],
          ),
          children: [Padding(padding: const EdgeInsets.all(16), child: child)],
        ),
      ),
    );
  }
}

class _Summary extends StatelessWidget {
  const _Summary({required this.cart});
  final Cart cart;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ...cart.items.map((item) => Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Row(
                children: [
                  Container(
                    width: 56,
                    height: 56,
                    decoration: BoxDecoration(
                      color: AppColors.zinc100,
                      borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                    ),
                    child: const Icon(Icons.image_outlined, color: AppColors.zinc300, size: 22),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(item.name,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(
                                fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500)),
                        Text('${item.formattedPrice} × ${item.quantity.toInt()}',
                            style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                      ],
                    ),
                  ),
                ],
              ),
            )),
        const Divider(),
        _row('Sub Total', cart.formattedSubTotal),
        _row('Tax', cart.formattedTaxTotal),
        if (cart.shippingMethod != null) _row('Shipping', cart.formattedShipping),
        const Divider(),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text('Grand Total',
                style: TextStyle(fontFamily: 'Poppins', fontWeight: FontWeight.w600)),
            Text(cart.formattedGrandTotal,
                style: const TextStyle(
                    fontFamily: 'Poppins', fontSize: 17, fontWeight: FontWeight.w600)),
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
          Text(label, style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc600)),
          Text(value, style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
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
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.check_circle_outline, size: 72, color: AppColors.green600),
            const SizedBox(height: 16),
            const Text('Thanks for your order!',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w500)),
            const SizedBox(height: 8),
            Text('Order #$incrementId',
                style: const TextStyle(
                    fontFamily: 'Poppins', fontSize: 16, color: AppColors.blue700)),
            const SizedBox(height: 8),
            Text(
              'We will email you an order confirmation with details.',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.zinc500),
            ),
            const SizedBox(height: 24),
            PrimaryButton(label: 'Continue Shopping', onPressed: () => context.go('/')),
          ],
        ),
      ),
    );
  }
}
