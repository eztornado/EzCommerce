import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/api/api_client.dart';
import '../../../core/constants/app_colors.dart';
import '../../../core/models/models.dart';
import '../../../core/state/app_state.dart';
import '../../../core/theme/ez_theme.dart';

/// Layout master del shop (Bagisto `components/layouts/index.blade.php`):
/// header sticky + contenido + tira de servicios + footer.
/// Header desktop (>=1024) con topbar/megamenú; móvil con drawer.
class ShopShell extends StatefulWidget {
  const ShopShell({super.key, required this.child});
  final Widget child;

  @override
  State<ShopShell> createState() => _ShopShellState();
}

class _ShopShellState extends State<ShopShell> {
  final GlobalKey<ScaffoldState> _scaffoldKey = GlobalKey<ScaffoldState>();

  @override
  Widget build(BuildContext context) {
    final isDesktop = MediaQuery.of(context).size.width >= 1024;
    return Scaffold(
      key: _scaffoldKey,
      backgroundColor: Colors.white,
      drawer: isDesktop ? null : const MobileDrawer(),
      body: Column(
        children: [
          const ShopHeader(),
          Expanded(child: widget.child),
          const ServicesStrip(),
          const ShopFooter(),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

class ShopHeader extends StatelessWidget {
  const ShopHeader({super.key});

  @override
  Widget build(BuildContext context) {
    final isDesktop = MediaQuery.of(context).size.width >= 1024;
    return Material(
      color: Colors.white,
      elevation: 0.5,
      child: isDesktop ? const DesktopHeader() : const MobileHeader(),
    );
  }
}

class _HeaderLogo extends StatelessWidget {
  const _HeaderLogo();

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: () => context.go('/'),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.baseline,
          textBaseline: TextBaseline.alphabetic,
          children: const [
            Text(
              'EzCommerce',
              style: TextStyle(
                fontFamily: EzTheme.serif,
                fontSize: 27,
                height: 1,
                color: AppColors.navyBlue,
              ),
            ),
            Text(
              '.',
              style: TextStyle(
                fontFamily: EzTheme.serif,
                fontSize: 30,
                height: 1,
                color: AppColors.darkPink,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class DesktopHeader extends StatelessWidget {
  const DesktopHeader({super.key});

  @override
  Widget build(BuildContext context) {
    final catalog = context.watch<CatalogProvider>();
    final offer = (catalog.context?['header_offer'] as String?)?.trim();
    return Column(
      children: [
        // Topbar: moneda | oferta | idioma
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 60),
          decoration: const BoxDecoration(
            border: Border(bottom: BorderSide(color: AppColors.zinc200)),
          ),
          child: SizedBox(
            height: 44,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                _CurrencySelector(catalog: catalog),
                GestureDetector(
                  onTap: () => context.go('/search'),
                  child: Text.rich(
                    TextSpan(
                      children: [
                        TextSpan(
                          text: offer == null || offer.isEmpty
                              ? 'Get ready for the new season — up to 30% off. '
                              : '$offer ',
                        ),
                        const TextSpan(
                          text: 'Shop now',
                          style: TextStyle(decoration: TextDecoration.underline),
                        ),
                      ],
                    ),
                    style: const TextStyle(
                      fontFamily: EzTheme.sans,
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                      color: AppColors.gray900,
                    ),
                  ),
                ),
                _LocaleSelector(catalog: catalog),
              ],
            ),
          ),
        ),
        // Barra principal: logo | categorías | búsqueda + iconos
        Container(
          height: 84,
          padding: const EdgeInsets.symmetric(horizontal: 60),
          decoration: const BoxDecoration(
            border: Border(bottom: BorderSide(color: AppColors.zinc200)),
          ),
          child: Row(
            children: [
              const _HeaderLogo(),
              const SizedBox(width: 44),
              Expanded(child: CategoryMenu(categories: catalog.categories)),
              const SizedBox(width: 24),
              const HeaderSearchBox(),
              const SizedBox(width: 22),
              const HeaderIcons(),
            ],
          ),
        ),
      ],
    );
  }
}

class CategoryMenu extends StatelessWidget {
  const CategoryMenu({super.key, required this.categories});
  final List<ShopCategory> categories;

  @override
  Widget build(BuildContext context) {
    if (categories.isEmpty) {
      return const SizedBox.shrink();
    }
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final category in categories.take(6)) _CategoryMenuItem(category: category),
      ],
    );
  }
}

class _CategoryMenuItem extends StatefulWidget {
  const _CategoryMenuItem({required this.category});
  final ShopCategory category;

  @override
  State<_CategoryMenuItem> createState() => _CategoryMenuItemState();
}

class _CategoryMenuItemState extends State<_CategoryMenuItem> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final active = GoRouterState.of(context).uri.path == '/category/${widget.category.id}';
    final highlighted = _hovering || active;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: () => context.go('/category/${widget.category.id}'),
        child: Container(
          height: 84,
          padding: const EdgeInsets.symmetric(horizontal: 18),
          alignment: Alignment.center,
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                width: 3,
                color: highlighted ? AppColors.navyBlue : Colors.transparent,
              ),
            ),
          ),
          child: Text(
            widget.category.name.toUpperCase(),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontFamily: EzTheme.sans,
              fontSize: 12,
              letterSpacing: .4,
              fontWeight: FontWeight.w500,
              color: highlighted ? AppColors.navyBlue : AppColors.gray900,
            ),
          ),
        ),
      ),
    );
  }
}

