/// Modelos ligeros sobre los recursos JSON del API (Bagisto contract).
/// Se trabaja con `Map<String, dynamic>` directo + helpers de lectura segura.
library;

class Product {
  Product(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get name => (json['name'] ?? '') as String;
  String get sku => (json['sku'] ?? '') as String;
  String get type => (json['type'] ?? 'simple') as String;
  String get urlKey => (json['url_key'] ?? '') as String;
  num get price => (json['price'] ?? 0) as num;
  String get formattedPrice => (json['formatted_price'] ?? '') as String;
  String get formattedRegularPrice => (json['formatted_regular_price'] ?? '') as String;
  bool get onSale => (json['on_sale'] ?? false) as bool;
  bool get isNew => (json['is_new'] ?? false) as bool;
  bool get isSaleable => (json['is_saleable'] ?? true) as bool;
  String? get shortDescription => json['short_description'] as String?;
  String? get description => json['description'] as String?;
  double get ratingAverage => ((json['ratings'] ?? const {})['average'] ?? 0).toDouble();
  int get ratingsTotal => ((json['ratings'] ?? const {})['total'] ?? 0) as int;

  String? get imageUrl {
    final base = json['base_image'];
    if (base is Map && base['url'] != null) return base['url'] as String;
    final images = json['images'];
    if (images is List && images.isNotEmpty) {
      return (images.first as Map)['url'] as String?;
    }
    return null;
  }

  List<String> get imageUrls {
    final images = json['images'];
    if (images is! List) return [];
    return images.map((e) => (e as Map)['url'] as String? ?? '').where((u) => u.isNotEmpty).toList();
  }

  List<SuperAttribute> get superAttributes =>
      ((json['super_attributes'] ?? const []) as List)
          .map((e) => SuperAttribute(Map<String, dynamic>.from(e as Map)))
          .toList();

  List<Variant> get variants => ((json['variants'] ?? const []) as List)
      .map((e) => Variant(Map<String, dynamic>.from(e as Map)))
      .toList();

  List<Map<String, dynamic>> get additionalInformation =>
      ((json['additional_information'] ?? const []) as List)
          .map((e) => Map<String, dynamic>.from(e as Map))
          .toList();

  List<GroupedEntry> get groupedProducts => ((json['grouped_products'] ?? const []) as List)
      .map((e) => GroupedEntry(Map<String, dynamic>.from(e as Map)))
      .toList();
}

class SuperAttribute {
  SuperAttribute(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get code => (json['code'] ?? '') as String;
  String get label => (json['label'] ?? '') as String;
  String get swatchType => (json['swatch_type'] ?? '') as String;
  List<AttributeOption> get options => ((json['options'] ?? const []) as List)
      .map((e) => AttributeOption(Map<String, dynamic>.from(e as Map)))
      .toList();
}

class AttributeOption {
  AttributeOption(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get name => (json['admin_name'] ?? '') as String;
  String? get swatchValue => json['swatch_value'] as String?;
}

class Variant {
  Variant(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get sku => (json['sku'] ?? '') as String;
  num? get price => json['price'] as num?;
  Map<String, dynamic> get attributes =>
      Map<String, dynamic>.from((json['attributes'] ?? const {}) as Map);
}

class GroupedEntry {
  GroupedEntry(this.json);
  final Map<String, dynamic> json;

  num? get qty => json['qty'] as num?;
  Product get product => Product(Map<String, dynamic>.from((json['product'] ?? const {}) as Map));
}

class ShopCategory {
  ShopCategory(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get name => (json['name'] ?? '') as String;
  String get slug => (json['slug'] ?? '') as String;
  String? get description => json['description'] as String?;
  String? get bannerUrl => json['banner_url'] as String?;
  List<ShopCategory> get children => ((json['children'] ?? const []) as List)
      .map((e) => ShopCategory(Map<String, dynamic>.from(e as Map)))
      .toList();
}

class CartItem {
  CartItem(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get name => (json['name'] ?? '') as String;
  String get sku => (json['sku'] ?? '') as String;
  num get quantity => (json['quantity'] ?? 0) as num;
  String get formattedPrice => (json['formatted_price'] ?? '') as String;
  String get formattedTotal => (json['formatted_total'] ?? '') as String;
  int? get productId => json['product_id'] as int?;
  Map<String, dynamic>? get additional =>
      json['additional'] is Map ? Map<String, dynamic>.from(json['additional'] as Map) : null;
}

class Cart {
  Cart(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  bool get isGuest => (json['is_guest'] ?? false) as bool;
  num get itemsQty => (json['items_qty'] ?? 0) as num;
  String get formattedSubTotal => (json['formatted_sub_total'] ?? '') as String;
  String get formattedTaxTotal => (json['formatted_tax_total'] ?? '') as String;
  String get formattedDiscount => (json['formatted_discount_amount'] ?? '') as String;
  String get formattedShipping => (json['formatted_shipping_amount'] ?? '') as String;
  String get formattedGrandTotal => (json['formatted_grand_total'] ?? '') as String;
  String? get couponCode => json['coupon_code'] as String?;
  String? get shippingMethod => json['shipping_method'] as String?;
  String? get paymentMethod => json['payment_method'] as String?;
  String? get paymentMethodTitle => json['payment_method_title'] as String?;
  List<CartItem> get items => ((json['items'] ?? const []) as List)
      .map((e) => CartItem(Map<String, dynamic>.from(e as Map)))
      .toList();
}

class ShippingRate {
  ShippingRate(this.json);
  final Map<String, dynamic> json;

