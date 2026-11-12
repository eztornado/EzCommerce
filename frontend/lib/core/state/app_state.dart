import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;

import '../api/api_client.dart';
import '../models/models.dart';

/// Estado global de autenticación de cliente (JWT).
class AuthProvider extends ChangeNotifier {
  bool _loading = false;
  Customer? _customer;
  String? _error;

  bool get loading => _loading;
  Customer? get customer => _customer;
  bool get isLoggedIn => _customer != null;
  String? get error => _error;

  Future<void> restore() async {
    if (!ApiClient.instance.isCustomerLoggedIn) return;
    try {
      final data = await ApiClient.instance.get('customer/get');
      _customer = Customer(Map<String, dynamic>.from(data as Map));
    } on ApiException catch (error) {
      if (error.isUnauthorized) await ApiClient.instance.clearCustomerSession();
    } catch (_) {}
    notifyListeners();
  }

  Future<bool> login(String email, String password) async {
    _loading = true;
    _error = null;
    notifyListeners();
    try {
      final data = await ApiClient.instance.post('customer/login', body: {
        'email': email,
        'password': password,
      });
      final map = Map<String, dynamic>.from(data as Map);
      await ApiClient.instance.setCustomerSession(map['token'] as String);
      _customer = Customer(Map<String, dynamic>.from((map['user'] ?? const {}) as Map));
      return true;
    } on ApiException catch (error) {
      _error = error.message;
      return false;
    } finally {
      _loading = false;
      notifyListeners();
    }
  }

  Future<bool> register({
    required String firstName,
    required String lastName,
    required String email,
    required String password,
    required String passwordConfirmation,
  }) async {
    _loading = true;
    _error = null;
    notifyListeners();
    try {
      final data = await ApiClient.instance.post('customer/register', body: {
        'first_name': firstName,
        'last_name': lastName,
        'email': email,
        'password': password,
        'password_confirmation': passwordConfirmation,
      });
      final map = Map<String, dynamic>.from(data as Map);
      if (map['token'] != null) {
        await ApiClient.instance.setCustomerSession(map['token'] as String);
        final user = map['user'];
        _customer = user is Map
            ? Customer(Map<String, dynamic>.from(user))
            : await _fetchCustomer();
      }
      return true;
    } on ApiException catch (error) {
      _error = error.message;
      return false;
    } finally {
      _loading = false;
      notifyListeners();
    }
  }

  Future<Customer?> _fetchCustomer() async {
    final data = await ApiClient.instance.get('customer/get');
    return Customer(Map<String, dynamic>.from(data as Map));
  }

  Future<void> logout() async {
    try {
      await ApiClient.instance.post('customer/logout');
    } catch (_) {}
    await ApiClient.instance.clearCustomerSession();
    _customer = null;
    notifyListeners();
  }
}

/// Carrito: invitado (X-Cart-Token) o registrado (JWT).
class CartProvider extends ChangeNotifier {
  Cart? _cart;
  bool _loading = false;
  int get itemCount => _cart?.itemsQty.toInt() ?? 0;
  Cart? get cart => _cart;
  bool get loading => _loading;

  Future<void> refresh() async {
    try {
      final data = await ApiClient.instance.get('checkout/cart');
      _cart = data is Map ? Cart(Map<String, dynamic>.from(data)) : null;
    } catch (_) {}
    notifyListeners();
  }

  Future<String?> addProduct(int productId, {int quantity = 1, Map<String, dynamic>? options}) async {
    _loading = true;
    notifyListeners();
    try {
      final body = {'product_id': productId, 'quantity': quantity, ...?options};
      final data = await ApiClient.instance.post('checkout/cart', body: body);
      final map = Map<String, dynamic>.from(data as Map);
      final token = map['cart_token'];
      if (token is String) await ApiClient.instance.setCartToken(token);
      await refresh();
      return null;
    } on ApiException catch (error) {
      return error.message;
    } finally {
      _loading = false;
      notifyListeners();
    }
  }

  Future<void> updateQuantity(int itemId, int quantity) async {
    try {
      await ApiClient.instance.put('checkout/cart', body: {
        'qty': [
          {'cart_item_id': itemId, 'quantity': quantity},
        ],
      });
      await refresh();
    } catch (_) {}
  }

  Future<void> removeItem(int itemId) async {
    try {
      await ApiClient.instance.delete('checkout/cart/selected', body: {'item_ids': [itemId]});
      await refresh();
    } catch (_) {}
  }

