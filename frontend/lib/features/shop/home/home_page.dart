import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';
import '../catalog/product_card.dart';

/// Home por secciones (`theme_sections`): image_carousel, category_carousel,
/// product_carousel, static_content.
class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  List<HomeSection> _sections = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final catalog = context.read<CatalogProvider>();
    final sections = await catalog.loadHomeSections();
    if (mounted) {
      setState(() {
        _sections = sections;
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(color: AppColors.navyBlue));
    }
    if (_sections.isEmpty) {
      return RefreshIndicator(
        onRefresh: _load,
        child: ListView(
          children: const [
            SizedBox(height: 120),
            Center(
              child: Text('Welcome to EzCommerce',
                  style: TextStyle(fontFamily: 'DM Serif Display', fontSize: 32, color: AppColors.navyBlue)),
            ),
          ],
        ),
      );
    }
    return SingleChildScrollView(
      child: Column(
        children: _sections.map((section) => _SectionRenderer(section: section)).toList(),
      ),
    );
  }
}

class _SectionRenderer extends StatelessWidget {
  const _SectionRenderer({required this.section});
  final HomeSection section;

  @override
  Widget build(BuildContext context) {
    switch (section.type) {
      case 'image_carousel':
        return HeroCarousel(slides: (section.options['slides'] as List?) ?? const []);
      case 'static_content':
        return OfferBanner(options: section.options);
      case 'category_carousel':
        return CategoryCarousel(title: section.options['title'] ?? section.name);
      case 'product_carousel':
        return ProductsCarousel(title: section.options['title'] ?? section.name);
      case 'services_content':
        return const SizedBox.shrink();
      case 'footer_links':
        return const SizedBox.shrink();
      default:
        return const SizedBox.shrink();
    }
  }
}

// ---------------------------------------------------------------------------
// Hero carousel (aspect 2.743/1, dots, autoplay)
// ---------------------------------------------------------------------------

class HeroCarousel extends StatefulWidget {
  const HeroCarousel({super.key, required this.slides});
  final List<dynamic> slides;

  @override
  State<HeroCarousel> createState() => _HeroCarouselState();
}

class _HeroCarouselState extends State<HeroCarousel> {
  final PageController _controller = PageController();
  Timer? _timer;
  int _index = 0;

  @override
  void initState() {
    super.initState();
    _timer = Timer.periodic(const Duration(seconds: 5), (_) {
      if (!mounted || widget.slides.isEmpty) return;
      final next = (_index + 1) % widget.slides.length;
      _controller.animateToPage(
        next,
        duration: const Duration(milliseconds: 700),
        curve: Curves.easeInOut,
      );
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final slides = widget.slides
        .map((e) => e is Map ? Map<String, dynamic>.from(e) : <String, dynamic>{})
        .toList();
    if (slides.isEmpty) return const SizedBox.shrink();
    return Stack(
      alignment: Alignment.bottomCenter,
      children: [
        AspectRatio(
          aspectRatio: 2.743,
          child: PageView.builder(
            controller: _controller,
            itemCount: slides.length,
            onPageChanged: (index) => setState(() => _index = index),
            itemBuilder: (context, index) {
              final slide = slides[index];
              return Container(
                color: AppColors.offerBanner,
                alignment: Alignment.center,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      slide['title'] ?? '',
                      style: const TextStyle(
                        fontFamily: 'DM Serif Display',
                        fontSize: 42,
                        color: AppColors.navyBlue,
                      ),
                    ),
                    const SizedBox(height: 12),
                    if ((slide['link'] ?? '').isNotEmpty)
                      TextButton(
                        onPressed: () => context.go(slide['link']),
                        child: const Text('Shop now',
                            style: TextStyle(
                                fontFamily: 'Poppins',
                                fontSize: 14,
                                color: AppColors.navyBlue,
                                decoration: TextDecoration.underline)),
                      ),
                  ],
                ),
              );
            },
          ),
        ),
        // Dots
        Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: List.generate(slides.length, (index) {
              return Container(
                width: 10,
                height: 10,
                margin: const EdgeInsets.symmetric(horizontal: 4),
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: index == _index
                      ? AppColors.navyBlue
                      : AppColors.gray500.withOpacity(0.3),
                ),
              );
            }),
          ),
        ),
        // Flechas
        Positioned(
          left: 24,
          top: 0,
          bottom: 0,
          child: Center(
            child: _CarouselArrow(
              icon: Icons.arrow_back_ios_new,
              onTap: () => _controller.previousPage(
                  duration: const Duration(milliseconds: 700), curve: Curves.easeInOut),
            ),
          ),
        ),
        Positioned(
          right: 24,
          top: 0,
          bottom: 0,
          child: Center(
            child: _CarouselArrow(
              icon: Icons.arrow_forward_ios,
              onTap: () => _controller.nextPage(
                  duration: const Duration(milliseconds: 700), curve: Curves.easeInOut),
            ),
          ),
        ),
      ],
    );
  }
}

