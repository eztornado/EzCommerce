import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import 'core/api/api_client.dart';
import 'core/state/app_state.dart';
import 'core/theme/ez_theme.dart';
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
        GoRoute(path: '/', builder: (_, __) => const HomePage()),
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
        GoRoute(path: '/cart', builder: (_, __) => const CartPage()),
        GoRoute(path: '/account', builder: (_, __) => const AccountPage()),
        GoRoute(path: '/account/orders', builder: (_, __) => const OrdersPage()),
        GoRoute(
          path: '/account/orders/:id',
          builder: (_, state) => OrderDetailPage(orderId: state.pathParameters['id']!),
        ),
        GoRoute(path: '/account/addresses', builder: (_, __) => const AddressesPage()),
        GoRoute(path: '/account/wishlist', builder: (_, __) => const WishlistPage()),
        GoRoute(path: '/account/profile', builder: (_, __) => const ProfilePage()),
      ],
    ),
    GoRoute(
      path: '/checkout',
      builder: (_, __) => const CheckoutPage(),
    ),
    GoRoute(
      path: '/checkout/success/:incrementId',
      builder: (_, state) => CheckoutSuccessPage(incrementId: state.pathParameters['incrementId'] ?? ''),
    ),
    GoRoute(path: '/login', builder: (_, __) => const SignInPage()),
    GoRoute(path: '/register', builder: (_, __) => const SignUpPage()),
    GoRoute(path: '/forgot-password', builder: (_, __) => const ForgotPasswordPage()),
    GoRoute(
      path: '/admin/login',
      builder: (_, __) => const AdminLoginPage(),
    ),
    ShellRoute(
      builder: (context, state, child) => AdminShell(child: child),
      routes: [
        GoRoute(path: '/admin', builder: (_, __) => const AdminDashboardPage()),
        GoRoute(path: '/admin/products', builder: (_, __) => const AdminProductsPage()),
        GoRoute(path: '/admin/orders', builder: (_, __) => const AdminOrdersPage()),
        GoRoute(path: '/admin/customers', builder: (_, __) => const AdminCustomersPage()),
        GoRoute(path: '/admin/settings', builder: (_, __) => const AdminSettingsPage()),
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
