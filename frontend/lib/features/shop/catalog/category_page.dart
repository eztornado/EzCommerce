import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../shell/shop_shell.dart';
import 'catalog_filters.dart';
import 'product_card.dart';

/// Listado de productos con el layout de Bagisto: banner 4:1, sidebar de
/// filtros, toolbar (sort + paginación + modo) y grid de 3 columnas.
class CategoryPage extends StatefulWidget {
  const CategoryPage({super.key, this.categoryId});
  final int? categoryId;

  @override
  State<CategoryPage> createState() => _CategoryPageState();
}

class _CategoryPageState extends State<CategoryPage> {
  static const _sortOptions = <String, String>{
    'newest': 'Newest first',
    'price_asc': 'Price: Low to High',
    'price_desc': 'Price: High to Low',
    'name_asc': 'Name: A-Z',
    'name_desc': 'Name: Z-A',
  };

  String _sort = 'newest';
  int _perPage = 12;
  String _mode = 'grid';
  ShopCategory? _category;
  Map<String, String> _filters = {};

  @override
  void initState() {
    super.initState();
    _loadProducts();
    _loadCategory();
  }

  @override
  void didUpdateWidget(covariant CategoryPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.categoryId != widget.categoryId) {
      _category = null;
      _filters = {};
      _loadProducts();
      _loadCategory();
    }
  }

  Future<void> _loadProducts() async {
    await context.read<CatalogProvider>().loadProducts(
      categoryId: widget.categoryId,
      sort: _sort,
      perPage: _perPage,
      filters: _filters,
    );
  }

  Future<void> _loadCategory() async {
    if (widget.categoryId == null) return;
    final data = await apiGet('categories/${widget.categoryId}');
    if (data is Map && mounted) {
      final json = Map<String, dynamic>.from(data);
      if (json['banner_url'] == null && json['category_banner'] is String) {
        json['banner_url'] = '/storage/${json['category_banner']}';
      }
      setState(() => _category = ShopCategory(json));
    }
  }

  @override
  Widget build(BuildContext context) {
    final catalog = context.watch<CatalogProvider>();
    final width = MediaQuery.of(context).size.width;
    final showSidebar = width >= 1060;
    final banner = apiUrlRelative(_category?.bannerUrl);
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);

    return Scaffold(
      backgroundColor: Colors.white,
      bottomNavigationBar: showSidebar
          ? null
          : MobileFilterBar(
              categoryId: widget.categoryId,
              currentSort: _sort,
              onSort: (value) {
                setState(() => _sort = value);
                _loadProducts();
              },
              onFilters: (filters) {
                setState(() => _filters = filters);
                _loadProducts();
              },
            ),
      body: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 1440),
                child: Padding(
                  padding: EdgeInsets.symmetric(horizontal: horizontal),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(height: 22),
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
                          Text(
                            _category?.name ?? 'Products',
                            style: const TextStyle(
                              fontFamily: EzTheme.sans,
                              fontSize: 13,
                              color: AppColors.zinc500,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 18),
                      if ((_category?.name ?? '').isNotEmpty) ...[
                        Text(
                          _category!.name,
                          style: TextStyle(
                            fontFamily: EzTheme.serif,
                            fontSize: width < 640 ? 28 : 34,
                            height: 1.15,
                            color: AppColors.navyBlue,
                          ),
                        ),
                        const SizedBox(height: 16),
                      ],
                      if (banner != null) ...[
                        const SizedBox(height: 2),
                        ClipRRect(
                          borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                          child: AspectRatio(
                            aspectRatio: 4,
                            child: Image.network(
                              banner,
                              fit: BoxFit.cover,
                              filterQuality: FilterQuality.medium,
                              errorBuilder: (_, _, _) => Container(
                                color: AppColors.zinc100,
                                alignment: Alignment.center,
                                child: const Icon(
                                  Icons.image_outlined,
                                  size: 40,
                                  color: AppColors.zinc300,
                                ),
                              ),
                            ),
                          ),
                        ),
                      ],
                      if ((_category?.description ?? '').isNotEmpty) ...[
                        const SizedBox(height: 28),
                        Text(
                          _category!.description!,
                          style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 14,
                            height: 1.7,
                            color: AppColors.zinc600,
                          ),
                        ),
                      ],
                      SizedBox(height: banner != null ? 40 : 32),
                      if (showSidebar)
                        Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            SizedBox(
                              width: 286,
                              child: CatalogFilters(
                                categoryId: widget.categoryId,
                                onApply: (filters) {
                                  setState(() => _filters = filters);
                                  _loadProducts();
                                },
                              ),
                            ),
                            const SizedBox(width: 40),
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.stretch,
                                children: [
                                  _Toolbar(
                                    sort: _sort,
                                    sortOptions: _sortOptions,
                                    mode: _mode,
                                    perPage: _perPage,
                                    onSort: (value) {
                                      setState(() => _sort = value);
                                      _loadProducts();
                                    },
                                    onMode: (value) =>
                                        setState(() => _mode = value),
                                    onPerPage: (value) {
                                      setState(() => _perPage = value);
                                      _loadProducts();
                                    },
                                  ),
                                  const SizedBox(height: 28),
                                  _ProductCollection(
                                    products: catalog.products,
                                    loading: catalog.loading,
                                    mode: _mode,
                                  ),
                                  const SizedBox(height: 28),
                                  _LoadMore(
                                    catalog: catalog,
                                    onPressed: () => catalog.loadMore(
                                      categoryId: widget.categoryId,
                                      sort: _sort,
                                      perPage: _perPage,
                                      filters: _filters,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                          ],
                        )
                      else ...[
                        _Toolbar(
                          sort: _sort,
                          sortOptions: _sortOptions,
                          mode: _mode,
                          perPage: _perPage,
                          compact: true,
                          onSort: (value) {
                            setState(() => _sort = value);
                            _loadProducts();
                          },
                          onMode: (value) => setState(() => _mode = value),
                          onPerPage: (value) {
                            setState(() => _perPage = value);
                            _loadProducts();
                          },
                        ),
                        const SizedBox(height: 20),
                        _ProductCollection(
                          products: catalog.products,
                          loading: catalog.loading,
                          mode: _mode,
                        ),
                        const SizedBox(height: 24),
                        _LoadMore(
                          catalog: catalog,
                          onPressed: () => catalog.loadMore(
                            categoryId: widget.categoryId,
                            sort: _sort,
                            perPage: _perPage,
                            filters: _filters,
                          ),
                        ),
                      ],
                      const SizedBox(height: 56),
                    ],
                  ),
                ),
              ),
            ),
            const ShopFooterSections(),
          ],
        ),
      ),
    );
  }
}

