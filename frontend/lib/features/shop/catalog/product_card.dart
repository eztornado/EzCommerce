import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Card de producto (modo grid del shop Bagisto): imagen con badges Sale/New,
/// rating flotante, nombre, precio y acciones que aparecen al hover.
class ProductCard extends StatefulWidget {
  const ProductCard({
    super.key,
    required this.product,
    this.width = 291,
    this.showActions = true,
  });

  final Product product;
  final double width;
  final bool showActions;

  @override
  State<ProductCard> createState() => _ProductCardState();
}

class _ProductCardState extends State<ProductCard> {
  bool _hovering = false;
  bool _adding = false;

  Product get product => widget.product;

  Future<void> _addToCart() async {
    if (product.type == 'configurable') {
      context.go('/product/${product.id}');
      return;
    }
    setState(() => _adding = true);
    final error = await context.read<CartProvider>().addProduct(product.id ?? 0);
    if (!mounted) return;
    setState(() => _adding = false);
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(
      content: Text(error ?? 'The item has been added to your cart successfully.'),
      backgroundColor: error != null ? AppColors.red600 : AppColors.navyBlue,
    ));
  }

  Future<void> _wishlist() async {
    try {
      await ApiClient.instance.post('customer/wishlist', body: {'product_id': product.id});
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
        content: Text('The product has been added to your wishlist.'),
      ));
    } catch (_) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
        content: Text('Please sign in to save products to your wishlist.'),
      ));
      context.push('/login');
    }
  }

  Future<void> _compare() async {
    try {
      await ApiClient.instance.post('customer/compare-items', body: {'product_id': product.id});
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
        content: Text('The product has been added to the compare list.'),
      ));
    } catch (_) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
        content: Text('Please sign in to compare products.'),
      ));
      context.push('/login');
    }
  }

  @override
  Widget build(BuildContext context) {
    final image = apiUrlRelative(product.imageUrl);
    final isNarrow = MediaQuery.of(context).size.width < 768;
    final showActions = widget.showActions && !isNarrow;
    final active = _hovering && showActions;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: () => context.go('/product/${product.id}'),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 220),
          width: widget.width,
          decoration: BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
            boxShadow: active
                ? const [
                    BoxShadow(
                      color: Color(0x1A000000),
                      blurRadius: 14,
                      offset: Offset(0, 6),
                    ),
                  ]
                : null,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Imagen
              ClipRRect(
                borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                child: Stack(
                  children: [
                    AspectRatio(
                      aspectRatio: 291 / 300,
                      child: Container(
                        color: AppColors.zinc100,
                        child: AnimatedScale(
                          duration: const Duration(milliseconds: 300),
                          scale: active ? 1.045 : 1,
                          child: image != null
                              ? Image.network(
                                  image,
                                  fit: BoxFit.cover,
                                  filterQuality: FilterQuality.medium,
                                  errorBuilder: (_, _, _) => const _ImageFallback(),
                                  loadingBuilder: (context, child, progress) =>
                                      progress == null ? child : const _ImageSkeleton(),
                                )
                              : const _ImageFallback(),
                        ),
                      ),
                    ),
                    // Badge Sale / New
                    if (product.onSale || product.isNew)
                      Positioned(
                        top: 10,
                        left: 10,
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                          decoration: BoxDecoration(
                            color: product.onSale ? AppColors.red600 : AppColors.navyBlue,
                            borderRadius: BorderRadius.circular(999),
                          ),
                          child: Text(
                            product.onSale ? 'Sale' : 'New',
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 11,
                              fontWeight: FontWeight.w500,
                              letterSpacing: .2,
                            ),
                          ),
                        ),
                      ),
                    // Rating
                    if (product.ratingsTotal > 0)
                      Positioned(
                        left: 10,
                        bottom: 10,
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          decoration: BoxDecoration(
                            color: Colors.white.withValues(alpha: .9),
                            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                          ),
                          child: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              Text(
                                product.ratingAverage.toStringAsFixed(1),
                                style: const TextStyle(
                                    fontSize: 11, fontWeight: FontWeight.w600),
                              ),
                              const SizedBox(width: 3),
                              const Icon(Icons.star, size: 12, color: AppColors.amber500),
                            ],
                          ),
                        ),
                      ),
                    // Acciones rápidas
                    if (showActions)
                      Positioned(
                        top: 10,
                        right: 10,
                        child: AnimatedOpacity(
                          opacity: active ? 1 : 0,
                          duration: const Duration(milliseconds: 180),
                          child: Column(
                            children: [
                              _QuickAction(
                                icon: Icons.favorite_border,
                                tooltip: 'Add to wishlist',
                                onTap: _wishlist,
                              ),
                              const SizedBox(height: 8),
                              _QuickAction(
                                icon: Icons.compare_arrows,
                                tooltip: 'Add to compare',
                                onTap: _compare,
                              ),
                            ],
                          ),
                        ),
                      ),
                  ],
                ),
              ),
              // Información
              Padding(
                padding: const EdgeInsets.fromLTRB(4, 12, 4, 12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      product.name,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: 15,
                        height: 1.35,
                        fontWeight: FontWeight.w500,
                        color: AppColors.gray900,
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
                    if (showActions) ...[
                      const SizedBox(height: 10),
                      AnimatedOpacity(
                        opacity: active ? 1 : 0,
                        duration: const Duration(milliseconds: 180),
                        child: SizedBox(
                          height: 38,
                          width: double.infinity,
                          child: TextButton(
                            onPressed: _adding ? null : _addToCart,
                            style: TextButton.styleFrom(
                              backgroundColor: Colors.white,
                              foregroundColor: AppColors.navyBlue,
                              side: const BorderSide(color: AppColors.navyBlue),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                              ),
                            ),
                            child: _adding
                                ? const SizedBox(
                                    width: 14,
                                    height: 14,
                                    child: CircularProgressIndicator(
                                        strokeWidth: 2, color: AppColors.navyBlue),
                                  )
                                : Text(
                                    product.type == 'configurable'
                                        ? 'Select Options'
                                        : 'Add To Cart',
                                    style: const TextStyle(
                                      fontFamily: EzTheme.sans,
                                      fontSize: 12,
                                      fontWeight: FontWeight.w500,
                                    ),
                                  ),
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _QuickAction extends StatelessWidget {
  const _QuickAction({required this.icon, required this.tooltip, required this.onTap});

  final IconData icon;
  final String tooltip;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: tooltip,
      child: Material(
        color: Colors.white,
        shape: const CircleBorder(),
        elevation: 0,
        child: InkWell(
          onTap: onTap,
          customBorder: const CircleBorder(),
          child: Container(
            width: 32,
            height: 32,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              border: Border.all(color: AppColors.zinc200),
            ),
            child: Icon(icon, size: 17, color: AppColors.gray900),
          ),
        ),
      ),
    );
  }
}

class _ImageFallback extends StatelessWidget {
  const _ImageFallback();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Icon(Icons.image_outlined, size: 44, color: AppColors.zinc300),
    );
  }
}