class HeaderSearchBox extends StatefulWidget {
  const HeaderSearchBox({super.key});

  @override
  State<HeaderSearchBox> createState() => _HeaderSearchBoxState();
}

class _HeaderSearchBoxState extends State<HeaderSearchBox> {
  final TextEditingController _controller = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 430,
      height: 44,
      child: TextField(
        controller: _controller,
        textInputAction: TextInputAction.search,
        onSubmitted: (query) => context.go('/search?q=${Uri.encodeQueryComponent(query)}'),
        style: const TextStyle(fontFamily: EzTheme.sans, fontSize: 13),
        decoration: InputDecoration(
          hintText: 'Search products here',
          prefixIcon: const Icon(Icons.search, size: 19, color: AppColors.zinc500),
          prefixIconConstraints: const BoxConstraints(minWidth: 44),
          contentPadding: const EdgeInsets.symmetric(vertical: 11),
          filled: true,
          fillColor: AppColors.zinc100,
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
            borderSide: BorderSide.none,
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
            borderSide: BorderSide.none,
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(EzTheme.radiusLg),
            borderSide: const BorderSide(color: AppColors.zinc400),
          ),
        ),
      ),
    );
  }
}

class HeaderIcons extends StatelessWidget {
  const HeaderIcons({super.key});