/// Resultados de búsqueda (misma grilla, título con la query).
class SearchPage extends StatelessWidget {
  const SearchPage({super.key, required this.query});
  final String query;

  @override
  Widget build(BuildContext context) {
    return SearchResultsView(query: query);
  }
}

class SearchResultsView extends StatefulWidget {
  const SearchResultsView({super.key, required this.query});
  final String query;

  @override
  State<SearchResultsView> createState() => _SearchResultsViewState();
}

class _SearchResultsViewState extends State<SearchResultsView> {
  String _sort = 'newest';
  String _mode = 'grid';

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<CatalogProvider>().loadProducts(
        query: widget.query,
        sort: _sort,
      );
    });
  }

  @override
  void didUpdateWidget(covariant SearchResultsView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.query != widget.query) {
      context.read<CatalogProvider>().loadProducts(
        query: widget.query,
        sort: _sort,
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final catalog = context.watch<CatalogProvider>();
    final width = MediaQuery.of(context).size.width;
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);

    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1440),
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: horizontal),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const SizedBox(height: 24),
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
                          'Search',
                          style: TextStyle(
                            fontSize: 13,
                            color: AppColors.zinc500,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 14),
                    Text(
                      widget.query.trim().isEmpty
                          ? 'All Products'
                          : "Search results for: '${widget.query}'",
                      style: TextStyle(
                        fontFamily: EzTheme.serif,
                        fontSize: width < 640 ? 24 : 30,
                        height: 1.2,
                        color: AppColors.navyBlue,
                      ),
                    ),
                    const SizedBox(height: 24),
                    _Toolbar(
                      sort: _sort,
                      sortOptions: _CategoryPageState._sortOptions,
                      mode: _mode,
                      perPage: 12,
                      compact: width < 720,
                      onSort: (value) {
                        setState(() => _sort = value);
                        context.read<CatalogProvider>().loadProducts(
                          query: widget.query,
                          sort: value,
                        );
                      },
                      onMode: (value) => setState(() => _mode = value),
                      onPerPage: (_) {},
                    ),
                    const SizedBox(height: 24),
                    _ProductCollection(
                      products: catalog.products,
                      loading: catalog.loading,
                      mode: _mode,
                    ),
                    const SizedBox(height: 56),
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