class _ImageSkeleton extends StatelessWidget {
  const _ImageSkeleton();

  @override
  Widget build(BuildContext context) {
    return Container(color: AppColors.zinc100);
  }
}

/// Botón de añadir al carrito reutilizable (con estado de carga).
class AddToCartButton extends StatefulWidget {
  const AddToCartButton({
    super.key,
    required this.productId,
    this.quantity = 1,
    this.options,
    this.label = 'Add To Cart',
    this.expand = true,
  });

  final int productId;
  final int quantity;
  final Map<String, dynamic>? options;
  final String label;
  final bool expand;

  @override
  State<AddToCartButton> createState() => _AddToCartButtonState();
}

class _AddToCartButtonState extends State<AddToCartButton> {
  bool _loading = false;

  Future<void> _add() async {
    setState(() => _loading = true);
    final error = await context.read<CartProvider>().addProduct(
          widget.productId,
          quantity: widget.quantity,
          options: widget.options,
        );
    if (!mounted) return;
    setState(() => _loading = false);
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(
      content: Text(error ?? 'The item has been added to your cart successfully.'),
      backgroundColor: error != null ? AppColors.red600 : AppColors.navyBlue,
    ));
  }

  @override
  Widget build(BuildContext context) {
    return PrimaryButton(
      label: _loading ? 'Adding…' : widget.label,
      expand: widget.expand,
      isLoading: _loading,
      onPressed: _loading ? null : _add,
      icon: Icons.shopping_bag_outlined,
    );
  }
}
