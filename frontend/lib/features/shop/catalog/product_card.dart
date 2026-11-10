import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Card de producto (modo grid del shop Bagisto): imagen con badge Sale/New,
/// rating pill flotante, nombre, precio y acciones al hover.
class ProductCard extends StatefulWidget {
  const ProductCard({super.key, required this.product});
  final Product product;

  @override
  State<ProductCard> createState() => _ProductCardState();
}

class _ProductCardState extends State<ProductCard> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final product = widget.product;
    final image = apiUrlRelative(product.imageUrl);
    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: () => context.go('/product/${product.id}'),
        child: Container(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
            boxShadow: _hovering
                ? const [BoxShadow(color: Color(0x1A000000), blurRadius: 10, offset: Offset(0, 5))]
                : null,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Stack(
                children: [
                  AspectRatio(
                    aspectRatio: 0.97,
                    child: Container(
                      decoration: BoxDecoration(
                        color: AppColors.zinc100,
                        borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                      ),
                      child: ClipRRect(
                        borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                        child: image != null
                            ? CachedNetworkImage(
                                imageUrl: image,
                                fit: BoxFit.contain,
                                errorWidget: (_, __, ___) => const Icon(Icons.image_outlined,
                                    size: 48, color: AppColors.zinc300),
                              )
                            : const Icon(Icons.image_outlined, size: 48, color: AppColors.zinc300),
                      ),
                    ),
                  ),
                  // Badge Sale / New
                  Positioned(
                    top: 8,
                    left: 0,
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                      decoration: BoxDecoration(
                        color: product.onSale ? AppColors.red600 : AppColors.navyBlue,
                        borderRadius: const BorderRadius.horizontal(right: Radius.circular(20)),
                      ),
                      child: Text(
                        product.onSale ? 'Sale' : 'New',
                        style: const TextStyle(
                            color: Colors.white, fontSize: 12, fontWeight: FontWeight.w500),
                      ),
                    ),
                  ),
                  // Rating pill
                  if (product.ratingsTotal > 0)
                    Positioned(
                      left: 8,
                      bottom: 8,
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                        decoration: BoxDecoration(
                          color: Colors.white.withOpacity(0.85),
                          borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Text(product.ratingAverage.toStringAsFixed(1),
                                style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600)),
                            const SizedBox(width: 4),
                            const Icon(Icons.star, size: 14, color: AppColors.amber500),
                          ],
                        ),
                      ),
                    ),
                ],
              ),
              const SizedBox(height: 10),
              Text(
                product.name,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w500),
              ),
              const SizedBox(height: 4),
              Row(
                children: [
                  Text(
                    product.formattedPrice,
                    style: const TextStyle(
                        fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w600),
                  ),
                  if (product.onSale) ...[
                    const SizedBox(width: 8),
                    Text(
                      product.formattedRegularPrice,
                      style: const TextStyle(
                          fontFamily: 'Poppins',
                          fontSize: 13,
                          color: AppColors.zinc500,
                          decoration: TextDecoration.lineThrough),
                    ),
                  ],
                ],
              ),
            ],
          ),
        ),
      ),
    );
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
    );
  }
}