// ---------------------------------------------------------------------------
// Piezas compartidas del listado
// ---------------------------------------------------------------------------

class _Toolbar extends StatelessWidget {
  const _Toolbar({
    required this.sort,
    required this.sortOptions,
    required this.mode,
    required this.perPage,
    required this.onSort,
    required this.onMode,
    required this.onPerPage,
    this.compact = false,
  });

  final String sort;
  final Map<String, String> sortOptions;
  final String mode;
  final int perPage;
  final ValueChanged<String> onSort;
  final ValueChanged<String> onMode;
  final ValueChanged<int> onPerPage;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    final sortButton = _ToolbarDropdown<String>(
      width: compact ? 170 : 210,
      value: sort,
      items: sortOptions.entries
          .map(
            (entry) =>
                DropdownMenuItem(value: entry.key, child: Text(entry.value)),
          )
          .toList(),
      onChanged: onSort,
    );

    return Row(
      children: [
        sortButton,
        const Spacer(),
        if (!compact) ...[
          _ToolbarDropdown<int>(
            width: 120,
            value: perPage,
            prefix: 'Show ',
            items: const [12, 24, 36]
                .map(
                  (value) =>
                      DropdownMenuItem(value: value, child: Text('$value')),
                )
                .toList(),
            onChanged: onPerPage,
          ),
          const SizedBox(width: 24),
        ],
        _ModeButton(
          icon: Icons.view_list_outlined,
          active: mode == 'list',
          tooltip: 'List view',
          onTap: () => onMode('list'),
        ),
        const SizedBox(width: 10),
        _ModeButton(
          icon: Icons.grid_view_rounded,
          active: mode == 'grid',
          tooltip: 'Grid view',
          onTap: () => onMode('grid'),
        ),
      ],
    );
  }
}

class _ToolbarDropdown<T> extends StatelessWidget {
  const _ToolbarDropdown({
    required this.value,
    required this.items,
    required this.onChanged,
    required this.width,
    this.prefix = '',
  });

  final T value;
  final List<DropdownMenuItem<T>> items;
  final ValueChanged<T> onChanged;
  final double width;
  final String prefix;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: 52,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(EzTheme.radiusLg),
        border: Border.all(color: AppColors.zinc200),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<T>(
          value: value,
          isExpanded: true,
          icon: const Icon(
            Icons.keyboard_arrow_down,
            size: 20,
            color: AppColors.zinc500,
          ),
          borderRadius: BorderRadius.circular(EzTheme.radiusLg),
          style: const TextStyle(
            fontFamily: EzTheme.sans,
            fontSize: 14,
            color: AppColors.gray900,
          ),
          selectedItemBuilder: (context) => items
              .map(
                (item) => Align(
                  alignment: Alignment.centerLeft,
                  child: Text(
                    prefix.isEmpty ? _label(item) : '$prefix${_label(item)}',
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontSize: 14,
                      color: AppColors.gray900,
                    ),
                  ),
                ),
              )
              .toList(),
          items: items,
          onChanged: (value) {
            if (value != null) onChanged(value);
          },
        ),
      ),
    );
  }

  String _label(DropdownMenuItem<T> item) {
    final child = item.child;
    return child is Text ? child.data ?? '${item.value}' : '${item.value}';
  }
}

