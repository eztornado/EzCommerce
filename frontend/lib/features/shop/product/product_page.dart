import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../catalog/product_card.dart';

/// Página de producto: galería (desktop thumbnails / móvil carrusel),
/// precio con tachado, opciones configurables, qty, tabs/accordions,
/// related products.
class ProductPage extends StatefulWidget {
  const ProductPage({super.key, required this.productId});
  final String productId;

  @override
  State<ProductPage> createState() => _ProductPageState();
}

class _ProductPageState extends State<ProductPage> {
  Product? _product;
  bool _loading = true;
  int _quantity = 1;
  bool _inWishlist = false;

  // Selección de super atributos (configurable)
  final Map<String, int> _selectedOptions = {};
  Variant? _matchedVariant;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final data = await apiGet('products/${widget.productId}');
      if (data is Map && mounted) {
        setState(() {
          _product = Product(Map<String, dynamic>.from(data));
          _loading = false;
        });
      }
    } catch (_) {
      if (mounted) setState(() => _loading = false);
    }
  }

  void _onOptionChanged(String code, int value) {
    setState(() {
      _selectedOptions[code] = value;
      _matchVariant();
    });
  }

  void _matchVariant() {
    if (_product == null) return;
    for (final variant in _product!.variants) {
      final matches = _selectedOptions.entries
          .every((entry) => variant.attributes[entry.key] == entry.value);
      if (matches && variant.attributes.length == _selectedOptions.length) {
        _matchedVariant = variant;
        return;
      }
    }
    _matchedVariant = null;
  }

  Future<void> _addToCart() async {
    final product = _product!;
    final options = <String, dynamic>{};
    _selectedOptions.forEach((code, value) => options[code] = value);
    final error = await context.read<CartProvider>().addProduct(
          _matchedVariant?.id ?? product.id ?? 0,
          quantity: _quantity,
          options: options.isNotEmpty ? options : null,
        );
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(
      content: Text(error ?? 'The item has been added to your cart successfully.'),
      backgroundColor: error != null ? AppColors.red600 : AppColors.navyBlue,
    ));
  }

  Future<void> _buyNow() async {
    await _addToCart();
    if (mounted && context.read<CartProvider>().cart != null) {
      context.go('/checkout');
    }
  }

  Future<void> _toggleWishlist() async {
    try {
      await apiPost('customer/wishlist', body: {'product_id': _product!.id});
      if (mounted) setState(() => _inWishlist = !_inWishlist);
    } catch (_) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
            content: Text('Please sign in to add products to your wishlist.')));
        context.push('/login');
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    final product = _product;
    if (product == null) {
      return const Center(child: Text('Product not found.'));
    }
    final isNarrow = MediaQuery.of(context).size.width <= 1180;
    return SingleChildScrollView(
      padding: EdgeInsets.symmetric(horizontal: isNarrow ? 16 : 60, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              GestureDetector(
                onTap: () => context.go('/'),
                child: const Text('Home'),
              ),
              const Icon(Icons.chevron_right, size: 16, color: AppColors.zinc400),
              Text(product.name, style: const TextStyle(color: AppColors.zinc500)),
            ],
          ),
          const SizedBox(height: 24),
          Flex(
            direction: isNarrow ? Axis.vertical : Axis.horizontal,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Galería
              Container(
                constraints: isNarrow
                    ? const BoxConstraints()
                    : const BoxConstraints(maxWidth: 520),
                child: ProductGallery(imageUrls: product.imageUrls, isNarrow: isNarrow),
              ),
              const SizedBox(width: 36, height: 24),
              // Detalles
              Expanded(
                flex: isNarrow ? 0 : 1,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Expanded(
                          child: Text(
                            product.name,
                            style: const TextStyle(
                                fontFamily: 'Poppins', fontSize: 28, fontWeight: FontWeight.w500),
                          ),
                        ),
                        IconButton(
                          icon: Icon(
                            _inWishlist ? Icons.favorite : Icons.favorite_border,
                            color: _inWishlist ? AppColors.red600 : AppColors.gray900,
                          ),
                          onPressed: _toggleWishlist,
                        ),
                      ],
                    ),
                    if (product.ratingsTotal > 0)
                      Row(
                        children: [
                          Text(product.ratingAverage.toStringAsFixed(1),
                              style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600)),
                          const SizedBox(width: 4),
                          const Icon(Icons.star, size: 16, color: AppColors.amber500),
                          const SizedBox(width: 8),
                          Text('(${product.ratingsTotal} reviews)',
                              style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                        ],
                      ),
                    const SizedBox(height: 12),
                    Row(
                      children: [
                        Text(
                          _matchedVariant != null
                              ? (_matchedVariant!.price ?? product.price).toStringAsFixed(2)
                              : product.formattedPrice,
                          style: const TextStyle(
                              fontFamily: 'Poppins', fontSize: 24, fontWeight: FontWeight.w600),
                        ),
                        if (product.onSale) ...[
                          const SizedBox(width: 10),
                          Text(
                            product.formattedRegularPrice,
                            style: const TextStyle(
                                fontFamily: 'Poppins',
                                fontSize: 16,
                                color: AppColors.zinc500,
                                decoration: TextDecoration.lineThrough),
                          ),
                        ],
                      ],
                    ),
                    const SizedBox(height: 16),
                    if (product.shortDescription != null)
                      Text(
                        product.shortDescription!,
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 16, color: AppColors.zinc500),
                      ),
                    const SizedBox(height: 24),
                    // Opciones configurables
                    ...product.superAttributes.map(
                      (attribute) => SuperAttributeSelector(
                        attribute: attribute,
                        selected: _selectedOptions[attribute.code],
                        swatchType: attribute.swatchType,
                        onChanged: (value) => _onOptionChanged(attribute.code, value),
                      ),
                    ),
                    const SizedBox(height: 24),
                    // Qty + Add To Cart
                    Wrap(
                      spacing: 16,
                      runSpacing: 12,
                      children: [
                        QuantityChanger(
                          quantity: _quantity,
                          onChanged: (value) => setState(() => _quantity = value),
                        ),
                        SizedBox(
                          width: isNarrow ? double.infinity : 260,
                          child: AddToCartButton(
                            productId: _matchedVariant?.id ?? product.id ?? 0,
                            quantity: _quantity,
                            options: _selectedOptions.map((k, v) => MapEntry(k, v)),
                            label: 'Add To Cart',
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    SizedBox(
                      width: isNarrow ? double.infinity : 300,
                      child: PrimaryButton(label: 'Buy Now', onPressed: _buyNow),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 48),
          // Tabs / accordions
          ProductInfoTabs(product: product),
          const SizedBox(height: 48),
          // Related
          ProductAssociations(productId: product.id, kind: 'related', title: 'Related Products'),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Galería
// ---------------------------------------------------------------------------

class ProductGallery extends StatefulWidget {
  const ProductGallery({super.key, required this.imageUrls, required this.isNarrow});
  final List<String> imageUrls;
  final bool isNarrow;

  @override
  State<ProductGallery> createState() => _ProductGalleryState();
}

class _ProductGalleryState extends State<ProductGallery> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final urls = widget.imageUrls;
    if (urls.isEmpty) {
      return AspectRatio(
        aspectRatio: 1,
        child: Container(
          decoration: BoxDecoration(
            color: AppColors.zinc100,
            borderRadius: BorderRadius.circular(EzTheme.radiusXl),
          ),
          child: const Icon(Icons.image_outlined, size: 64, color: AppColors.zinc300),
        ),
      );
    }
    final main = apiUrlRelative(urls[_index]);
    if (widget.isNarrow) {
      return AspectRatio(
        aspectRatio: 1,
        child: PageView.builder(
          itemCount: urls.length,
          onPageChanged: (index) => setState(() => _index = index),
          itemBuilder: (context, index) {
            final url = apiUrlRelative(urls[index]);
            return Padding(
              padding: const EdgeInsets.symmetric(horizontal: 2),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                child: CachedNetworkImage(imageUrl: url ?? '', fit: BoxFit.cover,
                    errorWidget: (_, __, ___) => Container(color: AppColors.zinc100)),
              ),
            );
          },
        ),
      );
    }
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Thumbnails
        SizedBox(
          width: 76,
          child: Column(
            children: List.generate(urls.length, (index) {
              final url = apiUrlRelative(urls[index]);
              return GestureDetector(
                onTap: () => setState(() => _index = index),
                child: Container(
                  width: 76,
                  height: 76,
                  margin: const EdgeInsets.only(bottom: 12),
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                    border: Border.all(
                      color: index == _index ? AppColors.navyBlue : Colors.white,
                      width: 2,
                    ),
                  ),
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(10),
                    child: CachedNetworkImage(imageUrl: url ?? '', fit: BoxFit.cover,
                        errorWidget: (_, __, ___) => Container(color: AppColors.zinc100)),
                  ),
                ),
              );
            }),
          ),
        ),
        const SizedBox(width: 20),
        Expanded(
          child: AspectRatio(
            aspectRatio: 0.95,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              child: CachedNetworkImage(imageUrl: main ?? '', fit: BoxFit.cover,
                  errorWidget: (_, __, ___) => Container(color: AppColors.zinc100)),
            ),
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Selector de super atributo (swatches o chips)
// ---------------------------------------------------------------------------

class SuperAttributeSelector extends StatelessWidget {
  const SuperAttributeSelector({
    super.key,
    required this.attribute,
    required this.selected,
    required this.swatchType,
    required this.onChanged,
  });

  final SuperAttribute attribute;
  final int? selected;
  final String swatchType;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(attribute.label,
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 18, fontWeight: FontWeight.w500)),
          const SizedBox(height: 8),
          Wrap(
            spacing: 10,
            runSpacing: 10,
            children: attribute.options.map((option) {
              final isSelected = selected == option.id;
              return GestureDetector(
                onTap: () => onChanged(option.id ?? 0),
                child: swatchType == 'color' && (option.swatchValue ?? '').isNotEmpty
                    ? Container(
                        width: 34,
                        height: 34,
                        decoration: BoxDecoration(
                          color: Color(
                              int.parse((option.swatchValue ?? '#000000').replaceFirst('#', '0xFF'))),
                          shape: BoxShape.circle,
                          border: Border.all(
                            color: isSelected ? AppColors.gray900 : AppColors.zinc200,
                            width: isSelected ? 2 : 1,
                          ),
                        ),
                      )
                    : Container(
                        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 10),
                        decoration: BoxDecoration(
                          borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                          border: Border.all(
                            color: isSelected ? AppColors.navyBlue : AppColors.zinc200,
                            width: isSelected ? 2 : 1,
                          ),
                        ),
                        child: Text(
                          option.name,
                          style: TextStyle(
                            fontFamily: 'Poppins',
                            fontSize: 14,
                            color: isSelected ? AppColors.navyBlue : AppColors.gray900,
                          ),
                        ),
                      ),
              );
            }).toList(),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Quantity changer
// ---------------------------------------------------------------------------

class QuantityChanger extends StatelessWidget {
  const QuantityChanger({super.key, required this.quantity, required this.onChanged});
  final int quantity;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(EzTheme.radiusXl),
        border: Border.all(color: AppColors.navyBlue),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          IconButton(
            icon: const Icon(Icons.remove, size: 20),
            onPressed: quantity > 1 ? () => onChanged(quantity - 1) : null,
          ),
          SizedBox(
            width: 40,
            child: Text(
              '$quantity',
              textAlign: TextAlign.center,
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.add, size: 20),
            onPressed: () => onChanged(quantity + 1),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Tabs (desktop) / accordions (móvil): Description | Additional | Reviews
// ---------------------------------------------------------------------------

class ProductInfoTabs extends StatelessWidget {
  const ProductInfoTabs({super.key, required this.product});
  final Product product;

  @override
  Widget build(BuildContext context) {
    final isNarrow = MediaQuery.of(context).size.width <= 1180;
    final description = _stripHtml(product.description ?? '');
    final additional = product.additionalInformation;

    if (isNarrow) {
      return Column(
        children: [
          _Accordion(title: 'Description', child: _DescriptionText(description)),
          _Accordion(title: 'Additional Information', child: _AdditionalTable(additional)),
          ReviewsSection(productId: product.id),
        ],
      );
    }
    return DefaultTabController(
      length: 3,
      child: Column(
        children: [
          Container(
            color: AppColors.zinc100,
            child: const TabBar(
              indicatorColor: AppColors.navyBlue,
              indicatorSize: TabBarIndicatorSize.tab,
              labelColor: Colors.black,
              unselectedLabelColor: AppColors.zinc600,
              labelStyle: TextStyle(fontFamily: 'Poppins', fontSize: 17, fontWeight: FontWeight.w500),
              tabs: [
                Tab(text: 'Description'),
                Tab(text: 'Additional Information'),
                Tab(text: 'Reviews'),
              ],
            ),
          ),
          SizedBox(
            height: 260,
            child: TabBarView(
              children: [
                SingleChildScrollView(child: _DescriptionText(description)),
                SingleChildScrollView(child: _AdditionalTable(additional)),
                const ReviewsInline(),
              ],
            ),
          ),
        ],
      ),
    );
  }

  String _stripHtml(String html) {
    return html
        .replaceAll(RegExp(r'<[^>]*>'), ' ')
        .replaceAll(RegExp(r'\s+'), ' ')
        .trim();
  }
}

class _DescriptionText extends StatelessWidget {
  const _DescriptionText(this.text);
  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Text(
        text.isEmpty ? 'No description available.' : text,
        style: TextStyle(fontFamily: 'Poppins', fontSize: 15, height: 1.6, color: AppColors.zinc600),
      ),
    );
  }
}

class _AdditionalTable extends StatelessWidget {
  const _AdditionalTable(this.rows);
  final List<Map<String, dynamic>> rows;

  @override
  Widget build(BuildContext context) {
    if (rows.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(16),
        child: Text('No additional information.'),
      );
    }
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Table(
        columnWidths: const {0: FlexColumnWidth(1), 1: FlexColumnWidth(3)},
        children: rows.map((row) {
          return TableRow(
            decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: AppColors.zinc200))),
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 10),
                child: Text('${row['label'] ?? row['code']}',
                    style: const TextStyle(fontFamily: 'Poppins', fontWeight: FontWeight.w500)),
              ),
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 10),
                child: Text('${_value(row['value'])}',
                    style: TextStyle(fontFamily: 'Poppins', color: AppColors.zinc500)),
              ),
            ],
          );
        }).toList(),
      ),
    );
  }

  String _value(dynamic value) {
    if (value is List) return value.join(', ');
    return value?.toString() ?? '';
  }
}