  @override
  Widget build(BuildContext context) {
    final cart = context.watch<CartProvider>();
    final auth = context.watch<AuthProvider>();
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _HeaderIconButton(
          icon: Icons.compare_arrows,
          tooltip: 'Compare',
          onTap: () {},
        ),
        // Mini carrito con badge
        Stack(
          clipBehavior: Clip.none,
          children: [
            _HeaderIconButton(
              icon: Icons.shopping_bag_outlined,
              tooltip: 'Cart',
              onTap: () => showMiniCart(context),
            ),
            if (cart.itemCount > 0)
              Positioned(
                right: 2,
                top: 3,
                child: Container(
                  constraints: const BoxConstraints(minWidth: 17),
                  height: 17,
                  padding: const EdgeInsets.symmetric(horizontal: 4),
                  decoration: BoxDecoration(
                    color: AppColors.navyBlue,
                    borderRadius: BorderRadius.circular(999),
                  ),
                  alignment: Alignment.center,
                  child: Text(
                    '${cart.itemCount}',
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 9,
                      height: 1,
                      fontWeight: FontWeight.w600,
                      fontFamily: EzTheme.sans,
                    ),
                  ),
                ),
              ),
          ],
        ),
        // Cuenta
        MenuAnchor(
          builder: (context, controller, child) {
            return _HeaderIconButton(
              icon: Icons.person_outline,
              tooltip: 'Account',
              onTap: controller.open,
            );
          },
          menuChildren: [
            Container(
              width: 260,
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    auth.isLoggedIn ? 'Hello, ${auth.customer!.name}' : 'Welcome Guest!',
                    style: const TextStyle(
                      fontFamily: 'DM Serif Display',
                      fontSize: 20,
                      color: AppColors.navyBlue,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Manage Cart, Orders & Wishlist',
                    style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500),
                  ),
                  const Divider(height: 24),
                  if (!auth.isLoggedIn) ...[
                    Row(
                      children: [
                        Expanded(
                          child: PrimaryButton(
                            label: 'Sign In',
                            onPressed: () {
                              Navigator.of(context, rootNavigator: true).pop();
                              context.go('/login');
                            },
                          ),
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: SecondaryButton(
                            label: 'Sign Up',
                            onPressed: () {
                              Navigator.of(context, rootNavigator: true).pop();
                              context.go('/register');
                            },
                          ),
                        ),
                      ],
                    ),
                  ] else ...[
                    _accountLink(context, 'Profile', '/account/profile'),
                    _accountLink(context, 'Orders', '/account/orders'),
                    _accountLink(context, 'Wishlist', '/account/wishlist'),
                    _accountLink(context, 'Addresses', '/account/addresses'),
                    ListTile(
                      dense: true,
                      contentPadding: const EdgeInsets.symmetric(horizontal: 8),
                      leading: const Icon(Icons.logout, size: 18, color: AppColors.zinc600),
                      title: Text('Logout', style: TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                      onTap: () async {
                        await auth.logout();
                        if (context.mounted) {
                          Navigator.of(context, rootNavigator: true).pop();
                          context.go('/');
                        }
                      },
                    ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _accountLink(BuildContext context, String label, String route) {
    return ListTile(
      dense: true,
      contentPadding: const EdgeInsets.symmetric(horizontal: 8),
      title: Text(label, style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
      onTap: () {
        Navigator.of(context, rootNavigator: true).pop();
        context.go(route);
      },
    );
  }
}

class _HeaderIconButton extends StatefulWidget {
  const _HeaderIconButton({
    required this.icon,
    required this.tooltip,
    required this.onTap,
  });

  final IconData icon;
  final String tooltip;
  final VoidCallback onTap;

  @override
  State<_HeaderIconButton> createState() => _HeaderIconButtonState();
}

class _HeaderIconButtonState extends State<_HeaderIconButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovering = true),
        onExit: (_) => setState(() => _hovering = false),
        child: GestureDetector(
          onTap: widget.onTap,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 160),
            width: 40,
            height: 40,
            margin: const EdgeInsets.symmetric(horizontal: 2),
            decoration: BoxDecoration(
              color: _hovering ? AppColors.zinc100 : Colors.transparent,
              shape: BoxShape.circle,
            ),
            child: Icon(widget.icon, size: 22, color: AppColors.gray900),
          ),
        ),
      ),
    );
  }
}

class MobileHeader extends StatelessWidget {
  const MobileHeader({super.key});

