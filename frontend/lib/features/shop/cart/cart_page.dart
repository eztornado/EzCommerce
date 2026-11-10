import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../product/product_page.dart' show QuantityChanger;

/// Página de carrito: items con qty-changer, cupón y Cart Summary.
class CartPage extends StatefulWidget {
  const CartPage({super.key});

  @override
  State<CartPage> createState() => _CartPageState();
}

class _CartPageState extends State<CartPage> {
  final _couponController = TextEditingController();

  @override
  Widget build(BuildContext context) {
    final cart = context.watch<CartProvider>();
    final isNarrow = MediaQuery.of(context).size.width < 1060;

    if (cart.cart == null || cart.cart!.items.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.shopping_cart_outlined, size: 56, color: AppColors.zinc300),
            const SizedBox(height: 12),
            const Text('Your shopping cart is empty.',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 18)),
            const SizedBox(height: 20),
            PrimaryButton(label: 'Continue Shopping', onPressed: () => context.go('/')),
          ],
        ),
      );
    }

    final items = cart.cart!.items;
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        children: [
          Flex(
            direction: isNarrow ? Axis.vertical : Axis.horizontal,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Items
              Expanded(
                flex: isNarrow ? 0 : 2,
                child: Column(
                  children: [
                    ...items.map((item) => _CartItemRow(
                          item: item,
                          onQuantity: (value) =>
                              cart.updateQuantity(item.id ?? 0, value),
                          onRemove: () => cart.removeItem(item.id ?? 0),
                        )),
                    const SizedBox(height: 24),
                    // Cupón
                    Row(
                      children: [
                        Expanded(
                          child: TextField(
                            controller: _couponController,
                            decoration: const InputDecoration(
                              hintText: 'Coupon code',
                              fillColor: Colors.white,
                              border: OutlineInputBorder(),
                            ),
                          ),
                        ),
                        const SizedBox(width: 12),
                        TextButton(
                          onPressed: () async {
                            final error = await cart.applyCoupon(_couponController.text);
                            if (context.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                                content: Text(error ?? 'The coupon code has been applied successfully.'),
                                backgroundColor: error != null ? AppColors.red600 : AppColors.navyBlue,
                              ));
                            }
                          },
                          style: TextButton.styleFrom(
                            backgroundColor: AppColors.navyBlue,
                            foregroundColor: Colors.white,
                            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
                          ),
                          child: const Text('Apply Coupon'),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 48, height: 32),
              // Summary
              Container(
                constraints: const BoxConstraints(maxWidth: 380),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                  border: Border.all(color: AppColors.zinc200),
                ),
                padding: const EdgeInsets.all(24),
                child: _CartSummary(cart: cart.cart!),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _CartItemRow extends StatelessWidget {
  const _CartItemRow({required this.item, required this.onQuantity, required this.onRemove});
  final CartItem item;
  final ValueChanged<int> onQuantity;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 16),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: AppColors.zinc200)),
      ),
      child: Row(
        children: [
          // Imagen
          Container(
            width: 80,
            height: 80,
            decoration: BoxDecoration(
              color: AppColors.zinc100,
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
            ),
            child: const Icon(Icons.image_outlined, color: AppColors.zinc300),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(item.name,
                    style: const TextStyle(
                        fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w500)),
                const SizedBox(height: 4),
                Text('SKU: ${item.sku}',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                const SizedBox(height: 8),
                QuantityChanger(
                  quantity: item.quantity.toInt(),
                  onChanged: onQuantity,
                ),
              ],
            ),
          ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Text(item.formattedTotal,
                  style: const TextStyle(
                      fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w600)),
              IconButton(
                icon: const Icon(Icons.delete_outline, size: 20, color: AppColors.zinc500),
                onPressed: onRemove,
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _CartSummary extends StatelessWidget {
  const _CartSummary({required this.cart});
  final Cart cart;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Cart Summary',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 22, fontWeight: FontWeight.w500)),
        const SizedBox(height: 16),
        _row('Sub Total', cart.formattedSubTotal),
        if (cart.formattedDiscount.isNotEmpty && cart.formattedDiscount != '0')
          _row('Discount', '-${cart.formattedDiscount}', color: AppColors.red600),
        _row('Tax', cart.formattedTaxTotal),
        if (cart.shippingMethod != null) _row('Shipping', cart.formattedShipping),
        const Divider(height: 24),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text('Grand Total',
                style: TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w600)),
            Text(cart.formattedGrandTotal,
                style: const TextStyle(
                    fontFamily: 'Poppins', fontSize: 18, fontWeight: FontWeight.w600)),
          ],
        ),
        const SizedBox(height: 24),
        PrimaryButton(
          label: 'Proceed to Checkout',
          expand: true,
          onPressed: () => context.go('/checkout'),
        ),
        const SizedBox(height: 12),
        SecondaryButton(label: 'Continue Shopping', expand: true, onPressed: () => context.go('/')),
      ],
    );
  }

  Widget _row(String label, String value, {Color? color}) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: color ?? AppColors.zinc600)),
          Text(value,
              style: TextStyle(fontFamily: 'Poppins', fontSize: 14, fontWeight: FontWeight.w500, color: color)),
        ],
      ),
    );
  }
}
