//! All API routes, mounted under `API_PREFIX` (`/api` by default).
//! Shop endpoints keep the Bagisto paths; admin endpoints live under
//! `/api/admin` (the Blade admin is replaced by the Flutter panel).

use larust_http::{Route, Router};

use crate::controllers::{
    AdminAuthController, AdminCartRuleController, AdminCatalogController, AdminCustomerController, AdminDashboardController,
    AdminSalesController, AdminSettingController, AuthController, CartController,
    CategoryController, CheckoutController, CoreController, CustomerController, ProductController,
};

pub fn routes() -> Router {
    // ------------------------------------------------------------------
    // Shop core
    // ------------------------------------------------------------------
    Route::get("/storage/{folder}/{file}", CoreController::storage)
        .post("/admin/upload", CoreController::upload)
        .get("/core/context", CoreController::context)
        .get("/core/home", CoreController::home)
        .get("/core/countries", CoreController::countries)
        .get("/core/states", CoreController::states)
        .get("/core/currencies", CoreController::currencies)
        // ------------------------------------------------------------------
        // Categories
        // ------------------------------------------------------------------
        .get("/categories", CategoryController::index)
        .get("/categories/tree", CategoryController::tree)
        .get("/categories/attributes", CategoryController::attributes)
        .get("/categories/attributes/{id}/options", CategoryController::attribute_options)
        .get("/categories/max-price/{id}", CategoryController::max_price)
        .get("/categories/{id}", CategoryController::show)
        // ------------------------------------------------------------------
        // Products
        // ------------------------------------------------------------------
        .get("/products", ProductController::index)
        .get("/products/{id}/related", ProductController::related)
        .get("/products/{id}/up-sell", ProductController::up_sell)
        .get("/products/{id}", ProductController::show)
        .get("/product/{id}/reviews", ProductController::reviews)
        .post("/product/{id}/review", ProductController::store_review)
        // ------------------------------------------------------------------
        // Customer auth + account
        // ------------------------------------------------------------------
        .post("/customer/login", AuthController::login)
        .post("/customer/register", AuthController::register)
        .post("/customer/logout", AuthController::logout)
        .get("/customer/get", AuthController::get)
        .get("/customer/refresh", AuthController::refresh)
        .put("/customer/profile", AuthController::update_profile)
        .post("/customer/forgot-password", AuthController::forgot_password)
        .post("/customer/reset-password", AuthController::reset_password)
        .get("/customer/addresses", CustomerController::addresses)
        .post("/customer/addresses", CustomerController::store_address)
        .put("/customer/addresses/edit/{id}", CustomerController::update_address)
        .patch("/customer/addresses/default/{id}", CustomerController::make_default)
        .delete("/customer/addresses/{id}", CustomerController::delete_address)
        .get("/customer/wishlist", CustomerController::wishlist)
        .post("/customer/wishlist", CustomerController::add_to_wishlist)
        .post("/customer/wishlist/{id}/move-to-cart", CustomerController::move_to_cart)
        .delete("/customer/wishlist/all", CustomerController::remove_wishlist)
        .delete("/customer/wishlist/{id}", CustomerController::remove_wishlist)
        .get("/customer/compare-items", CustomerController::compare_items)
        .post("/customer/compare-items", CustomerController::add_to_compare)
        .delete("/customer/compare-items/all", CustomerController::remove_compare)
        .delete("/customer/compare-items", CustomerController::remove_compare)
        .get("/customer/orders", CustomerController::orders)
        .get("/customer/orders/{id}", CustomerController::order_detail)
        .post("/customer/orders/{id}/cancel", CustomerController::cancel_order)
        .post("/customer/orders/{id}/reorder", CustomerController::reorder)
        // ------------------------------------------------------------------
        // Cart + checkout
        // ------------------------------------------------------------------
        .get("/checkout/cart", CartController::get)
        .post("/checkout/cart", CartController::add)
        .put("/checkout/cart", CartController::update)
        .delete("/checkout/cart", CartController::destroy)
        .delete("/checkout/cart/selected", CartController::remove_selected)
        .post("/checkout/cart/move-to-wishlist", CartController::move_to_wishlist)
        .post("/checkout/cart/coupon", CartController::apply_coupon)
        .delete("/checkout/cart/coupon", CartController::remove_coupon)
        .post("/checkout/cart/estimate-shipping-methods", CartController::estimate_shipping)
        .get("/checkout/cart/cross-sell", CartController::cross_sell)
        .get("/checkout/onepage/summary", CheckoutController::summary)
        .post("/checkout/onepage/addresses", CheckoutController::save_addresses)
        .get("/checkout/onepage/shipping-methods", CheckoutController::shipping_methods)
        .post("/checkout/onepage/shipping-methods", CheckoutController::save_shipping_method)
        .get("/checkout/onepage/payment-methods", CheckoutController::payment_methods)
        .post("/checkout/onepage/payment-methods", CheckoutController::save_payment_method)
        .post("/checkout/onepage/orders", CheckoutController::store_order)
        .get("/checkout/onepage/success/{order_id}", CheckoutController::success)
        // ------------------------------------------------------------------
        // Admin
        // ------------------------------------------------------------------
        .post("/admin/login", AuthController::admin_login)
        .post("/admin/logout", AuthController::admin_logout)
        .get("/admin/me", AuthController::admin_me)
        .get("/admin/acl", AdminAuthController::acl)
        .get("/admin/dashboard/stats", AdminDashboardController::stats)
        // catalog
        .post("/admin/catalog/products", AdminCatalogController::products_grid)
        .post("/admin/catalog/products/create", AdminCatalogController::store_product)
        .get("/admin/catalog/products/edit/{id}", AdminCatalogController::show_product)
        .put("/admin/catalog/products/edit/{id}", AdminCatalogController::update_product)
        .delete("/admin/catalog/products/{id}", AdminCatalogController::delete_product)
        .post("/admin/catalog/products/mass-delete", AdminCatalogController::mass_delete_products)
        .post("/admin/catalog/categories", AdminCatalogController::categories_grid)
        .get("/admin/catalog/categories/tree", AdminCatalogController::categories_tree)
        .post("/admin/catalog/categories/store", AdminCatalogController::store_category)
        .put("/admin/catalog/categories/edit/{id}", AdminCatalogController::update_category)
        .delete("/admin/catalog/categories/{id}", AdminCatalogController::delete_category)
        .post("/admin/catalog/attributes", AdminCatalogController::attributes_grid)
        // marketing: cart rules
        .post("/admin/marketing/cart-rules", AdminCartRuleController::grid)
        .post("/admin/marketing/cart-rules/store", AdminCartRuleController::store)
        .get("/admin/marketing/cart-rules/view/{id}", AdminCartRuleController::view)
        .put("/admin/marketing/cart-rules/edit/{id}", AdminCartRuleController::update)
        .delete("/admin/marketing/cart-rules/{id}", AdminCartRuleController::destroy)
        .post("/admin/marketing/cart-rules/coupons/{id}", AdminCartRuleController::generate_coupons)
        .get("/admin/marketing/cart-rules/coupons/{id}", AdminCartRuleController::coupons)
        // customers
        .post("/admin/customers", AdminCustomerController::grid)
        .get("/admin/customers/view/{id}", AdminCustomerController::view)
        .put("/admin/customers/edit/{id}", AdminCustomerController::update)
        .delete("/admin/customers/{id}", AdminCustomerController::destroy)
        .get("/admin/customers/groups", AdminCustomerController::groups)
        // sales
        .post("/admin/sales/orders", AdminSalesController::orders_grid)
        .get("/admin/sales/orders/view/{id}", AdminSalesController::order_view)
        .post("/admin/sales/orders/{id}/cancel", AdminSalesController::order_cancel)
        .post("/admin/sales/orders/comment/{order_id}", AdminSalesController::order_comment)
        .get("/admin/sales/orders/search", AdminSalesController::order_search)
        .post("/admin/sales/invoices/create/{order_id}", AdminSalesController::invoice_create)
        .get("/admin/sales/invoices/view/{id}", AdminSalesController::invoice_view)
        .post("/admin/sales/shipments/create/{order_id}", AdminSalesController::shipment_create)
        .get("/admin/sales/shipments/view/{id}", AdminSalesController::shipment_view)
        .post("/admin/sales/refunds/create/{order_id}", AdminSalesController::refund_create)
        .get("/admin/sales/refunds/view/{id}", AdminSalesController::refund_view)
        // settings
        .get("/admin/settings/users", AdminSettingController::users)
        .post("/admin/settings/users", AdminSettingController::store_user)
        .delete("/admin/settings/users/{id}", AdminSettingController::destroy_user)
        .get("/admin/settings/roles", AdminSettingController::roles)
        .post("/admin/settings/roles", AdminSettingController::store_role)
        .get("/admin/settings/locales", AdminSettingController::locales)
        .post("/admin/settings/locales", AdminSettingController::store_locale)
        .get("/admin/settings/currencies", AdminSettingController::currencies)
        .get("/admin/settings/channels", AdminSettingController::channels)
        .put("/admin/settings/channels/edit/{id}", AdminSettingController::update_channel)
        .get("/admin/configuration/{code}", AdminSettingController::configuration)
        .post("/admin/configuration", AdminSettingController::save_configuration)
        .get("/admin/appearance/sections/{locale}", AdminSettingController::theme_sections)
}