  @override
  Widget build(BuildContext context) {
    final cart = context.watch<CartProvider>();
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 12),
      child: Column(
        children: [
          Row(
            children: [
              Builder(
                builder: (context) => IconButton(
                  icon: const Icon(Icons.menu, color: AppColors.gray900),
                  onPressed: () => Scaffold.of(context).openDrawer(),
                ),
              ),
              const Expanded(child: Center(child: _HeaderLogo())),
              IconButton(
                icon: const Icon(Icons.shopping_cart_outlined, size: 22, color: AppColors.gray900),
                onPressed: () => context.go('/cart'),
              ),
              Stack(
                alignment: Alignment.topRight,
                children: [
                  IconButton(
                    icon: const Icon(Icons.person_outline, size: 22, color: AppColors.gray900),
                    onPressed: () => context.go('/account'),
                  ),
                  if (cart.itemCount > 0)
                    Container(
                      padding: const EdgeInsets.all(3),
                      decoration: const BoxDecoration(color: AppColors.navyBlue, shape: BoxShape.circle),
                      child: Text('${cart.itemCount}',
                          style: const TextStyle(color: Colors.white, fontSize: 9)),
                    ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 8),
          TextField(
            textInputAction: TextInputAction.search,
            onSubmitted: (query) => context.go('/search?q=$query'),
            decoration: InputDecoration(
              hintText: 'Search products here',
              prefixIcon: const Icon(Icons.search, size: 20, color: AppColors.zinc500),
              isDense: true,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                borderSide: const BorderSide(color: Color(0xFFE3E3E3)),
              ),
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                borderSide: const BorderSide(color: Color(0xFFE3E3E3)),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class MobileDrawer extends StatelessWidget {
  const MobileDrawer({super.key});

  @override
  Widget build(BuildContext context) {
    final catalog = context.watch<CatalogProvider>();
    final auth = context.watch<AuthProvider>();
    return Drawer(
      backgroundColor: Colors.white,
      child: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  const Expanded(child: _HeaderLogo()),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ],
              ),
            ),
            // Bloque cuenta
            Container(
              margin: const EdgeInsets.symmetric(horizontal: 12),
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                border: Border.all(color: AppColors.zinc200),
              ),
              child: Row(
                children: [
                  CircleAvatar(
                    radius: 26,
                    backgroundColor: AppColors.zinc100,
                    child: Icon(
                      auth.isLoggedIn ? Icons.person : Icons.person_outline,
                      color: AppColors.navyBlue,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: auth.isLoggedIn
                        ? Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text('Hello, ${auth.customer!.name}',
                                  style: const TextStyle(
                                      fontFamily: 'Poppins', fontWeight: FontWeight.w500, fontSize: 15)),
                              Text(auth.customer!.email,
                                  style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc500)),
                            ],
                          )
                        : GestureDetector(
                            onTap: () {
                              Navigator.of(context).pop();
                              context.go('/login');
                            },
                            child: Row(
                              children: const [
                                Text('Login',
                                    style: TextStyle(
                                        fontFamily: 'Poppins',
                                        fontWeight: FontWeight.w500,
                                        color: AppColors.navyBlue)),
                                Icon(Icons.double_arrow, size: 16, color: AppColors.navyBlue),
                              ],
                            ),
                          ),
                  ),
                ],
              ),
            ),
            const Divider(height: 24),
            // Árbol de categorías
            Expanded(
              child: ListView(
                padding: const EdgeInsets.symmetric(horizontal: 8),
                children: catalog.categories
                    .map((category) => ExpansionTile(
                          tilePadding: const EdgeInsets.symmetric(horizontal: 12),
                          childrenPadding: const EdgeInsets.only(left: 16),
                          title: Text(category.name,
                              style: const TextStyle(fontFamily: 'Poppins', fontSize: 14)),
                          onExpansionChanged: (_) {},
                          initiallyExpanded: false,
                          children: [
                            ListTile(
                              dense: true,
                              title: Text('View all ${category.name}',
                                  style: const TextStyle(
                                      fontFamily: 'Poppins', fontSize: 13, color: AppColors.navyBlue)),
                              onTap: () {
                                Navigator.of(context).pop();
                                context.go('/category/${category.id}');
                              },
                            ),
                            ...category.children.map(
                              (child) => ListTile(
                                dense: true,
                                title: Text(child.name,
                                    style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
                                onTap: () {
                                  Navigator.of(context).pop();
                                  context.go('/category/${child.id}');
                                },
                              ),
                            ),
                          ],
                        ))
                    .toList(),
              ),
            ),
            // Footer del drawer: moneda | locale
            Container(
              decoration: const BoxDecoration(
                border: Border(top: BorderSide(color: AppColors.zinc200)),
              ),
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              child: Row(
                children: [
                  _CurrencySelector(catalog: catalog, compact: true),
                  Container(width: 2, height: 18, color: AppColors.zinc200, margin: const EdgeInsets.symmetric(horizontal: 16)),
                  _LocaleSelector(catalog: catalog, compact: true),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Tira de servicios (4 iconos círculo)
// ---------------------------------------------------------------------------

class ServicesStrip extends StatelessWidget {
  const ServicesStrip({super.key});

  @override
  Widget build(BuildContext context) {
    final isNarrow = MediaQuery.of(context).size.width < 768;
    final services = const [
      (Icons.local_shipping_outlined, 'Free Delivery', 'On all orders over €50'),
      (Icons.workspace_premium_outlined, 'Quality Products', 'Curated, premium materials'),
      (Icons.replay_outlined, 'Money Back Guarantee', '30-day easy returns'),
      (Icons.headset_mic_outlined, 'Online Support 24/7', 'We are here to help'),
    ];
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 1440),
        child: Container(
          margin: const EdgeInsets.only(top: 88),
          padding: EdgeInsets.symmetric(horizontal: isNarrow ? 20 : 60, vertical: 40),
          decoration: const BoxDecoration(
            border: Border(
              top: BorderSide(color: AppColors.zinc100),
              bottom: BorderSide(color: AppColors.zinc100),
            ),
          ),
          child: isNarrow
              ? Wrap(
                  spacing: 24,
                  runSpacing: 28,
                  alignment: WrapAlignment.center,
                  children: services.map((s) => _ServiceItem(s.$1, s.$2, s.$3)).toList(),
                )
              : Row(
                  mainAxisAlignment: MainAxisAlignment.spaceAround,
                  children:
                      services.map((s) => Expanded(child: _ServiceItem(s.$1, s.$2, s.$3))).toList(),
                ),
        ),
      ),
    );
  }
}

class _ServiceItem extends StatelessWidget {
  const _ServiceItem(this.icon, this.title, this.subtitle);
  final IconData icon;
  final String title;
  final String subtitle;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 60,
          height: 60,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: Colors.white,
            border: Border.all(color: AppColors.gray900, width: 1),
          ),
          child: Icon(icon, size: 25, color: AppColors.navyBlue),
        ),
        const SizedBox(width: 18),
        Flexible(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: const TextStyle(
                  fontFamily: EzTheme.serif,
                  fontSize: 16,
                  color: AppColors.gray900,
                ),
              ),
              const SizedBox(height: 3),
              Text(
                subtitle,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(fontFamily: EzTheme.sans, fontSize: 12, color: AppColors.zinc500),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

class ShopFooter extends StatelessWidget {
  const ShopFooter({super.key});

  @override
  Widget build(BuildContext context) {
    final isNarrow = MediaQuery.of(context).size.width < 1060;
    return Container(
      color: AppColors.lightOrange,
      child: Column(
        children: [
          Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1440),
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: isNarrow ? 24 : 60, vertical: 56),
                child: isNarrow
                    ? Column(children: [_newsletter(), const SizedBox(height: 36), _links()])
                    : Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Expanded(flex: 3, child: _links()),
                          const SizedBox(width: 56),
                          Expanded(flex: 2, child: _newsletter()),
                        ],
                      ),
              ),
            ),
          ),
          Container(
            color: AppColors.footerBand,
            padding: const EdgeInsets.symmetric(vertical: 14),
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 1440),
                child: Padding(
                  padding: EdgeInsets.symmetric(horizontal: isNarrow ? 24 : 60),
                  child: Text(
                    '© Copyright ${DateTime.now().year}, EzCommerce. All rights reserved.',
                    style: TextStyle(
                        fontFamily: EzTheme.sans, fontSize: 12, color: AppColors.zinc600),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _links() {
    final columns = {
      'Shop': ['New Arrivals', 'Electronics', 'Men', 'Women', 'Home & Living'],
      'Customer Service': ['Contact Us', 'Shipping Policy', 'Return & Refunds', 'Track Order'],
      'About': ['Our Story', 'Sustainability', 'Careers', 'Press'],
      'Legal': ['Terms of Use', 'Privacy Policy', 'Cookie Policy'],
    };
    return Wrap(
      spacing: 44,
      runSpacing: 28,
      children: columns.entries
          .map((entry) => SizedBox(
                width: 150,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(entry.key,
                        style: const TextStyle(
                            fontFamily: EzTheme.sans,
                            fontSize: 13,
                            fontWeight: FontWeight.w600,
                            color: AppColors.navyBlue)),
                    const SizedBox(height: 16),
                    ...entry.value.map(
                      (link) => Padding(
                        padding: const EdgeInsets.only(bottom: 11),
                        child: MouseRegion(
                          cursor: SystemMouseCursors.click,
                          child: Text(link,
                              style: TextStyle(
                                  fontFamily: EzTheme.sans,
                                  fontSize: 13,
                                  color: AppColors.zinc600)),
                        ),
                      ),
                    ),
                  ],
                ),
              ))
          .toList(),
    );
  }

  Widget _newsletter() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Get Ready for our Fun Newsletter!',
          style: TextStyle(
            fontFamily: 'DM Serif Display',
            fontSize: 28,
            fontStyle: FontStyle.italic,
            color: AppColors.navyBlue,
          ),
        ),
        const SizedBox(height: 8),
        Text('Subscribe to stay in touch.',
            style: TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc600)),
        const SizedBox(height: 16),
        const _NewsletterForm(),
      ],
    );
  }
}