class _ModeButton extends StatelessWidget {
  const _ModeButton({
    required this.icon,
    required this.active,
    required this.tooltip,
    required this.onTap,
  });

  final IconData icon;
  final bool active;
  final String tooltip;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(EzTheme.radiusLg),
        child: Container(
          width: 42,
          height: 42,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
            border: Border.all(
              color: active ? AppColors.navyBlue : Colors.transparent,
            ),
          ),
          child: Icon(
            icon,
            size: 22,
            color: active ? AppColors.navyBlue : AppColors.zinc400,
          ),
        ),
      ),
    );
  }
}

class _LoadMore extends StatelessWidget {
  const _LoadMore({required this.catalog, required this.onPressed});
  final CatalogProvider catalog;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    if (!catalog.hasMore) return const SizedBox.shrink();
    return Center(
      child: catalog.loading
          ? const Padding(
              padding: EdgeInsets.all(18),
              child: CircularProgressIndicator(color: AppColors.navyBlue),
            )
          : SecondaryButton(
              label: 'Load More',
              height: 52,
              onPressed: onPressed,
            ),
    );
  }
}

/// Grid (3/2/1 columnas) o lista, con skeletons y estado vacío.
class _ProductCollection extends StatelessWidget {
  const _ProductCollection({
    required this.products,
    required this.loading,
    required this.mode,
  });

  final List<Product> products;
  final bool loading;
  final String mode;

  @override
  Widget build(BuildContext context) {
    if (loading && products.isEmpty) {
      return LayoutBuilder(
        builder: (context, constraints) {
          final columns = _columns(constraints.maxWidth);
          final gap = _gap(constraints.maxWidth);
          final width = (constraints.maxWidth - gap * (columns - 1)) / columns;
          return Wrap(
            spacing: gap,
            runSpacing: gap,
            children: List.generate(
              columns * 2,
              (_) => SizedBox(
                width: width,
                height: _height(width, constraints.maxWidth),
                child: const _CardSkeleton(),
              ),
            ),
          );
        },
      );
    }

    if (products.isEmpty) {
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 96),
        child: Column(
          children: [
            Icon(
              Icons.inventory_2_outlined,
              size: 64,
              color: AppColors.zinc300,
            ),
            SizedBox(height: 16),
            Text(
              'No products were found.',
              style: TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 18,
                color: AppColors.zinc500,
              ),
            ),
            SizedBox(height: 8),
            Text(
              'Try a different keyword or browse the categories in the menu.',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontFamily: EzTheme.sans,
                fontSize: 14,
                color: AppColors.zinc400,
              ),
            ),
          ],
        ),
      );
    }

    if (mode == 'list') {
      return Column(
        children: products
            .map(
              (product) => Padding(
                padding: const EdgeInsets.only(bottom: 20),
                child: _ProductListTile(product: product),
              ),
            )
            .toList(),
      );
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        final columns = _columns(constraints.maxWidth);
        final gap = _gap(constraints.maxWidth);
        final width = (constraints.maxWidth - gap * (columns - 1)) / columns;
        return GridView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          itemCount: products.length,
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: columns,
            crossAxisSpacing: gap,
            mainAxisSpacing: constraints.maxWidth < 600 ? 24 : 32,
            mainAxisExtent: _height(width, constraints.maxWidth),
          ),
          itemBuilder: (context, index) =>
              ProductCard(product: products[index]),
        );
      },
    );
  }

  double _gap(double width) {
    if (width >= 600) return 32;
    return 16;
  }

  double _height(double cardWidth, double containerWidth) {
    final imageHeight = cardWidth * 300 / 291;
    return imageHeight + (containerWidth < 600 ? 104 : 154);
  }

  int _columns(double width) => width >= 900 ? 3 : 2;
}

