import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/api/api_client.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import 'product_card.dart';

/// Listado de productos con toolbar (orden), Load More y barra móvil.
/// Los filtros por atributo llegan por query (`/category/:id` + sort/search).
class CategoryPage extends StatefulWidget {
  const CategoryPage({super.key, this.categoryId});
  final int? categoryId;

  @override
  State<CategoryPage> createState() => _CategoryPageState();
}

class _CategoryPageState extends State<CategoryPage> {
  String _sort = 'newest';
  ShopCategory? _category;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void didUpdateWidget(covariant CategoryPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.categoryId != widget.categoryId) _load();
  }

  Future<void> _load() async {
    final catalog = context.read<CatalogProvider>();
    catalog.loadProducts(categoryId: widget.categoryId, sort: _sort);
    if (widget.categoryId != null) {
      final data = await apiGet('categories/${widget.categoryId}');
      if (data is Map && mounted) {
        setState(() => _category = ShopCategory(Map<String, dynamic>.from(data)));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final catalog = context.watch<CatalogProvider>();
    final title = _category?.name ?? 'Products';
    return SingleChildScrollView(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Breadcrumbs
          Row(
            children: [
              GestureDetector(onTap: () => context.go('/'), child: const Text('Home')),
              const Icon(Icons.chevron_right, size: 16, color: AppColors.zinc400),
              Text(title, style: const TextStyle(color: AppColors.zinc500)),
            ],
          ),
          if (_category?.bannerUrl != null) ...[
            const SizedBox(height: 16),
            ClipRRect(
              borderRadius: BorderRadius.circular(EzTheme.radiusXl),
              child: AspectRatio(
                aspectRatio: 4,
                child: Container(color: AppColors.zinc100, alignment: Alignment.center,
                    child: const Icon(Icons.image, size: 32, color: AppColors.zinc300)),
              ),
            ),
          ],
          const SizedBox(height: 16),
          // Toolbar
          Row(
            children: [
              Text(
                catalog.total > 0 ? '$title (${catalog.total} items)' : title,
                style: const TextStyle(fontFamily: 'Poppins', fontSize: 22, fontWeight: FontWeight.w500),
              ),
              const Spacer(),
              DropdownButton<String>(
                value: _sort,
                underline: const SizedBox.shrink(),
                items: const [
                  DropdownMenuItem(value: 'newest', child: Text('Newest first')),
                  DropdownMenuItem(value: 'price_asc', child: Text('Price: Low to High')),
                  DropdownMenuItem(value: 'price_desc', child: Text('Price: High to Low')),
                  DropdownMenuItem(value: 'name_asc', child: Text('Name: A-Z')),
                ],
                onChanged: (value) {
                  if (value == null) return;
                  setState(() => _sort = value);
                  context.read<CatalogProvider>().loadProducts(
                        categoryId: widget.categoryId,
                        sort: value,
                      );
                },
              ),
            ],
          ),
          const Divider(height: 24),
          if (catalog.loading && catalog.products.isEmpty)
            const SizedBox(
              height: 300,
              child: Center(child: CircularProgressIndicator(color: AppColors.navyBlue)),
            )
          else if (catalog.products.isEmpty)
            SizedBox(
              height: 300,
              child: Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(Icons.shopping_bag_outlined, size: 48, color: AppColors.zinc300),
                    const SizedBox(height: 12),
                    Text(
                      'No products were found in this category.',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 16, color: AppColors.zinc500),
                    ),
                  ],
                ),
              ),
            )
          else
            GridView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: catalog.products.length,
              gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                maxCrossAxisExtent: 300,
                mainAxisExtent: 400,
                crossAxisSpacing: 24,
                mainAxisSpacing: 24,
              ),
              itemBuilder: (context, index) =>
                  ProductCard(product: catalog.products[index]),
            ),
          const SizedBox(height: 24),
          // Load More
          if (catalog.hasMore)
            Center(
              child: catalog.loading
                  ? const CircularProgressIndicator(color: AppColors.navyBlue)
                  : SecondaryButton(
                      label: 'Load More',
                      onPressed: () => catalog.loadMore(
                        categoryId: widget.categoryId,
                        sort: _sort,
                      ),
                    ),
            ),
          const SizedBox(height: 32),
        ],
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
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<CatalogProvider>().loadProducts(query: widget.query);
    });
  }

  @override
  Widget build(BuildContext context) {
    final catalog = context.watch<CatalogProvider>();
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            "Search results for: '${widget.query}'",
            style: const TextStyle(fontFamily: 'Poppins', fontSize: 22, fontWeight: FontWeight.w500),
          ),
          const Divider(height: 24),
          if (catalog.products.isEmpty && !catalog.loading)
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 64),
              child: Center(child: Text('No products found.')),
            )
          else
            GridView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: catalog.products.length,
              gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                maxCrossAxisExtent: 300,
                mainAxisExtent: 400,
                crossAxisSpacing: 24,
                mainAxisSpacing: 24,
              ),
              itemBuilder: (context, index) => ProductCard(product: catalog.products[index]),
            ),
        ],
      ),
    );
  }
}