/// Formulario de suscripción al newsletter: marca al cliente como suscrito
/// para las campañas de email (#22).
class _NewsletterForm extends StatefulWidget {
  const _NewsletterForm();

  @override
  State<_NewsletterForm> createState() => _NewsletterFormState();
}

class _NewsletterFormState extends State<_NewsletterForm> {
  final TextEditingController _email = TextEditingController();
  bool _sending = false;

  Future<void> _subscribe() async {
    if (_sending) return;
    setState(() => _sending = true);
    try {
      final result = await ApiClient.instance.post('newsletter/subscribe', body: {
        'email': _email.text.trim(),
      });
      if (!mounted) return;
      final message = result is Map && result['message'] != null
          ? '${result['message']}'
          : 'You have been subscribed successfully.';
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(message)));
      _email.clear();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('$error'), backgroundColor: AppColors.red600));
      }
    } finally {
      if (mounted) setState(() => _sending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Stack(
          alignment: Alignment.centerRight,
          children: [
            TextField(
              controller: _email,
              keyboardType: TextInputType.emailAddress,
              decoration: InputDecoration(
                hintText: 'Email address',
                filled: true,
                fillColor: AppColors.footerBand,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                  borderSide: const BorderSide(color: AppColors.footerBorder, width: 2),
                ),
                enabledBorder: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(EzTheme.radiusXl),
                  borderSide: const BorderSide(color: AppColors.footerBorder, width: 2),
                ),
                contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 18),
              ),
            ),
            Padding(
              padding: const EdgeInsets.only(right: 8),
              child: TextButton(
                onPressed: _sending ? null : _subscribe,
                style: TextButton.styleFrom(
                  backgroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 12),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(EzTheme.radiusXl)),
                ),
                child: _sending
                    ? const SizedBox(
                        width: 14,
                        height: 14,
                        child: CircularProgressIndicator(strokeWidth: 2, color: AppColors.navyBlue))
                    : const Text('Subscribe',
                        style: TextStyle(fontFamily: 'Poppins', fontSize: 13, color: AppColors.gray900)),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Mini-carrito (drawer derecho 500px desde el header)
// ---------------------------------------------------------------------------

void showMiniCart(BuildContext context) {
  final cart = context.read<CartProvider>();
  showGeneralDialog(
    context: context,
    barrierDismissible: true,
    barrierLabel: 'Mini cart',
    barrierColor: Colors.black.withValues(alpha: 0.4),
    transitionDuration: const Duration(milliseconds: 250),
    pageBuilder: (dialogContext, _, _) {
      return Align(
        alignment: Alignment.centerRight,
        child: Material(
          color: Colors.white,
          child: SafeArea(
            child: Container(
              width: 460,
              height: double.infinity,
              padding: const EdgeInsets.all(24),
              child: StatefulBuilder(
                builder: (context, setState) {
                  final current = context.watch<CartProvider>().cart;
                  final items = current?.items ?? const [];
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          const Expanded(
                            child: Text('Shopping Cart',
                                style: TextStyle(fontFamily: 'Poppins',
                                    fontSize: 24, fontWeight: FontWeight.w500)),
                          ),
                          IconButton(
                            icon: const Icon(Icons.close),
                            onPressed: () => Navigator.of(dialogContext).pop(),
                          ),
                        ],
                      ),
                      const Divider(),
                      if (items.isEmpty)
                        const Expanded(
                          child: Center(child: Text('Your shopping cart is empty.')),
                        )
                      else
                        Expanded(
                          child: ListView.builder(
                            itemCount: items.length,
                            itemBuilder: (context, index) {
                              final item = items[index];
                              return Padding(
                                padding: const EdgeInsets.only(bottom: 16),
                                child: Row(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    ClipRRect(
                                      borderRadius: BorderRadius.circular(EzTheme.radiusLg),
                                      child: Container(
                                        width: 72,
                                        height: 72,
                                        color: AppColors.zinc100,
                                        child: item.imageUrl == null
                                            ? const Icon(Icons.image_outlined,
                                                color: AppColors.zinc300, size: 24)
                                            : Image.network(
                                                apiUrlRelative(item.imageUrl)!,
                                                fit: BoxFit.cover,
                                                errorBuilder: (_, _, _) => const Icon(
                                                    Icons.image_outlined,
                                                    color: AppColors.zinc300,
                                                    size: 24),
                                              ),
                                      ),
                                    ),
                                    const SizedBox(width: 12),
                                    Expanded(
                                      child: Column(
                                        crossAxisAlignment: CrossAxisAlignment.start,
                                        children: [
                                          Text(item.name,
                                              maxLines: 1,
                                              overflow: TextOverflow.ellipsis,
                                              style: const TextStyle(fontFamily: 'Poppins',
                                                  fontSize: 14, fontWeight: FontWeight.w500)),
                                          const SizedBox(height: 4),
                                          Row(
                                            children: [
                                              GestureDetector(
                                                onTap: () => cart.updateQuantity(
                                                    item.id ?? 0, item.quantity.toInt() - 1),
                                                child: const Icon(Icons.remove_circle_outline, size: 18),
                                              ),
                                              Padding(
                                                padding: const EdgeInsets.symmetric(horizontal: 10),
                                                child: Text('${item.quantity.toInt()}',
                                                    style: const TextStyle(
                                                        fontFamily: 'Poppins', fontSize: 14)),
                                              ),
                                              GestureDetector(
                                                onTap: () => cart.updateQuantity(
                                                    item.id ?? 0, item.quantity.toInt() + 1),
                                                child: const Icon(Icons.add_circle_outline, size: 18),
                                              ),
                                              const SizedBox(width: 12),
                                              GestureDetector(
                                                onTap: () => cart.removeItem(item.id ?? 0),
                                                child: const Icon(Icons.delete_outline,
                                                    size: 18, color: AppColors.zinc500),
                                              ),
                                            ],
                                          ),
                                        ],
                                      ),
                                    ),
                                    Text(item.formattedTotal,
                                        style: const TextStyle(fontFamily: 'Poppins',
                                            fontSize: 14, fontWeight: FontWeight.w600)),
                                  ],
                                ),
                              );
                            },
                          ),
                        ),
                      const Divider(),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          const Text('Sub Total',
                              style: TextStyle(fontFamily: 'Poppins',
                                  fontSize: 15, fontWeight: FontWeight.w600)),
                          Text(current?.formattedSubTotal ?? '',
                              style: const TextStyle(fontFamily: 'Poppins',
                                  fontSize: 15, fontWeight: FontWeight.w600)),
                        ],
                      ),
                      const SizedBox(height: 16),
                      Row(
                        children: [
                          Expanded(
                            child: SecondaryButton(
                              label: 'View Cart',
                              onPressed: () {
                                Navigator.of(dialogContext).pop();
                                context.go('/cart');
                              },
                            ),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: PrimaryButton(
                              label: 'Checkout',
                              onPressed: () {
                                Navigator.of(dialogContext).pop();
                                context.go('/checkout');
                              },
                            ),
                          ),
                        ],
                      ),
                    ],
                  );
                },
              ),
            ),
          ),
        ),
      );
    },
  );
}

