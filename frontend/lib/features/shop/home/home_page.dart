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
/// product_carousel y static_content.
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
    if (catalog.categories.isEmpty) await catalog.loadCategories();
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
      return const _HomeSkeleton();
    }
    if (_sections.isEmpty) {
      return RefreshIndicator(
        onRefresh: _load,
        child: ListView(
          children: const [
            SizedBox(height: 140),
            Center(
              child: Text(
                'Welcome to EzCommerce',
                style: TextStyle(
                    fontFamily: EzTheme.serif, fontSize: 36, color: AppColors.navyBlue),
              ),
            ),
          ],
        ),
      );
    }
    return RefreshIndicator(
      onRefresh: _load,
      child: SingleChildScrollView(
        physics: const AlwaysScrollableScrollPhysics(),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: _sections.map((section) => _SectionRenderer(section: section)).toList(),
        ),
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
      case 'footer_links':
        return const SizedBox.shrink();
      default:
        return const SizedBox.shrink();
    }
  }
}

/// Ancho máximo de las secciones del home, con márgenes tipo Bagisto.
class _SectionWidth extends StatelessWidget {
  const _SectionWidth({required this.child});
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.of(context).size.width;
    final horizontal = width >= 1200 ? 60.0 : (width >= 768 ? 32.0 : 16.0);
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 1440),
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: horizontal),
          child: child,
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Hero carousel
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

  List<Map<String, dynamic>> get slides => widget.slides
      .whereType<Map>()
      .map((e) => Map<String, dynamic>.from(e))
      .toList();

  @override
  void initState() {
    super.initState();
    _timer = Timer.periodic(const Duration(seconds: 6), (_) {
      if (!mounted || slides.length < 2) return;
      final next = (_index + 1) % slides.length;
      _controller.animateToPage(
        next,
        duration: const Duration(milliseconds: 650),
        curve: Curves.easeOutCubic,
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
    final list = slides;
    if (list.isEmpty) return const SizedBox.shrink();
    final width = MediaQuery.of(context).size.width;
    final height = width >= 1200
        ? (width / 2.65).clamp(420.0, 660.0)
        : (width >= 768 ? 470.0 : 500.0);

    return SizedBox(
      height: height,
      child: Stack(
        fit: StackFit.expand,
        children: [
          PageView.builder(
            controller: _controller,
            itemCount: list.length,
            onPageChanged: (index) => setState(() => _index = index),
            itemBuilder: (context, index) => _HeroSlide(slide: list[index], height: height),
          ),
          if (list.length > 1) ...[
            Positioned(
              left: 20,
              top: 0,
              bottom: 0,
              child: Center(
                child: _HeroArrow(
                  icon: Icons.arrow_back_ios_new,
                  onTap: () => _controller.previousPage(
                      duration: const Duration(milliseconds: 550), curve: Curves.easeOutCubic),
                ),
              ),
            ),
            Positioned(
              right: 20,
              top: 0,
              bottom: 0,
              child: Center(
                child: _HeroArrow(
                  icon: Icons.arrow_forward_ios,
                  onTap: () => _controller.nextPage(
                      duration: const Duration(milliseconds: 550), curve: Curves.easeOutCubic),
                ),
              ),
            ),
            Positioned(
              left: 0,
              right: 0,
              bottom: 18,
              child: Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: List.generate(list.length, (index) {
                  final active = index == _index;
                  return GestureDetector(
                    onTap: () => _controller.animateToPage(index,
                        duration: const Duration(milliseconds: 500),
                        curve: Curves.easeOutCubic),
                    child: AnimatedContainer(
                      duration: const Duration(milliseconds: 220),
                      width: active ? 26 : 8,
                      height: 8,
                      margin: const EdgeInsets.symmetric(horizontal: 4),
                      decoration: BoxDecoration(
                        color: active ? Colors.white : Colors.white.withValues(alpha: .45),
                        borderRadius: BorderRadius.circular(999),
                      ),
                    ),
                  );
                }),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _HeroSlide extends StatelessWidget {
  const _HeroSlide({required this.slide, required this.height});
  final Map<String, dynamic> slide;
  final double height;

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.of(context).size.width;
    final image = apiUrlRelative((slide['image'] ?? '') as String?);
    final title = (slide['title'] ?? '') as String;
    final subtitle = (slide['subtitle'] ?? '') as String;
    final button = slide['button'] is Map ? Map<String, dynamic>.from(slide['button'] as Map) : null;
    final link = (button?['link'] ?? slide['link'] ?? '/search') as String;
    final buttonTitle = (button?['title'] ?? 'Shop now') as String;
    final isMobile = width < 768;

    return Stack(
      fit: StackFit.expand,
      children: [
        if (image != null && image.isNotEmpty)
          Image.network(
            image,
            fit: BoxFit.cover,
            filterQuality: FilterQuality.high,
            errorBuilder: (_, _, _) => Container(color: AppColors.navyBlue),
          )
        else
          const DecoratedBox(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [AppColors.navyBlue, Color(0xFF22306F)],
                begin: Alignment.centerLeft,
                end: Alignment.centerRight,
              ),
            ),
          ),
        // Oscurece la zona del texto sin apagar la fotografía.
        DecoratedBox(
          decoration: BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.centerLeft,
              end: Alignment.centerRight,
              colors: [
                AppColors.navyBlue.withValues(alpha: isMobile ? .82 : .72),
                AppColors.navyBlue.withValues(alpha: isMobile ? .5 : .28),
                Colors.transparent,
              ],
              stops: isMobile ? const [0, .72, 1] : const [0, .55, 1],
            ),
          ),
        ),
        Positioned.fill(
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1440),
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: isMobile ? 28 : 72),
                child: Align(
                  alignment: Alignment.centerLeft,
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 560),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'EZCOMMERCE  ·  NEW SEASON',
                          style: TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: isMobile ? 10 : 12,
                            letterSpacing: 2.6,
                            fontWeight: FontWeight.w500,
                            color: Colors.white.withValues(alpha: .78),
                          ),
                        ),
                        SizedBox(height: isMobile ? 12 : 18),
                        Text(
                          title,
                          style: TextStyle(
                            fontFamily: EzTheme.serif,
                            fontSize: isMobile ? 38 : 56,
                            height: 1.02,
                            color: Colors.white,
                          ),
                        ),
                        if (subtitle.isNotEmpty) ...[
                          SizedBox(height: isMobile ? 10 : 14),
                          Text(
                            subtitle,
                            style: TextStyle(
                              fontFamily: EzTheme.sans,
                              fontSize: isMobile ? 14 : 16,
                              height: 1.55,
                              color: Colors.white.withValues(alpha: .88),
                            ),
                          ),
                        ],
                        SizedBox(height: isMobile ? 22 : 30),
                        SizedBox(
                          height: 48,
                          child: TextButton(
                            onPressed: () => context.go(link),
                            style: TextButton.styleFrom(
                              backgroundColor: Colors.white,
                              foregroundColor: AppColors.navyBlue,
                              padding: const EdgeInsets.symmetric(horizontal: 28),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                              ),
                            ),
                            child: Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Text(
                                  buttonTitle,
                                  style: const TextStyle(
                                    fontFamily: EzTheme.sans,
                                    fontSize: 14,
                                    fontWeight: FontWeight.w500,
                                    color: AppColors.navyBlue,
                                  ),
                                ),
                                const SizedBox(width: 10),
                                const Icon(Icons.arrow_forward, size: 16),
                              ],
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _HeroArrow extends StatefulWidget {
  const _HeroArrow({required this.icon, required this.onTap});
  final IconData icon;
  final VoidCallback onTap;

  @override
  State<_HeroArrow> createState() => _HeroArrowState();
}

class _HeroArrowState extends State<_HeroArrow> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 180),
          width: 46,
          height: 46,
          decoration: BoxDecoration(
            color: Colors.black.withValues(alpha: _hovering ? .65 : .35),
            shape: BoxShape.circle,
          ),
          child: Icon(widget.icon, size: 15, color: Colors.white),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Categorías
// ---------------------------------------------------------------------------

class CategoryCarousel extends StatelessWidget {
  const CategoryCarousel({super.key, required this.title});
  final String title;

  @override
  Widget build(BuildContext context) {
    final categories = context.watch<CatalogProvider>().categories;
    if (categories.isEmpty) return const SizedBox.shrink();
    final width = MediaQuery.of(context).size.width;
    final size = width >= 1200 ? 138.0 : (width >= 768 ? 112.0 : 88.0);

    return _SectionWidth(
      child: Padding(
        padding: const EdgeInsets.only(top: 72),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: TextStyle(
                      fontFamily: EzTheme.serif,
                      fontSize: width < 768 ? 26 : 34,
                      color: AppColors.gray900,
                    ),
                  ),
                ),
                TextButton(
                  onPressed: () => context.go('/search'),
                  child: const Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text('View all',
                          style: TextStyle(fontFamily: EzTheme.sans, fontSize: 13)),
                      SizedBox(width: 6),
                      Icon(Icons.arrow_forward, size: 15),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 28),
            Wrap(
              spacing: width < 768 ? 22 : 46,
              runSpacing: 26,
              children: [
                for (final category in categories.take(8))
                  _CategoryTile(category: category, size: size),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _CategoryTile extends StatefulWidget {
  const _CategoryTile({required this.category, required this.size});
  final ShopCategory category;
  final double size;

  @override
  State<_CategoryTile> createState() => _CategoryTileState();
}

class _CategoryTileState extends State<_CategoryTile> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final image = apiUrlRelative(widget.category.bannerUrl);
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: () => context.go('/category/${widget.category.id}'),
        child: SizedBox(
          width: widget.size + 12,
          child: Column(
            children: [
              AnimatedContainer(
                duration: const Duration(milliseconds: 220),
                width: widget.size,
                height: widget.size,
                padding: const EdgeInsets.all(3),
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: Colors.white,
                  border: Border.all(
                    color: _hovering ? AppColors.navyBlue : AppColors.zinc200,
                    width: _hovering ? 2 : 1,
                  ),
                  boxShadow: _hovering
                      ? const [
                          BoxShadow(
                              color: Color(0x1A000000),
                              blurRadius: 16,
                              offset: Offset(0, 7)),
                        ]
                      : null,
                ),
                child: ClipOval(
                  child: AnimatedScale(
                    duration: const Duration(milliseconds: 260),
                    scale: _hovering ? 1.06 : 1,
                    child: image != null
                        ? Image.network(
                            image,
                            fit: BoxFit.cover,
                            errorBuilder: (_, _, _) =>
                                const ColoredBox(color: AppColors.zinc100, child: _CategoryIcon()),
                          )
                        : const ColoredBox(color: AppColors.zinc100, child: _CategoryIcon()),
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Text(
                widget.category.name,
                textAlign: TextAlign.center,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontFamily: EzTheme.sans,
                  fontSize: MediaQuery.of(context).size.width < 768 ? 13 : 15,
                  fontWeight: FontWeight.w500,
                  color: _hovering ? AppColors.navyBlue : AppColors.gray900,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _CategoryIcon extends StatelessWidget {
  const _CategoryIcon();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Icon(Icons.category_outlined, size: 30, color: AppColors.zinc400),
    );
  }
}

// ---------------------------------------------------------------------------
// Carrusel de productos
// ---------------------------------------------------------------------------

class ProductsCarousel extends StatefulWidget {
  const ProductsCarousel({super.key, required this.title});
  final String title;

  @override
  State<ProductsCarousel> createState() => _ProductsCarouselState();
}

class _ProductsCarouselState extends State<ProductsCarousel> {
  final ScrollController _scroll = ScrollController();
  List<Product> _products = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    final products = await context.read<CatalogProvider>().loadHomeProducts();
    if (mounted) {
      setState(() {
        _products = products;
        _loading = false;
      });
    }
  }

  void _scrollBy(double delta) {
    if (!_scroll.hasClients) return;
    final target = (_scroll.offset + delta).clamp(0.0, _scroll.position.maxScrollExtent);
    _scroll.animateTo(target,
        duration: const Duration(milliseconds: 450), curve: Curves.easeOutCubic);
  }

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.of(context).size.width;
    final narrow = width < 768;
    final cardWidth = narrow ? 208.0 : 291.0;
    final cardHeight = cardWidth * (300 / 291) + 148;
    if (!_loading && _products.isEmpty) return const SizedBox.shrink();

    return _SectionWidth(
      child: Padding(
        padding: const EdgeInsets.only(top: 76),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    widget.title,
                    style: TextStyle(
                      fontFamily: EzTheme.serif,
                      fontSize: narrow ? 26 : 34,
                      color: AppColors.gray900,
                    ),
                  ),
                ),
                if (!narrow) ...[
                  TextButton(
                    onPressed: () => context.go('/search'),
                    child: const Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text('View all',
                            style: TextStyle(fontFamily: EzTheme.sans, fontSize: 13)),
                        SizedBox(width: 6),
                        Icon(Icons.arrow_forward, size: 15),
                      ],
                    ),
                  ),
                  const SizedBox(width: 24),
                  _RoundArrow(icon: Icons.arrow_back, onTap: () => _scrollBy(-323)),
                  const SizedBox(width: 10),
                  _RoundArrow(icon: Icons.arrow_forward, onTap: () => _scrollBy(323)),
                ],
              ],
            ),
            const SizedBox(height: 26),
            if (_loading)
              SizedBox(
                height: cardHeight,
                child: ListView.separated(
                  scrollDirection: Axis.horizontal,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: narrow ? 2 : 4,
                  separatorBuilder: (_, _) => const SizedBox(width: 28),
                  itemBuilder: (_, _) => const _CardSkeleton(),
                ),
              )
            else
              SizedBox(
                height: cardHeight,
                child: ListView.separated(
                  controller: _scroll,
                  scrollDirection: Axis.horizontal,
                  clipBehavior: Clip.none,
                  itemCount: _products.length,
                  separatorBuilder: (_, _) => const SizedBox(width: 28),
                  itemBuilder: (context, index) => ProductCard(
                    product: _products[index],
                    width: cardWidth,
                  ),
                ),
              ),
            if (narrow) ...[
              const SizedBox(height: 22),
              Center(
                child: SecondaryButton(
                  label: 'View All',
                  onPressed: () => context.go('/search'),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _RoundArrow extends StatefulWidget {
  const _RoundArrow({required this.icon, required this.onTap});
  final IconData icon;
  final VoidCallback onTap;

  @override
  State<_RoundArrow> createState() => _RoundArrowState();
}

class _RoundArrowState extends State<_RoundArrow> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 180),
          width: 42,
          height: 42,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: _hovering ? AppColors.navyBlue : Colors.white,
            border: Border.all(color: AppColors.navyBlue),
          ),
          child: Icon(
            widget.icon,
            size: 17,
            color: _hovering ? Colors.white : AppColors.navyBlue,
          ),
        ),
      ),
    );
  }
}

class _CardSkeleton extends StatelessWidget {
  const _CardSkeleton();

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(builder: (context, constraints) {
      return Container(
        width: 291,
        decoration: BoxDecoration(
          color: AppColors.zinc50,
          borderRadius: BorderRadius.circular(EzTheme.radiusLg),
        ),
        child: Column(
          children: [
            Expanded(
              child: Container(
                decoration: BoxDecoration(
                  color: AppColors.zinc100,
                  borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                ),
              ),
            ),
            const SizedBox(height: 12),
            Container(height: 14, width: 220, color: AppColors.zinc100),
            const SizedBox(height: 10),
            Container(height: 14, width: 90, color: AppColors.zinc100),
            const SizedBox(height: 22),
          ],
        ),
      );
    });
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
    final width = MediaQuery.of(context).size.width;
    final narrow = width < 768;
    final button =
        options['button'] is Map ? Map<String, dynamic>.from(options['button'] as Map) : null;
    final title = (options['title'] ?? '') as String;
    final subtitle = (options['subtitle'] ?? '') as String;
    if (title.isEmpty && subtitle.isEmpty) return const SizedBox.shrink();

    return _SectionWidth(
      child: Padding(
        padding: const EdgeInsets.only(top: 84),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(EzTheme.radius2Xl),
          child: SizedBox(
            height: narrow ? 360 : 320,
            child: Stack(
              fit: StackFit.expand,
              children: [
                Image.network(
                  '${apiUrlRelative('/storage/categories/women.jpg')}',
                  fit: BoxFit.cover,
                  alignment: const Alignment(.65, -.1),
                  errorBuilder: (_, _, _) => const ColoredBox(color: AppColors.navyBlue),
                ),
                DecoratedBox(
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.centerLeft,
                      end: Alignment.centerRight,
                      colors: [
                        AppColors.navyBlue,
                        AppColors.navyBlue.withValues(alpha: .88),
                        AppColors.navyBlue.withValues(alpha: .12),
                      ],
                      stops: narrow ? const [0, .75, 1] : const [0, .48, 1],
                    ),
                  ),
                ),
                Padding(
                  padding: EdgeInsets.symmetric(horizontal: narrow ? 28 : 56),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 520),
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'LIMITED TIME',
                            style: TextStyle(
                              fontFamily: EzTheme.sans,
                              fontSize: 11,
                              letterSpacing: 2.4,
                              fontWeight: FontWeight.w500,
                              color: Colors.white.withValues(alpha: .7),
                            ),
                          ),
                          const SizedBox(height: 12),
                          Text(
                            title,
                            style: TextStyle(
                              fontFamily: EzTheme.serif,
                              fontSize: narrow ? 30 : 44,
                              height: 1.06,
                              color: Colors.white,
                            ),
                          ),
                          if (subtitle.isNotEmpty) ...[
                            const SizedBox(height: 10),
                            Text(
                              subtitle,
                              style: TextStyle(
                                fontFamily: EzTheme.sans,
                                fontSize: narrow ? 13 : 15,
                                color: Colors.white.withValues(alpha: .82),
                              ),
                            ),
                          ],
                          const SizedBox(height: 24),
                          if (button != null)
                            SizedBox(
                              height: 48,
                              child: TextButton(
                                onPressed: () => context.go((button['link'] ?? '/search') as String),
                                style: TextButton.styleFrom(
                                  backgroundColor: Colors.white,
                                  foregroundColor: AppColors.navyBlue,
                                  padding: const EdgeInsets.symmetric(horizontal: 28),
                                  shape: RoundedRectangleBorder(
                                    borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                                  ),
                                ),
                                child: Text(
                                  (button['title'] ?? 'Shop now') as String,
                                  style: const TextStyle(
                                    fontFamily: EzTheme.sans,
                                    fontSize: 14,
                                    fontWeight: FontWeight.w500,
                                    color: AppColors.navyBlue,
                                  ),
                                ),
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Skeleton del home
// ---------------------------------------------------------------------------

class _HomeSkeleton extends StatelessWidget {
  const _HomeSkeleton();

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        children: [
          Container(height: 480, color: AppColors.zinc100),
          const SizedBox(height: 48),
          const _SectionWidth(
            child: Row(
              children: [
                _CircleSkeleton(),
                SizedBox(width: 40),
                _CircleSkeleton(),
                SizedBox(width: 40),
                _CircleSkeleton(),
                SizedBox(width: 40),
                _CircleSkeleton(),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _CircleSkeleton extends StatelessWidget {
  const _CircleSkeleton();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 120,
      height: 120,
      decoration: const BoxDecoration(color: AppColors.zinc100, shape: BoxShape.circle),
    );
  }
}