class _ProductListTile extends StatefulWidget {
  const _ProductListTile({required this.product});
  final Product product;

  @override
  State<_ProductListTile> createState() => _ProductListTileState();
}

class _ProductListTileState extends State<_ProductListTile> {
  @override
  Widget build(BuildContext context) {
    final product = widget.product;
    final image = apiUrlRelative(product.imageUrl);
    final phone = MediaQuery.of(context).size.width < 640;
    return InkWell(
      onTap: () => context.go('/product/${product.id}'),
      borderRadius: BorderRadius.circular(EzTheme.radiusXl),
      child: Container(
        padding: EdgeInsets.all(phone ? 12 : 16),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(EzTheme.radiusXl),
          border: Border.all(color: AppColors.zinc200),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            ClipRRect(
              borderRadius: BorderRadius.circular(EzTheme.radiusLg),
              child: SizedBox(
                width: phone ? 108 : 210,
                height: phone ? 120 : 220,
                child: Container(
                  color: AppColors.zinc100,
                  child: image != null
                      ? Image.network(
                          image,
                          fit: BoxFit.cover,
                          errorBuilder: (_, _, _) => const Icon(
                            Icons.image_outlined,
                            color: AppColors.zinc300,
                            size: 40,
                          ),
                        )
                      : const Icon(
                          Icons.image_outlined,
                          color: AppColors.zinc300,
                          size: 40,
                        ),
                ),
              ),
            ),
            SizedBox(width: phone ? 14 : 28),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 4),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      product.name,
                      maxLines: phone ? 2 : null,
                      overflow: phone ? TextOverflow.ellipsis : null,
                      style: TextStyle(
                        fontFamily: EzTheme.sans,
                        fontSize: phone ? 15 : 20,
                        fontWeight: FontWeight.w500,
                        color: AppColors.gray900,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Row(
                      children: [
                        Text(
                          product.formattedPrice,
                          style: TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: phone ? 15 : 18,
                            fontWeight: FontWeight.w600,
                            color: AppColors.gray900,
                          ),
                        ),
                        if (product.onSale) ...[
                          const SizedBox(width: 10),
                          Text(
                            product.formattedRegularPrice,
                            style: const TextStyle(
                              fontSize: 13,
                              color: AppColors.zinc400,
                              decoration: TextDecoration.lineThrough,
                            ),
                          ),
                        ],
                      ],
                    ),
                    if (!phone &&
                        (product.shortDescription ?? '').isNotEmpty) ...[
                      const SizedBox(height: 12),
                      Text(
                        product.shortDescription!,
                        maxLines: 3,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontSize: 14,
                          height: 1.7,
                          color: AppColors.zinc600,
                        ),
                      ),
                    ],
                    SizedBox(height: phone ? 12 : 18),
                    SizedBox(
                      width: phone ? double.infinity : 200,
                      child: AddToCartButton(
                        productId: product.id ?? 0,
                        label: product.type == 'configurable'
                            ? 'Select Options'
                            : 'Add To Cart',
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _CardSkeleton extends StatelessWidget {
  const _CardSkeleton();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        AspectRatio(
          aspectRatio: 291 / 300,
          child: Container(
            decoration: BoxDecoration(
              color: AppColors.shimmerBase,
              borderRadius: BorderRadius.circular(EzTheme.radiusLg),
            ),
          ),
        ),
        const SizedBox(height: 14),
        Container(
          height: 14,
          width: double.infinity,
          color: AppColors.shimmerBase,
        ),
        const SizedBox(height: 8),
        Container(height: 14, width: 120, color: AppColors.shimmerBase),
      ],
    );
  }
}
