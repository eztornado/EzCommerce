import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../product/product_page.dart' show QuantityChanger;
import '../shell/shop_shell.dart';

/// Carrito (Bagisto): items con imagen, qty y subtotal, cupón y resumen.
class CartPage extends StatefulWidget {
  const CartPage({super.key});

  @override
  State<CartPage> createState() => _CartPageState();
}

class _CartPageState extends State<CartPage> {
  final _couponController = TextEditingController();
  bool _couponLoading = false;

  @override
  void dispose() {
    _couponController.dispose();
    super.dispose();
  }

  Future<void> _applyCoupon(CartProvider cart) async {
    setState(() => _couponLoading = true);
    final error = await cart.applyCoupon(_couponController.text.trim());
    if (!mounted) return;
    setState(() => _couponLoading = false);
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          error ?? 'The coupon code has been applied successfully.',
        ),
        backgroundColor: error != null ? AppColors.red600 : AppColors.navyBlue,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final cart = context.watch<CartProvider>();
    final width = MediaQuery.of(context).size.width;
    final isNarrow = width < 1060;
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);

    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1440),
              child: Padding(
                padding: EdgeInsets.symmetric(
                  horizontal: horizontal,
                  vertical: 32,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        GestureDetector(
                          onTap: () => context.go('/'),
                          child: const Text(
                            'Home',
                            style: TextStyle(
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
                          'Shopping Cart',
                          style: TextStyle(
                            fontSize: 13,
                            color: AppColors.zinc500,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    const Text(
                      'Shopping Cart',
                      style: TextStyle(
                        fontFamily: EzTheme.serif,
                        fontSize: 36,
                        color: AppColors.navyBlue,
                      ),
                    ),
                    const SizedBox(height: 28),
                    if (cart.cart == null || cart.cart!.items.isEmpty)
                      _EmptyCart(onContinue: () => context.go('/'))
                    else
                      isNarrow
                          ? Column(
                              children: [
                                _CartItems(
                                  cart: cart,
                                  coupon: _couponBlock(cart),
                                ),
                                const SizedBox(height: 28),
                                _CartSummary(cart: cart.cart!),
                              ],
                            )
                          : Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Expanded(
                                  flex: 2,
                                  child: _CartItems(
                                    cart: cart,
                                    coupon: _couponBlock(cart),
                                  ),
                                ),
                                const SizedBox(width: 44),
                                SizedBox(
                                  width: 400,
                                  child: _CartSummary(cart: cart.cart!),
                                ),
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

  Widget _couponBlock(CartProvider cart) {
    return Container(
      margin: const EdgeInsets.only(top: 24),
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Apply Coupon Code',
            style: TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 16,
              fontWeight: FontWeight.w500,
              color: AppColors.gray900,
            ),
          ),
          const SizedBox(height: 14),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _couponController,
                  decoration: const InputDecoration(
                    hintText: 'Enter coupon code',
                    fillColor: Colors.white,
                    border: OutlineInputBorder(),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              PrimaryButton(
                label: 'Apply',
                height: 52,
                isLoading: _couponLoading,
                onPressed: _couponLoading ? null : () => _applyCoupon(cart),
              ),
            ],
          ),
          if (cart.cart?.couponCode != null) ...[
            const SizedBox(height: 12),
            Text(
              'Applied coupon: ${cart.cart!.couponCode}',
              style: const TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 13,
                color: AppColors.emerald600,
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _CartItems extends StatelessWidget {
  const _CartItems({required this.cart, required this.coupon});
  final CartProvider cart;
  final Widget coupon;

  @override
  Widget build(BuildContext context) {
    final items = cart.cart!.items;
    return Column(
      children: [
        Container(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(EzTheme.radiusXl),
            border: Border.all(color: AppColors.zinc200),
          ),
          child: Column(
            children: [
              for (var i = 0; i < items.length; i++) ...[
                _CartItemRow(
                  item: items[i],
                  onQuantity: (value) =>
                      cart.updateQuantity(items[i].id ?? 0, value),
                  onRemove: () => cart.removeItem(items[i].id ?? 0),
                ),
                if (i != items.length - 1)
                  const Divider(height: 1, color: AppColors.zinc200),
              ],
            ],
          ),
        ),
        coupon,
      ],
    );
  }
}

class _CartItemRow extends StatelessWidget {
  const _CartItemRow({
    required this.item,
    required this.onQuantity,
    required this.onRemove,
  });

  final CartItem item;
  final ValueChanged<int> onQuantity;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    final image = apiUrlRelative(item.imageUrl);
    final thumb = ClipRRect(
      borderRadius: BorderRadius.circular(EzTheme.radiusLg),
      child: Container(
        color: AppColors.zinc100,
        child: image != null
            ? Image.network(
                image,
                fit: BoxFit.cover,
                errorBuilder: (_, _, _) => const Icon(
                  Icons.image_outlined,
                  size: 32,
                  color: AppColors.zinc300,
                ),
              )
            : const Icon(
                Icons.image_outlined,
                size: 32,
                color: AppColors.zinc300,
              ),
      ),
    );
    final name = Text(
      item.name,
      style: const TextStyle(
        fontFamily: EzTheme.sans,
        fontSize: 16,
        fontWeight: FontWeight.w500,
        color: AppColors.gray900,
      ),
    );
    final sku = Text(
      'SKU: ${item.sku}',
      style: const TextStyle(
        fontFamily: EzTheme.sans,
        fontSize: 12,
        color: AppColors.zinc500,
      ),
    );
    final remove = TextButton.icon(
      onPressed: onRemove,
      style: TextButton.styleFrom(
        foregroundColor: AppColors.blue700,
        padding: EdgeInsets.zero,
        minimumSize: const Size(0, 0),
        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
      ),
      icon: const Icon(Icons.delete_outline, size: 15),
      label: const Text(
        'Remove',
        style: TextStyle(fontFamily: EzTheme.sans, fontSize: 12),
      ),
    );
    final total = Text(
      item.formattedTotal,
      style: const TextStyle(
        fontFamily: EzTheme.sans,
        fontSize: 16,
        fontWeight: FontWeight.w600,
        color: AppColors.gray900,
      ),
    );

    return LayoutBuilder(
      builder: (context, constraints) {
        final narrow = constraints.maxWidth < 520;
        if (narrow) {
          return Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SizedBox(width: 92, height: 104, child: thumb),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Expanded(child: name),
                          const SizedBox(width: 8),
                          total,
                        ],
                      ),
                      const SizedBox(height: 5),
                      sku,
                      if (item.additional != null &&
                          item.additional!.isNotEmpty) ...[
                        const SizedBox(height: 4),
                        Text(
                          item.additional!.entries
                              .map((entry) => '${entry.key}: ${entry.value}')
                              .join('  ·  '),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 11,
                            color: AppColors.zinc500,
                          ),
                        ),
                      ],
                      const SizedBox(height: 10),
                      Row(
                        children: [
                          QuantityChanger(
                            quantity: item.quantity.toInt(),
                            onChanged: onQuantity,
                          ),
                          const Spacer(),
                          remove,
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
          );
        }
        return Padding(
          padding: const EdgeInsets.all(20),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SizedBox(width: 112, height: 120, child: thumb),
              const SizedBox(width: 20),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    name,
                    const SizedBox(height: 5),
                    sku,
                    if (item.additional != null &&
                        item.additional!.isNotEmpty) ...[
                      const SizedBox(height: 6),
                      Text(
                        item.additional!.entries
                            .map((entry) => '${entry.key}: ${entry.value}')
                            .join('  ·  '),
                        style: const TextStyle(
                          fontFamily: EzTheme.sans,
                          fontSize: 12,
                          color: AppColors.zinc500,
                        ),
                      ),
                    ],
                    const SizedBox(height: 14),
                    QuantityChanger(
                      quantity: item.quantity.toInt(),
                      onChanged: onQuantity,
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 16),
              Column(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [total, const SizedBox(height: 6), remove],
              ),
            ],
          ),
        );
      },
    );
  }
}

class _CartSummary extends StatelessWidget {
  const _CartSummary({required this.cart});
  final Cart cart;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppColors.zinc50,
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.zinc200),
      ),
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Cart Summary',
            style: TextStyle(
              fontFamily: EzTheme.serif,
              fontSize: 24,
              color: AppColors.navyBlue,
            ),
          ),
          const SizedBox(height: 20),
          _row('Sub Total', cart.formattedSubTotal),
          if (cart.formattedDiscount.isNotEmpty &&
              cart.formattedDiscount != '0')
            _row(
              'Discount',
              '-${cart.formattedDiscount}',
              color: AppColors.red600,
            ),
          _row('Tax', cart.formattedTaxTotal),
          if (cart.shippingMethod != null)
            _row('Shipping', cart.formattedShipping),
          const Divider(height: 28),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Grand Total',
                style: TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: 17,
                  fontWeight: FontWeight.w600,
                  color: AppColors.gray900,
                ),
              ),
              Text(
                cart.formattedGrandTotal,
                style: const TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: 20,
                  fontWeight: FontWeight.w600,
                  color: AppColors.navyBlue,
                ),
              ),
            ],
          ),
          const SizedBox(height: 24),
          PrimaryButton(
            label: 'Proceed to Checkout',
            expand: true,
            icon: Icons.lock_outline,
            onPressed: () => context.go('/checkout'),
          ),
          const SizedBox(height: 12),
          SecondaryButton(
            label: 'Continue Shopping',
            expand: true,
            onPressed: () => context.go('/'),
          ),
        ],
      ),
    );
  }

  Widget _row(String label, String value, {Color? color}) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            label,
            style: const TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 14,
              color: AppColors.zinc600,
            ),
          ),
          Text(
            value,
            style: TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 14,
              fontWeight: FontWeight.w500,
              color: color ?? AppColors.gray900,
            ),
          ),
        ],
      ),
    );
  }
}

class _EmptyCart extends StatelessWidget {
  const _EmptyCart({required this.onContinue});
  final VoidCallback onContinue;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 110),
      child: Column(
        children: [
          Container(
            width: 88,
            height: 88,
            decoration: const BoxDecoration(
              shape: BoxShape.circle,
              color: AppColors.zinc100,
            ),
            child: const Icon(
              Icons.shopping_bag_outlined,
              size: 36,
              color: AppColors.zinc400,
            ),
          ),
          const SizedBox(height: 20),
          const Text(
            'Your shopping cart is empty.',
            style: TextStyle(
              fontFamily: EzTheme.serif,
              fontSize: 26,
              color: AppColors.navyBlue,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            'Looks like you have not added anything to your cart yet.',
            textAlign: TextAlign.center,
            style: TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 14,
              color: AppColors.zinc500,
            ),
          ),
          const SizedBox(height: 24),
          PrimaryButton(label: 'Continue Shopping', onPressed: onContinue),
        ],
      ),
    );
  }
}