  Future<String?> applyCoupon(String code) async {
    try {
      await ApiClient.instance.post('checkout/cart/coupon', body: {'code': code});
      await refresh();
      return null;
    } on ApiException catch (error) {
      return error.message;
    }
  }
}

/// Catálogo: árbol de categorías, listado con filtros, contexto de tienda.
class CatalogProvider extends ChangeNotifier {
  List<ShopCategory> _categories = [];
  List<ShopCategory> get categories => _categories;

  List<Product> _products = [];
  int _total = 0;
  bool _loading = false;
  int _currentPage = 1;
  int get total => _total;
  bool get loading => _loading;
  bool get hasMore => _products.length < _total;
  List<Product> get products => _products;

  Map<String, dynamic>? context;

  Future<void> loadContext() async {
    try {
      final data = await ApiClient.instance.get('core/context');
      context = Map<String, dynamic>.from(data as Map);
    } catch (_) {}
    // El árbol de categorías alimenta el header y el home; debe cargarse al
    // arrancar, no solo al cambiar de idioma/moneda.
    await loadCategories();
    notifyListeners();
  }

  /// Locales y monedas habilitados en el canal (para los selectores).
  List<Map<String, dynamic>> get locales {
    final value = context?['locales'];
    if (value is List) {
      return value.map((e) => Map<String, dynamic>.from(e as Map)).toList();
    }
    return const [];
  }

  List<Map<String, dynamic>> get currencies {
    final value = context?['currencies'];
    if (value is List) {
      return value.map((e) => Map<String, dynamic>.from(e as Map)).toList();
    }
    return const [];
  }

  String get localeCode => '${context?['locale'] ?? 'en'}';
  String get currencyCode => '${context?['currency'] ?? ''}';

  /// Tasa de cambio de la moneda activa respecto a la base (presentación en
  /// carrito/checkout; el backend convierte catálogo).
  double get exchangeRate {
    final rate = context?['exchange_rate'];
    return rate is num && rate > 0 ? rate.toDouble() : 1.0;
  }

  /// Cambia el idioma de la tienda y recarga el contenido.
  Future<void> changeLocale(String code) async {
    await ApiClient.setShopLocale(code);
    await loadContext();
    await loadCategories();
    await loadHomeSections();
    await refreshProducts();
  }

  /// Cambia la moneda de la tienda y recarga el contenido.
  Future<void> changeCurrency(String code) async {
    await ApiClient.setShopCurrency(code);
    await loadContext();
    await loadCategories();
    await loadHomeSections();
    await refreshProducts();
  }

  /// Recarga el listado con los últimos filtros (cambio de idioma/moneda).
  Future<void> refreshProducts() async {
    _products = [];
    _currentPage = 1;
    await loadProducts(
      categoryId: _lastCategoryId,
      query: _lastQuery,
      page: 1,
      filters: _lastFilters,
    );
  }

  Future<void> loadCategories() async {
    try {
      final data = await ApiClient.instance.get('categories/tree');
      _categories = ((data as List?) ?? const [])
          .map((e) => ShopCategory(Map<String, dynamic>.from(e as Map)))
          .toList();
    } catch (_) {}
    notifyListeners();
  }

  Future<void> loadProducts({
    int? categoryId,
    String? query,
    String sort = 'newest',
    int page = 1,
    Map<String, String> filters = const {},
  }) async {
    _loading = true;
    notifyListeners();
    try {
      final queryParams = <String, String>{
        'page': '$page',
        'per_page': '12',
        'sort': sort,
        ...filters,
      };
      if (categoryId != null) queryParams['category_id'] = '$categoryId';
      if (query != null && query.isNotEmpty) queryParams['query'] = query;
      final data = await ApiClient.instance.get('products', query: queryParams);
      final map = Map<String, dynamic>.from(data as Map);
      final list = Map<String, dynamic>.from(map);
      final meta = (list['meta'] ?? const {}) as Map;
      _total = (meta['total'] ?? 0) as int;
      _currentPage = page;
      final records = ((list['data'] ?? const []) as List)
          .map((e) => Product(Map<String, dynamic>.from(e as Map)))
          .toList();
      if (page == 1) {
        _lastFilters = filters;
        _lastQuery = query;
        _lastCategoryId = categoryId;
      }
      _products = page == 1 ? records : [..._products, ...records];
    } catch (_) {}
    _loading = false;
    notifyListeners();
  }

  Map<String, String> _lastFilters = const {};
  String? _lastQuery;
  int? _lastCategoryId;