/// Selector de moneda: solo las habilitadas en el canal (Settings).
class _CurrencySelector extends StatelessWidget {
  const _CurrencySelector({required this.catalog, this.compact = false});
  final CatalogProvider catalog;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    final currencies = catalog.currencies;
    final label = compact
        ? catalog.currencyCode
        : () {
            final match = currencies
                .where((currency) => currency['code'] == catalog.currencyCode)
                .firstOrNull;
            return match == null
                ? catalog.currencyCode
                : '${match['symbol'] ?? ''} ${match['code']}';
          }();
    if (currencies.length < 2) {
      return Text(label,
          style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc600));
    }
    return PopupMenuButton<String>(
      tooltip: 'Currency',
      onSelected: (code) => catalog.changeCurrency(code),
      itemBuilder: (context) => [
        for (final currency in currencies)
          PopupMenuItem(
            value: '${currency['code']}',
            child: Text('${currency['symbol'] ?? ''} ${currency['code']}',
                style: const TextStyle(fontFamily: EzTheme.sans, fontSize: 13)),
          ),
      ],
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 4),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Text(label,
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc600)),
          const Icon(Icons.expand_more, size: 14, color: AppColors.zinc600),
        ]),
      ),
    );
  }
}

/// Selector de idioma: solo los locales habilitados en el canal.
class _LocaleSelector extends StatelessWidget {
  const _LocaleSelector({required this.catalog, this.compact = false});
  final CatalogProvider catalog;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    final locales = catalog.locales;
    final label = compact
        ? () {
            final match = locales
                .where((locale) => locale['code'] == catalog.localeCode)
                .firstOrNull;
            return match == null ? catalog.localeCode : '${match['name']}';
          }()
        : () {
            final match = locales
                .where((locale) => locale['code'] == catalog.localeCode)
                .firstOrNull;
            return match == null ? catalog.localeCode : '${match['name']}';
          }();
    if (locales.length < 2) {
      return Text(label,
          style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc600));
    }
    return PopupMenuButton<String>(
      tooltip: 'Language',
      onSelected: (code) => catalog.changeLocale(code),
      itemBuilder: (context) => [
        for (final locale in locales)
          PopupMenuItem(
            value: '${locale['code']}',
            child: Text('${locale['name']}',
                style: const TextStyle(fontFamily: 'Poppins', fontSize: 13)),
          ),
      ],
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 4),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Text(label,
              style: const TextStyle(fontFamily: 'Poppins', fontSize: 12, color: AppColors.zinc600)),
          const Icon(Icons.expand_more, size: 14, color: AppColors.zinc600),
        ]),
      ),
    );
  }
}