  String get carrier => (json['carrier'] ?? '') as String;
  String get carrierTitle => (json['carrier_title'] ?? '') as String;
  String get method => (json['method'] ?? '') as String;
  String get methodTitle => (json['method_title'] ?? '') as String;
  String get methodDescription => (json['method_description'] ?? '') as String;
  num get price => (json['price'] ?? 0) as num;
}

class PaymentMethod {
  PaymentMethod(this.json);
  final Map<String, dynamic> json;

  String get method => (json['method'] ?? '') as String;
  String get methodTitle => (json['method_title'] ?? '') as String;
  String get description => (json['description'] ?? '') as String;
}

class Order {
  Order(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get incrementId => (json['increment_id'] ?? '') as String;
  String get status => (json['status'] ?? '') as String;
  String get createdAt => (json['created_at'] ?? '') as String;
  String get formattedTotal => (json['grand_total'] ?? '').toString();
  int? get totalItemCount => json['total_item_count'] as int?;

  bool get canCancel => (json['can_cancel'] ?? false) as bool;
  bool get canReorder => (json['can_reorder'] ?? false) as bool;
  bool get canInvoice => (json['can_invoice'] ?? false) as bool;
  bool get canShip => (json['can_ship'] ?? false) as bool;
  bool get canRefund => (json['can_refund'] ?? false) as bool;

  List<OrderItem> get items => ((json['items'] ?? const []) as List)
      .map((e) => OrderItem(Map<String, dynamic>.from(e as Map)))
      .toList();

  Map<String, dynamic>? get billingAddress =>
      json['billing_address'] is Map ? Map<String, dynamic>.from(json['billing_address'] as Map) : null;
  Map<String, dynamic>? get payment =>
      json['payment'] is Map ? Map<String, dynamic>.from(json['payment'] as Map) : null;
  List<Map<String, dynamic>> get comments => ((json['comments'] ?? const []) as List)
      .map((e) => Map<String, dynamic>.from(e as Map))
      .toList();
}

class OrderItem {
  OrderItem(this.json);
  final Map<String, dynamic> json;

  String get name => (json['name'] ?? '') as String;
  String get sku => (json['sku'] ?? '') as String;
  num get qtyOrdered => (json['qty_ordered'] ?? 0) as num;
  num get price => (json['price'] ?? 0) as num;
  num get total => (json['total'] ?? 0) as num;
}

class Address {
  Address(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get firstName => (json['first_name'] ?? '') as String;
  String get lastName => (json['last_name'] ?? '') as String;
  String get address1 => (json['address1'] ?? json['address'] ?? '') as String;
  String get city => (json['city'] ?? '') as String;
  String get state => (json['state'] ?? '') as String;
  String get country => (json['country'] ?? '') as String;
  String get postcode => (json['postcode'] ?? '') as String;
  String get phone => (json['phone'] ?? '') as String;
  bool get isDefault => (json['default_address'] ?? false) as bool;

  Map<String, dynamic> toPayload() => Map<String, dynamic>.from(json);
}

class Customer {
  Customer(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get firstName => (json['first_name'] ?? '') as String;
  String get lastName => (json['last_name'] ?? '') as String;
  String get name => (json['name'] ?? '$firstName $lastName') as String;
  String get email => (json['email'] ?? '') as String;
  String? get image => json['image'] as String?;
}

/// Sección del home (`theme_sections`): type + options traducidas.
class HomeSection {
  HomeSection(this.json);
  final Map<String, dynamic> json;

  int? get id => json['id'] as int?;
  String get type => (json['type'] ?? '') as String;
  String get name => (json['name'] ?? '') as String;
  Map<String, dynamic> get options =>
      Map<String, dynamic>.from((json['options'] ?? const {}) as Map);
}