  Future<void> loadMore({
    int? categoryId,
    String? query,
    String sort = 'newest',
    Map<String, String> filters = const {},
  }) async {
    if (!hasMore || _loading) return;
    await loadProducts(
      categoryId: categoryId,
      query: query,
      sort: sort,
      page: _currentPage + 1,
      filters: filters.isNotEmpty ? filters : _lastFilters,
    );
  }

  Future<List<Product>> loadHomeProducts() async {
    try {
      final data = await ApiClient.instance.get('products', query: {
        'per_page': '8',
        'sort': 'newest',
      });
      final map = Map<String, dynamic>.from(data as Map);
      return ((map['data'] ?? const []) as List)
          .map((e) => Product(Map<String, dynamic>.from(e as Map)))
          .toList();
    } catch (_) {
      return [];
    }
  }

  Future<List<HomeSection>> loadHomeSections() async {
    try {
      final data = await ApiClient.instance.get('core/home');
      return ((data as List?) ?? const [])
          .map((e) => HomeSection(Map<String, dynamic>.from(e as Map)))
          .toList();
    } catch (_) {
      return [];
    }
  }
}

/// Sesión de administrador (JWT admin).
class AdminProvider extends ChangeNotifier {
  bool _loading = false;
  String? _error;
  Map<String, dynamic>? _me;
  bool get loading => _loading;
  bool get isLoggedIn => ApiClient.instance.isAdminLoggedIn;
  String? get error => _error;
  Map<String, dynamic>? get me => _me;

  Future<bool> login(String email, String password) async {
    _loading = true;
    _error = null;
    notifyListeners();
    try {
      final data = await ApiClient.instance.post('admin/login', body: {
        'email': email,
        'password': password,
      }, admin: true);
      final map = Map<String, dynamic>.from(data as Map);
      await ApiClient.instance.setAdminSession(map['token'] as String);
      await loadMe();
      return true;
    } on ApiException catch (error) {
      _error = error.message;
      return false;
    } finally {
      _loading = false;
      notifyListeners();
    }
  }

  Future<void> loadMe() async {
    try {
      final data = await ApiClient.instance.get('admin/me', admin: true);
      _me = Map<String, dynamic>.from(data as Map);
    } catch (_) {}
    notifyListeners();
  }

  Future<void> logout() async {
    try {
      await ApiClient.instance.post('admin/logout', admin: true);
    } catch (_) {}
    await ApiClient.instance.clearAdminSession();
    _me = null;
    notifyListeners();
  }

  /// Ejecuta un datagrid admin y devuelve el contrato Bagisto.
  Future<Map<String, dynamic>?> datagrid(String path, {Map<String, dynamic> body = const {}}) async {
    try {
      final data = await ApiClient.instance.post(path, body: body, admin: true);
      return Map<String, dynamic>.from(data as Map);
    } on ApiException {
      return null;
    }
  }

  Future<dynamic> get(String path, {Map<String, String>? query}) async {
    try {
      return await ApiClient.instance.get(path, query: query, admin: true);
    } catch (_) {
      return null;
    }
  }

  Future<dynamic> post(String path, {Object? body}) async {
    try {
      return await ApiClient.instance.post(path, body: body, admin: true);
    } catch (_) {
      return null;
    }
  }

  Future<dynamic> put(String path, {Object? body}) async {
    try {
      return await ApiClient.instance.put(path, body: body, admin: true);
    } catch (_) {
      return null;
    }
  }

  Future<dynamic> delete(String path) async {
    try {
      return await ApiClient.instance.delete(path, admin: true);
    } catch (_) {
      return null;
    }
  }
}

/// Decodifica respuestas paginadas `{data, meta}`.
List<Map<String, dynamic>> decodeList(dynamic data) {
  if (data is List) {
    return data.map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }
  if (data is Map && data['data'] is List) {
    return (data['data'] as List).map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }
  return [];
}

/// Decodifica payloads de ids planos (`[1, 2]`) tolerando `null`.
List<int> decodeIds(dynamic data) {
  if (data is List) return data.whereType<int>().toList();
  if (data is Map && data['data'] is List) {
    return (data['data'] as List).whereType<int>().toList();
  }
  return [];
}

String? apiUrlRelative(String? path) =>
    path == null || path.isEmpty ? null : '${ApiClient.baseUrl}$path';

Future<dynamic> jsonRequest(Future<http.Response> Function() request) async {
  final response = await request();
  return jsonDecode(response.body);
}
