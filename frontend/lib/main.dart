import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import 'core/api/api_client.dart';
import 'core/state/app_state.dart';
import 'core/theme/ez_theme.dart';
import 'features/admin/admin_product_editor.dart';
import 'features/admin/admin_product_create.dart';
import 'features/admin/admin_cart_rules.dart';
import 'features/admin/admin_cms.dart';
import 'features/admin/admin_reporting.dart';
import 'features/admin/admin_sales_pages.dart';
import 'features/admin/admin_catalog_rules.dart';
import 'features/admin/admin_categories.dart';
import 'features/admin/admin_attributes.dart';
import 'features/admin/admin_appearance.dart';
import 'features/admin/admin_communications.dart';
import 'features/admin/admin_shell.dart';
import 'features/shop/account/account_pages.dart';
import 'features/shop/auth/auth_pages.dart';
import 'features/shop/cart/cart_page.dart';
import 'features/shop/catalog/category_page.dart';
import 'features/shop/checkout/checkout_pages.dart';
import 'features/shop/home/home_page.dart';
import 'features/shop/product/product_page.dart';
import 'features/shop/shell/shop_shell.dart';

final _rootNavigatorKey = GlobalKey<NavigatorState>();

final GoRouter router = GoRouter(
  navigatorKey: _rootNavigatorKey,
  initialLocation: '/',
  routes: [
    ShellRoute(
      builder: (context, state, child) => ShopShell(child: child),
      routes: [
        GoRoute(path: '/', builder: (_, _) => const HomePage()),
        GoRoute(
          path: '/category/:id',
          builder: (_, state) => CategoryPage(
            categoryId: int.tryParse(state.pathParameters['id'] ?? ''),
          ),
        ),
        GoRoute(
          path: '/product/:id',
          builder: (_, state) => ProductPage(productId: state.pathParameters['id']!),
        ),
        GoRoute(path: '/search', builder: (_, state) => SearchPage(query: state.uri.queryParameters['q'] ?? '')),
        GoRoute(path: '/cart', builder: (_, _) => const CartPage()),
        GoRoute(path: '/account', builder: (_, _) => const AccountPage()),
        GoRoute(path: '/account/orders', builder: (_, _) => const OrdersPage()),
        GoRoute(
          path: '/account/orders/:id',
          builder: (_, state) => OrderDetailPage(orderId: state.pathParameters['id']!),
        ),
        GoRoute(path: '/account/addresses', builder: (_, _) => const AddressesPage()),
        GoRoute(path: '/account/wishlist', builder: (_, _) => const WishlistPage()),
        GoRoute(path: '/account/profile', builder: (_, _) => const ProfilePage()),
      ],
    ),
    GoRoute(
      path: '/checkout',
      builder: (_, _) => const CheckoutPage(),
    ),
    GoRoute(
      path: '/checkout/success/:incrementId',
      builder: (_, state) => CheckoutSuccessPage(incrementId: state.pathParameters['incrementId'] ?? ''),
    ),
    GoRoute(path: '/login', builder: (_, _) => const SignInPage()),
    GoRoute(path: '/register', builder: (_, _) => const SignUpPage()),
    GoRoute(path: '/forgot-password', builder: (_, _) => const ForgotPasswordPage()),
    GoRoute(
      path: '/admin/login',
      builder: (_, _) => const AdminLoginPage(),
    ),
    ShellRoute(
      builder: (context, state, child) => AdminShell(child: child),
      routes: [
        GoRoute(path: '/admin', builder: (_, _) => const AdminDashboardPage()),
        GoRoute(path: '/admin/products', builder: (_, _) => const AdminProductsPage()),
        GoRoute(path: '/admin/products/create', builder: (_, _) => const AdminProductCreatePage()),
        GoRoute(path: '/admin/categories', builder: (_, _) => const AdminCategoriesPage()),
        GoRoute(path: '/admin/attributes', builder: (_, _) => const AdminAttributesPage()),
        GoRoute(path: '/admin/cart-rules', builder: (_, _) => const AdminCartRulesPage()),
        GoRoute(path: '/admin/catalog-rules', builder: (_, _) => const AdminCatalogRulesPage()),
        GoRoute(path: '/admin/cms', builder: (_, _) => const AdminCmsPage()),
        GoRoute(path: '/admin/appearance', builder: (_, _) => const AdminAppearancePage()),
        GoRoute(
          path: '/admin/products/edit/:id',
          builder: (_, state) =>
              AdminProductEditorPage(productId: state.pathParameters['id']!),
        ),
        GoRoute(path: '/admin/orders', builder: (_, _) => const AdminOrdersPage()),
        GoRoute(path: '/admin/invoices', builder: (_, _) => const AdminInvoicesPage()),
        GoRoute(path: '/admin/shipments', builder: (_, _) => const AdminShipmentsPage()),
        GoRoute(path: '/admin/refunds', builder: (_, _) => const AdminRefundsPage()),
        GoRoute(path: '/admin/reporting', builder: (_, _) => const AdminReportingPage()),
        GoRoute(path: '/admin/customers', builder: (_, _) => const AdminCustomersPage()),
        GoRoute(path: '/admin/communications', builder: (_, _) => const AdminCommunicationsPage()),
        GoRoute(path: '/admin/settings', builder: (_, _) => const AdminSettingsPage()),
      ],
    ),
  ],
);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await ApiClient.instance.load();

  runApp(MultiProvider(
    providers: [
      ChangeNotifierProvider(create: (_) => AuthProvider()..restore()),
      ChangeNotifierProvider(create: (_) => CartProvider()..refresh()),
      ChangeNotifierProvider(create: (_) => CatalogProvider()..loadContext()),
      ChangeNotifierProvider(create: (_) => AdminProvider()),
    ],
    child: const EzApp(),
  ));
}

class EzApp extends StatelessWidget {
  const EzApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'EzCommerce',
      debugShowCheckedModeBanner: false,
      theme: EzTheme.light(),
      routerConfig: router,
    );
  }
}