class _Accordion extends StatelessWidget {
  const _Accordion({required this.title, required this.child});
  final String title;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: AppColors.zinc200))),
      child: ExpansionTile(
        tilePadding: const EdgeInsets.symmetric(horizontal: 8),
        title: Text(title,
            style: const TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
        children: [child],
      ),
    );
  }
}

class ReviewsInline extends StatelessWidget {
  const ReviewsInline({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(child: Text('See reviews below.'));
  }
}

class ReviewsSection extends StatefulWidget {
  const ReviewsSection({super.key, required this.productId});
  final int? productId;

  @override
  State<ReviewsSection> createState() => _ReviewsSectionState();
}

class _ReviewsSectionState extends State<ReviewsSection> {
  List<Map<String, dynamic>> _reviews = [];
  bool _loading = true;
  final _formKey = GlobalKey<FormState>();
  final _titleController = TextEditingController();
  final _commentController = TextEditingController();
  int _rating = 5;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final data = await apiGet('product/${widget.productId ?? 0}/reviews');
      if (data is Map) {
        setState(() {
          _reviews = decodeList(data['data']);
          _loading = false;
        });
      }
    } catch (_) {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;
    try {
      await apiPost('product/${widget.productId ?? 0}/review', body: {
        'title': _titleController.text,
        'comment': _commentController.text,
        'rating': _rating,
      });
      _titleController.clear();
      _commentController.clear();
      await _load();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Your review has been submitted successfully.')));
      }
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Reviews',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 20, fontWeight: FontWeight.w500)),
        const SizedBox(height: 16),
        if (_loading)
          const Center(child: CircularProgressIndicator(color: AppColors.navyBlue))
        else if (_reviews.isEmpty)
          Text('No reviews yet. Be the first to review this product.',
              style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.zinc500))
        else
          ..._reviews.map((review) => Container(
                margin: const EdgeInsets.only(bottom: 16),
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                  border: Border.all(color: AppColors.zinc200),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: List.generate(
                        (review['rating'] ?? 0) as int,
                        (_) => const Icon(Icons.star, size: 16, color: AppColors.amber500),
                      ),
                    ),
                    const SizedBox(height: 6),
                    Text('${review['title'] ?? ''}',
                        style: const TextStyle(
                            fontFamily: 'Poppins', fontSize: 15, fontWeight: FontWeight.w600)),
                    Text('${review['comment'] ?? ''}',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc500)),
                    const SizedBox(height: 6),
                    Text('${review['name'] ?? ''}',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc400)),
                  ],
                ),
              )),
        const Divider(height: 32),
        // Formulario
        Form(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Write a review',
                  style: TextStyle(fontFamily: 'Poppins', fontSize: 16, fontWeight: FontWeight.w500)),
              const SizedBox(height: 8),
              Row(
                children: List.generate(5, (index) {
                  return IconButton(
                    icon: Icon(
                      index < _rating ? Icons.star : Icons.star_border,
                      color: AppColors.amber500,
                    ),
                    onPressed: () => setState(() => _rating = index + 1),
                  );
                }),
              ),
              TextFormField(
                controller: _titleController,
                decoration: const InputDecoration(hintText: 'Title'),
                validator: (value) =>
                    (value ?? '').trim().isEmpty ? 'The title field is required.' : null,
              ),
              const SizedBox(height: 12),
              TextFormField(
                controller: _commentController,
                maxLines: 4,
                decoration: const InputDecoration(hintText: 'Your review'),
                validator: (value) =>
                    (value ?? '').trim().isEmpty ? 'The comment field is required.' : null,
              ),
              const SizedBox(height: 12),
              PrimaryButton(label: 'Submit Review', onPressed: _submit),
            ],
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Related products carousel
// ---------------------------------------------------------------------------

class ProductAssociations extends StatefulWidget {
  const ProductAssociations({
    super.key,
    required this.productId,
    required this.kind,
    required this.title,
  });

  final int? productId;
  final String kind;
  final String title;

  @override
  State<ProductAssociations> createState() => _ProductAssociationsState();
}

class _ProductAssociationsState extends State<ProductAssociations> {
  List<Product> _products = [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    if (widget.productId == null) return;
    try {
      final data = await apiGet('products/${widget.productId}/${widget.kind}');
      if (data is List && mounted) {
        setState(() {
          _products = data
              .map((e) => Product(Map<String, dynamic>.from(e as Map)))
              .toList();
        });
      }
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    if (_products.isEmpty) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(widget.title,
            style: const TextStyle(
                fontFamily: 'DM Serif Display', fontSize: 26, color: AppColors.gray900)),
        const SizedBox(height: 20),
        SizedBox(
          height: 360,
          child: ListView.separated(
            scrollDirection: Axis.horizontal,
            itemCount: _products.length,
            separatorBuilder: (_, __) => const SizedBox(width: 24),
            itemBuilder: (context, index) => SizedBox(
              width: 260,
              child: ProductCard(product: _products[index]),
            ),
          ),
        ),
      ],
    );
  }
}
