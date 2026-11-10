import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

/// Cliente HTTP del API EzCommerce (Larust). Gestiona:
/// - JWT del cliente (`Authorization: Bearer`) y del admin
/// - token de carrito de invitado (`X-Cart-Token`)
/// - envelope `{data, message}` y errores `{message}`
class ApiClient {
  ApiClient._();
  static final ApiClient instance = ApiClient._();

  /// URL del API. En web se sirve normalmente en otro origen; móvil usa 10.0.2.2 para emulador.
  static const String productionUrl = 'http://127.0.0.1:34187';
  static String baseUrl = productionUrl;

  String? _customerToken;
  String? _adminToken;
  String? _cartToken;

  bool get isCustomerLoggedIn => _customerToken != null;
  bool get isAdminLoggedIn => _adminToken != null;
  String? get adminToken => _adminToken;

  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    _customerToken = prefs.getString('customer_token');
    _adminToken = prefs.getString('admin_token');
    _cartToken = prefs.getString('cart_token');
  }

  Future<void> setCustomerSession(String token) async {
    _customerToken = token;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('customer_token', token);
  }

  Future<void> setAdminSession(String token) async {
    _adminToken = token;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('admin_token', token);
  }

  Future<void> setCartToken(String token) async {
    _cartToken = token;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('cart_token', token);
  }

  Future<void> clearCustomerSession() async {
    _customerToken = null;
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('customer_token');
  }

  Future<void> clearAdminSession() async {
    _adminToken = null;
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('admin_token');
  }

  Map<String, String> _headers({bool admin = false}) {
    final headers = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };
    final token = admin ? _adminToken : _customerToken;
    if (token != null) headers['Authorization'] = 'Bearer $token';
    if (!admin && _cartToken != null) headers['X-Cart-Token'] = _cartToken!;
    return headers;
  }

  Uri _uri(String path, Map<String, String>? query) {
    final normalized = path.startsWith('/') ? path.substring(1) : path;
    final uri = Uri.parse('$baseUrl/api/$normalized');
    return query == null || query.isEmpty ? uri : uri.replace(queryParameters: query);
  }

  /// GET. Devuelve el `data` del envelope o lanza `ApiException`.
  Future<dynamic> get(String path, {Map<String, String>? query, bool admin = false}) async {
    final response = await http.get(_uri(path, query), headers: _headers(admin: admin));
    return _handle(response);
  }

  /// POST.
  Future<dynamic> post(String path, {Object? body, bool admin = false}) async {
    final response = await http.post(
      _uri(path, null),
      headers: _headers(admin: admin),
      body: jsonEncode(body ?? {}),
    );
    return _handle(response);
  }

  /// PUT.
  Future<dynamic> put(String path, {Object? body, bool admin = false}) async {
    final response = await http.put(
      _uri(path, null),
      headers: _headers(admin: admin),
      body: jsonEncode(body ?? {}),
    );
    return _handle(response);
  }

  /// PATCH.
  Future<dynamic> patch(String path, {Object? body, bool admin = false}) async {
    final response = await http.patch(
      _uri(path, null),
      headers: _headers(admin: admin),
      body: jsonEncode(body ?? {}),
    );
    return _handle(response);
  }

  /// DELETE.
  Future<dynamic> delete(String path,
      {Map<String, String>? query, Object? body, bool admin = false}) async {
    final request = http.Request('DELETE', _uri(path, query));
    request.headers.addAll(_headers(admin: admin));
    if (body != null) request.body = jsonEncode(body);
    final response = await http.Response.fromStream(await request.send());
    return _handle(response);
  }

  dynamic _handle(http.Response response) {
    dynamic payload;
    try {
      payload = response.body.isEmpty ? null : jsonDecode(response.body);
    } catch (_) {
      payload = null;
    }

    if (response.statusCode >= 200 && response.statusCode < 300) {
      // Envelope {data, message} del API.
      if (payload is Map && payload.containsKey('data')) {
        return payload['data'];
      }
      return payload;
    }

    String message = 'Error ${response.statusCode}';
    if (payload is Map) {
      final errors = payload['errors'];
      if (errors is Map && errors.isNotEmpty) {
        final first = errors.values.first;
        message = first is List && first.isNotEmpty ? first.first.toString() : errors.toString();
      } else if (payload['message'] is String && (payload['message'] as String).isNotEmpty) {
        message = payload['message'] as String;
      }
    }
    throw ApiException(message, response.statusCode);
  }
}

class ApiException implements Exception {
  ApiException(this.message, this.statusCode);
  final String message;
  final int statusCode;

  bool get isUnauthorized => statusCode == 401;

  @override
  String toString() => message;
}

/// Helpers top-level para uso directo desde widgets.
Future<dynamic> apiGet(String path, {Map<String, String>? query}) =>
    ApiClient.instance.get(path, query: query);
Future<dynamic> apiPost(String path, {Object? body}) =>
    ApiClient.instance.post(path, body: body);
Future<dynamic> apiPut(String path, {Object? body}) =>
    ApiClient.instance.put(path, body: body);
Future<dynamic> apiDelete(String path) => ApiClient.instance.delete(path);