class _CarouselArrow extends StatelessWidget {
  const _CarouselArrow({required this.icon, required this.onTap});
  final IconData icon;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      customBorder: const CircleBorder(),
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: Colors.black.withOpacity(0.3),
          shape: BoxShape.circle,
        ),
        child: Icon(icon, size: 16, color: Colors.white),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Banner de oferta (static_content)
// ---------------------------------------------------------------------------

class OfferBanner extends StatelessWidget {
  const OfferBanner({super.key, required this.options});
  final Map<String, dynamic> options;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      margin: const EdgeInsets.only(top: 32),
      padding: const EdgeInsets.symmetric(vertical: 28),
      color: AppColors.offerBanner,
      alignment: Alignment.center,
      child: Column(
        children: [
          Text(
            options['title'] ?? '',
            textAlign: TextAlign.center,
            style: const TextStyle(
              fontFamily: 'DM Serif Display',
              fontSize: 24,
              color: AppColors.gray900,
            ),
          ),
          if ((options['subtitle'] ?? '').isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                options['subtitle'],
                style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.zinc600),
              ),
            ),
          if (options['button'] is Map)
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: PrimaryButton(
                label: (options['button'] as Map)['title'] ?? 'Shop now',
                onPressed: () => context.go((options['button'] as Map)['link'] ?? '/'),
              ),
            ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// ShopCategory carousel (círculos)
// ---------------------------------------------------------------------------

class CategoryCarousel extends StatefulWidget {
  const CategoryCarousel({super.key, required this.title});
  final String title;

  @override
  State<CategoryCarousel> createState() => _CategoryCarouselState();
}

class _CategoryCarouselState extends State<CategoryCarousel> {
  List<ShopCategory> _categories = [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final catalog = context.read<CatalogProvider>();
    if (catalog.categories.isNotEmpty && mounted) {
      setState(() => _categories = catalog.categories);
    }
  }

  @override
  Widget build(BuildContext context) {
    final catalog = context.watch<CatalogProvider>();
    final categories = catalog.categories;
    if (categories.isEmpty) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 60),
      margin: const EdgeInsets.only(top: 48),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            widget.title,
            style: const TextStyle(
              fontFamily: 'DM Serif Display',
              fontSize: 30,
              color: AppColors.gray900,
            ),
          ),
          const SizedBox(height: 24),
          SizedBox(
            height: 140,
            child: ListView.separated(
              scrollDirection: Axis.horizontal,
              itemCount: categories.length,
              separatorBuilder: (_, __) => const SizedBox(width: 32),
              itemBuilder: (context, index) {
                final category = categories[index];
                return GestureDetector(
                  onTap: () => context.go('/category/${category.id}'),
                  child: Column(
                    children: [
                      Container(
                        width: 90,
                        height: 90,
                        decoration: const BoxDecoration(
                          color: AppColors.zinc100,
                          shape: BoxShape.circle,
                        ),
                        child: Icon(Icons.category_outlined, size: 36, color: AppColors.zinc500),
                      ),
                      const SizedBox(height: 10),
                      SizedBox(
                        width: 90,
                        child: Text(
                          category.name,
                          textAlign: TextAlign.center,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(fontFamily: 'Poppins', fontSize: 13),
                        ),
                      ),
                    ],
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Products carousel
// ---------------------------------------------------------------------------

class ProductsCarousel extends StatefulWidget {
  const ProductsCarousel({super.key, required this.title});
  final String title;

  @override
  State<ProductsCarousel> createState() => _ProductsCarouselState();
}

class _ProductsCarouselState extends State<ProductsCarousel> {
  List<Product> _products = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final products = await context.read<CatalogProvider>().loadHomeProducts();
    if (mounted) setState(() {
      _products = products;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    final isNarrow = MediaQuery.of(context).size.width < 1024;
    return Container(
      padding: EdgeInsets.symmetric(horizontal: isNarrow ? 16 : 60),
      margin: const EdgeInsets.only(top: 56),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                widget.title,
                style: const TextStyle(
                  fontFamily: 'DM Serif Display',
                  fontSize: 30,
                  color: AppColors.gray900,
                ),
              ),
              if (!isNarrow)
                TextButton(
                  onPressed: () => context.go('/search'),
                  child: const Text('View All',
                      style: TextStyle(fontFamily: 'Poppins', fontSize: 14, color: AppColors.navyBlue)),
                ),
            ],
          ),
          const SizedBox(height: 24),
          if (_loading)
            const Center(child: CircularProgressIndicator(color: AppColors.navyBlue))
          else
            SizedBox(
              height: 360,
              child: ListView.separated(
                scrollDirection: Axis.horizontal,
                itemCount: _products.length,
                separatorBuilder: (_, __) => const SizedBox(width: 24),
                itemBuilder: (context, index) => SizedBox(
                  width: 280,
                  child: ProductCard(product: _products[index]),
                ),
              ),
            ),
          if (!isNarrow) ...[
            const SizedBox(height: 24),
            Center(
              child: SecondaryButton(
                label: 'View All',
                onPressed: () => context.go('/search'),
              ),
            ),
          ],
        ],
      ),
    );
  }
}
