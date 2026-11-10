//! Cart -> Order transformation and order lifecycle helpers. Mirrors
//! `Sales\Transformers\OrderResource`, `OrderRepository::create`,
//! `updateOrderStatus` (state derived from quantities) and inventory
//! reservation via `product_ordered_inventories`.

use larust_support::serde_json::{json, Map, Value};

use super::cart;
use super::db::{self, RowMap};
use super::response::{ApiError, ApiResult};

fn now() -> String {
    db::now_db()
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Creates the order from a validated cart. Inventory is reserved
/// (`product_ordered_inventories`) inside the same sequential flow; the
/// order starts `pending` (invoice moves it to `processing`).
pub async fn create_order_from_cart(cart_id: i64) -> ApiResult<Value> {
    let cart = db::find_by_id("cart", cart_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Cart not found"))?;
    cart::collect_totals(cart_id).await?;
    let cart = db::find_by_id("cart", cart_id).await?.unwrap_or_else(db::empty_row);

    let items = db::select(
        "SELECT * FROM `cart_items` WHERE `cart_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(cart_id)],
    )
    .await?;
    if items.is_empty() {
        return Err(ApiError::unprocessable("The cart is empty."));
    }

    let channel = db::find_by_id("channels", cart["channel_id"].as_i64().unwrap_or(1))
        .await
        .unwrap_or(None);
    let channel_name = channel
        .as_ref()
        .and_then(|channel| channel["code"].as_str())
        .unwrap_or("default")
        .to_string();

    // Addresses (cart_billing / cart_shipping) -> order_billing/order_shipping.
    let cart_addresses = db::select(
        "SELECT * FROM `addresses` WHERE `cart_id` = ? ORDER BY `id`",
        &[Value::from(cart_id)],
    )
    .await?;

    let payment = db::select_one(
        "SELECT * FROM `cart_payment` WHERE `cart_id` = ? LIMIT 1",
        &[Value::from(cart_id)],
    )
    .await?
    .ok_or_else(|| ApiError::unprocessable("Payment method not selected."))?;

    let next_id: i64 = db::scalar_i64("SELECT COALESCE(MAX(`id`), 0) + 1 FROM `orders`", &[]).await?;
    let increment_id = format!("{next_id:08}");

    let order_id = db::insert(
        "orders",
        &json!({
            "increment_id": increment_id,
            "status": "pending",
            "channel_name": channel_name,
            "channel_id": cart["channel_id"].clone(),
            "is_guest": cart["is_guest"].clone(),
            "customer_id": cart["customer_id"].clone(),
            "customer_email": cart["customer_email"].clone(),
            "customer_first_name": cart["customer_first_name"].clone(),
            "customer_last_name": cart["customer_last_name"].clone(),
            "shipping_method": cart["shipping_method"].clone(),
            "coupon_code": cart["coupon_code"].clone(),
            "total_item_count": cart["items_count"].clone(),
            "total_qty_ordered": cart["items_qty"].clone(),
            "base_currency_code": cart["base_currency_code"].clone(),
            "channel_currency_code": cart["channel_currency_code"].clone(),
            "order_currency_code": cart["cart_currency_code"].clone(),
            "grand_total": cart["grand_total"].clone(),
            "base_grand_total": cart["base_grand_total"].clone(),
            "sub_total": cart["sub_total"].clone(),
            "base_sub_total": cart["base_sub_total"].clone(),
            "tax_amount": cart["tax_total"].clone(),
            "base_tax_amount": cart["base_tax_total"].clone(),
            "discount_amount": cart["discount_amount"].clone(),
            "base_discount_amount": cart["base_discount_amount"].clone(),
            "shipping_amount": cart["shipping_amount"].clone(),
            "base_shipping_amount": cart["base_shipping_amount"].clone(),
            "cart_id": cart_id,
            "created_at": now(),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    // Order addresses.
    for address in &cart_addresses {
        let mut row = address.clone();
        let address_type = row["address_type"].as_str().unwrap_or("cart_shipping").to_string();
        row.insert("order_id".into(), json!(order_id));
        row.insert("address_type".into(), json!(address_type.replacen("cart_", "order_", 1)));
        row.insert("cart_id".into(), Value::Null);
        row.insert("id".into(), Value::Null);
        row.insert("created_at".into(), json!(now()));
        row.insert("updated_at".into(), json!(now()));
        db::insert("addresses", &row).await?;
    }

    // Payment.
    db::insert(
        "order_payment",
        &json!({
            "order_id": order_id,
            "method": payment["method"].clone(),
            "method_title": payment["method_title"].clone(),
            "created_at": now(),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    // Items (+ inventory reservation).
    let shipping_rate = db::select_one(
        "SELECT * FROM `cart_shipping_rates` WHERE `cart_id` = ? AND `is_selected` = 1 LIMIT 1",
        &[Value::from(cart_id)],
    )
    .await
    .unwrap_or(None);
    for item in &items {
        let item_id = db::insert(
            "order_items",
            &json!({
                "order_id": order_id,
                "product_id": item["product_id"].clone(),
                "product_type": item["type"].clone(),
                "type": item["type"].clone(),
                "sku": item["sku"].clone(),
                "name": item["name"].clone(),
                "qty_ordered": item["quantity"].clone(),
                "price": item["price"].clone(),
                "base_price": item["base_price"].clone(),
                "total": item["total"].clone(),
                "base_total": item["base_total"].clone(),
                "tax_percent": item["tax_percent"].clone(),
                "tax_amount": item["tax_amount"].clone(),
                "base_tax_amount": item["base_tax_amount"].clone(),
                "discount_percent": item["discount_percent"].clone(),
                "discount_amount": item["discount_amount"].clone(),
                "base_discount_amount": item["base_discount_amount"].clone(),
                "weight": item["weight"].clone(),
                "total_weight": item["total_weight"].clone(),
                "additional": item["additional"].clone(),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await?;

        // Reserve stock for stockable products.
        if is_stockable(item["type"].as_str().unwrap_or("simple")) {
            let product_id = item["product_id"].as_i64().unwrap_or(0);
            if product_id > 0 {
                let existing = db::select_one(
                    "SELECT `id`, `qty` FROM `product_ordered_inventories` WHERE `product_id` = ? AND `channel_id` = ? LIMIT 1",
                    &[item["product_id"].clone(), cart["channel_id"].clone()],
                )
                .await
                .unwrap_or(None);
                match existing {
                    Some(row) => {
                        db::update(
                            "product_ordered_inventories",
                            row["id"].as_i64().unwrap_or(0),
                            &json!({"qty": row["qty"].as_i64().unwrap_or(0) + item["quantity"].as_i64().unwrap_or(0), "updated_at": now()}),
                        )
                        .await?;
                    }
                    None => {
                        db::insert(
                            "product_ordered_inventories",
                            &json!({
                                "product_id": item["product_id"].clone(),
                                "channel_id": cart["channel_id"].clone(),
                                "qty": item["quantity"].as_i64().unwrap_or(0),
                                "created_at": now(),
                                "updated_at": now(),
                            })
                            ,
                        )
                        .await?;
                    }
                }
            }
        }
        let _ = item_id;
    }

    let _ = shipping_rate;

    // Deactivate the cart.
    db::update(
        "cart",
        cart_id,
        &json!({"is_active": 0, "updated_at": now()})
            ,
    )
    .await?;

    order_resource(order_id).await
}

fn is_stockable(product_type: &str) -> bool {
    !matches!(product_type, "downloadable" | "booking" | "virtual")
}

/// Order detail with items, addresses, payment, comments.
pub async fn order_resource(order_id: i64) -> ApiResult<Value> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = db::select(
        "SELECT * FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();
    let addresses = db::select(
        "SELECT * FROM `addresses` WHERE `order_id` = ? ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();
    let payment = db::select_one(
        "SELECT * FROM `order_payment` WHERE `order_id` = ? LIMIT 1",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or(None);
    let comments = db::select(
        "SELECT * FROM `order_comments` WHERE `order_id` = ? ORDER BY `id` DESC",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();
    let invoices = db::select(
        "SELECT * FROM `invoices` WHERE `order_id` = ? ORDER BY `id` DESC",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();
    let shipments = db::select(
        "SELECT * FROM `shipments` WHERE `order_id` = ? ORDER BY `id` DESC",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();

    let mut payload = Value::Object(order.clone());
    payload["items"] = Value::Array(items.into_iter().map(Value::Object).collect());
    payload["billing_address"] = addresses
        .iter()
        .find(|address| address["address_type"].as_str() == Some("order_billing"))
        .cloned()
        .map(Value::Object)
        .unwrap_or(Value::Null);
    payload["shipping_address"] = addresses
        .iter()
        .find(|address| address["address_type"].as_str() == Some("order_shipping"))
        .cloned()
        .map(Value::Object)
        .unwrap_or(Value::Null);
    payload["payment"] = payment.map(Value::Object).unwrap_or(Value::Null);
    payload["comments"] = Value::Array(comments.into_iter().map(Value::Object).collect());
    payload["invoices"] = Value::Array(invoices.into_iter().map(Value::Object).collect());
    payload["shipments"] = Value::Array(shipments.into_iter().map(Value::Object).collect());

    // Derived action permissions (`canShip/canInvoice/canCancel/canRefund`).
    let totals = db::select_one(
        "SELECT SUM(`qty_ordered`) AS ordered, SUM(`qty_invoiced`) AS invoiced, \
                SUM(`qty_shipped`) AS shipped, SUM(`qty_canceled`) AS canceled, SUM(`qty_refunded`) AS refunded \
         FROM `order_items` WHERE `order_id` = ?",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or(None);
    let ordered = totals.as_ref().and_then(|row| row["ordered"].as_i64()).unwrap_or(0);
    let invoiced = totals.as_ref().and_then(|row| row["invoiced"].as_i64()).unwrap_or(0);
    let shipped = totals.as_ref().and_then(|row| row["shipped"].as_i64()).unwrap_or(0);
    let canceled = totals.as_ref().and_then(|row| row["canceled"].as_i64()).unwrap_or(0);
    let refunded = totals.as_ref().and_then(|row| row["refunded"].as_i64()).unwrap_or(0);

    let status = order["status"].as_str().unwrap_or("pending").to_string();
    let actionable = status != "closed" && status != "fraud";
    payload["can_invoice"] = json!(actionable && ordered > invoiced + canceled);
    payload["can_ship"] = json!(actionable && ordered > shipped + canceled);
    payload["can_cancel"] = json!(actionable && ordered != canceled);
    payload["can_refund"] = json!(actionable && ordered != refunded + canceled);

    Ok(payload)
}

/// Recomputes the order status from item quantities
/// (`OrderRepository::updateOrderStatus`).
pub async fn update_order_status(order_id: i64) -> ApiResult<()> {
    let totals = db::select_one(
        "SELECT SUM(`qty_ordered`) AS ordered, SUM(`qty_invoiced`) AS invoiced, \
                SUM(`qty_shipped`) AS shipped, SUM(`qty_canceled`) AS canceled, SUM(`qty_refunded`) AS refunded \
         FROM `order_items` WHERE `order_id` = ?",
        &[Value::from(order_id)],
    )
    .await?
    .unwrap_or_else(db::empty_row);
    let ordered = totals["ordered"].as_i64().unwrap_or(0);
    let invoiced = totals["invoiced"].as_i64().unwrap_or(0);
    let shipped = totals["shipped"].as_i64().unwrap_or(0);
    let canceled = totals["canceled"].as_i64().unwrap_or(0);
    let refunded = totals["refunded"].as_i64().unwrap_or(0);

    let mut status = "processing".to_string();
    if ordered == invoiced + canceled && ordered == shipped + canceled + refunded {
        status = "completed".to_string();
    } else if ordered == canceled {
        status = "canceled".to_string();
    } else if ordered == refunded + canceled {
        status = "closed".to_string();
    }

    db::update(
        "orders",
        order_id,
        &json!({"status": status, "updated_at": now()})
            ,
    )
    .await?;
    Ok(())
}

/// Creates an invoice for the order (offline payments: `paid`), prorating
/// tax/discount per item (`campo / qty_ordered * qty`) and refreshing the
/// order totals.
pub async fn create_invoice(order_id: i64) -> ApiResult<RowMap> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = db::select(
        "SELECT * FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await?;

    let invoice_id = db::insert(
        "invoices",
        &json!({
            "order_id": order_id,
            "state": "paid",
            "base_currency_code": order["base_currency_code"].clone(),
            "channel_currency_code": order["channel_currency_code"].clone(),
            "order_currency_code": order["order_currency_code"].clone(),
            "created_at": now(),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    let mut sub_total = 0.0;
    let mut tax_total = 0.0;
    let mut discount_total = 0.0;
    let mut total_qty = 0.0;

    for item in &items {
        let qty_ordered = item["qty_ordered"].as_i64().unwrap_or(0) as f64;
        let qty_to_invoice = item["qty_ordered"].as_i64().unwrap_or(0)
            - item["qty_invoiced"].as_i64().unwrap_or(0)
            - item["qty_canceled"].as_i64().unwrap_or(0);
        if qty_to_invoice <= 0 {
            continue;
        }
        let qty = qty_to_invoice as f64;
        let price = item["base_price"].as_f64().unwrap_or(0.0);
        let tax = if qty_ordered > 0.0 {
            item["base_tax_amount"].as_f64().unwrap_or(0.0) / qty_ordered * qty
        } else {
            0.0
        };
        let discount = if qty_ordered > 0.0 {
            item["base_discount_amount"].as_f64().unwrap_or(0.0) / qty_ordered * qty
        } else {
            0.0
        };
        let total = price * qty;

        db::insert(
            "invoice_items",
            &json!({
                "invoice_id": invoice_id,
                "order_item_id": item["id"].clone(),
                "product_id": item["product_id"].clone(),
                "product_type": item["product_type"].clone(),
                "name": item["name"].clone(),
                "sku": item["sku"].clone(),
                "qty": qty_to_invoice,
                "price": price,
                "base_price": price,
                "total": round2(total),
                "base_total": round2(total),
                "tax_amount": round2(tax),
                "base_tax_amount": round2(tax),
                "discount_amount": round2(discount),
                "base_discount_amount": round2(discount),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await?;

        db::update(
            "order_items",
            item["id"].as_i64().unwrap_or(0),
            &json!({
                "qty_invoiced": item["qty_invoiced"].as_i64().unwrap_or(0) + qty_to_invoice,
                "total_invoiced": round2(item["total_invoiced"].as_f64().unwrap_or(0.0) + total),
                "base_total_invoiced": round2(item["base_total_invoiced"].as_f64().unwrap_or(0.0) + total),
                "tax_amount_invoiced": round2(item["tax_amount_invoiced"].as_f64().unwrap_or(0.0) + tax),
                "base_tax_amount_invoiced": round2(item["base_tax_amount_invoiced"].as_f64().unwrap_or(0.0) + tax),
                "discount_invoiced": round2(item["discount_invoiced"].as_f64().unwrap_or(0.0) + discount),
                "base_discount_invoiced": round2(item["base_discount_invoiced"].as_f64().unwrap_or(0.0) + discount),
                "updated_at": now(),
            })
            ,
        )
        .await?;

        sub_total += total;
        tax_total += tax;
        discount_total += discount;
        total_qty += qty;
    }

    // Shipping is invoiced with the first invoice only.
    let prior_invoices = db::scalar_i64(
        "SELECT COUNT(*) FROM `invoices` WHERE `order_id` = ?",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or(1);
    let shipping_amount = if prior_invoices <= 1 {
        order["base_shipping_amount"].as_f64().unwrap_or(0.0)
    } else {
        0.0
    };
    let grand_total = sub_total + tax_total + shipping_amount - discount_total;

    db::update(
        "invoices",
        invoice_id,
        &json!({
            "total_qty": round2(total_qty),
            "sub_total": round2(sub_total),
            "base_sub_total": round2(sub_total),
            "grand_total": round2(grand_total),
            "base_grand_total": round2(grand_total),
            "shipping_amount": round2(shipping_amount),
            "base_shipping_amount": round2(shipping_amount),
            "tax_amount": round2(tax_total),
            "base_tax_amount": round2(tax_total),
            "discount_amount": round2(discount_total),
            "base_discount_amount": round2(discount_total),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    db::update(
        "orders",
        order_id,
        &json!({
            "grand_total_invoiced": round2(order["grand_total_invoiced"].as_f64().unwrap_or(0.0) + grand_total),
            "base_grand_total_invoiced": round2(order["base_grand_total_invoiced"].as_f64().unwrap_or(0.0) + grand_total),
            "sub_total_invoiced": round2(order["sub_total_invoiced"].as_f64().unwrap_or(0.0) + sub_total),
            "base_sub_total_invoiced": round2(order["base_sub_total_invoiced"].as_f64().unwrap_or(0.0) + sub_total),
            "tax_amount_invoiced": round2(order["tax_amount_invoiced"].as_f64().unwrap_or(0.0) + tax_total),
            "base_tax_amount_invoiced": round2(order["base_tax_amount_invoiced"].as_f64().unwrap_or(0.0) + tax_total),
            "discount_invoiced": round2(order["discount_invoiced"].as_f64().unwrap_or(0.0) + discount_total),
            "base_discount_invoiced": round2(order["base_discount_invoiced"].as_f64().unwrap_or(0.0) + discount_total),
            "shipping_invoiced": round2(order["shipping_invoiced"].as_f64().unwrap_or(0.0) + shipping_amount),
            "base_shipping_invoiced": round2(order["base_shipping_invoiced"].as_f64().unwrap_or(0.0) + shipping_amount),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    update_order_status(order_id).await?;
    db::find_by_id("invoices", invoice_id).await.map(|invoice| invoice.unwrap_or_else(db::empty_row))
}

/// Cancels the order: returns reserved stock, marks quantities canceled and
/// recomputes the status.
pub async fn cancel_order(order_id: i64) -> ApiResult<()> {
    let items = db::select(
        "SELECT * FROM `order_items` WHERE `order_id` = ? ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await?;
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;

    for item in &items {
        let qty_to_cancel = item["qty_ordered"].as_i64().unwrap_or(0)
            - item["qty_canceled"].as_i64().unwrap_or(0)
            - item["qty_invoiced"].as_i64().unwrap_or(0);
        if qty_to_cancel <= 0 {
            continue;
        }
        if is_stockable(item["type"].as_str().unwrap_or("simple")) {
            let product_id = item["product_id"].as_i64().unwrap_or(0);
            let _ = db::execute(
                "UPDATE `product_ordered_inventories` SET `qty` = MAX(`qty` - ?, 0), `updated_at` = ? \
                 WHERE `product_id` = ? AND `channel_id` = ?",
                &[
                    Value::from(qty_to_cancel),
                    Value::String(now()),
                    Value::from(product_id),
                    order["channel_id"].clone(),
                ],
            )
            .await;
        }
        db::update(
            "order_items",
            item["id"].as_i64().unwrap_or(0),
            &json!({
                "qty_canceled": item["qty_canceled"].as_i64().unwrap_or(0) + qty_to_cancel,
                "updated_at": now(),
            })
            ,
        )
        .await?;
    }

    db::update(
        "orders",
        order_id,
        &json!({"updated_at": now()})
            ,
    )
    .await?;
    update_order_status(order_id).await
}

/// Creates a shipment for the order: per-item `qty_to_ship = qty_ordered −
/// qty_shipped − qty_refunded − qty_canceled` (capped by the request),
/// discounts the inventory source and refreshes the order status.
pub async fn create_shipment(
    order_id: i64,
    source_id: i64,
    requested: &Map<String, Value>,
) -> ApiResult<RowMap> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = db::select(
        "SELECT * FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await?;

    let source = db::find_by_id("inventory_sources", source_id)
        .await?
        .unwrap_or_else(db::empty_row);

    let shipment_id = db::insert(
        "shipments",
        &json!({
            "order_id": order_id,
            "customer_id": order["customer_id"].clone(),
            "carrier_code": "flatrate",
            "carrier_title": "Flat Rate",
            "track_number": "",
            "inventory_source_id": if source_id > 0 { json!(source_id) } else { Value::Null },
            "inventory_source_name": source["name"].clone(),
            "created_at": now(),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    let mut total_qty = 0i64;
    for item in &items {
        let item_key = item["id"].as_i64().unwrap_or(0).to_string();
        let qty_requested = requested
            .get(&item_key)
            .or_else(|| requested.get("all"))
            .and_then(Value::as_i64);
        let qty_to_ship = item["qty_ordered"].as_i64().unwrap_or(0)
            - item["qty_shipped"].as_i64().unwrap_or(0)
            - item["qty_refunded"].as_i64().unwrap_or(0)
            - item["qty_canceled"].as_i64().unwrap_or(0);
        let qty = qty_requested.unwrap_or(qty_to_ship).min(qty_to_ship);
        if qty <= 0 {
            continue;
        }

        // Discount the inventory source (floor at 0).
        let _ = db::execute(
            "UPDATE `product_inventories` SET `qty` = MAX(`qty` - ?, 0), `updated_at` = ? \
             WHERE `product_id` = ? AND `inventory_source_id` = ?",
            &[
                Value::from(qty),
                Value::String(now()),
                item["product_id"].clone(),
                Value::from(source_id),
            ],
        )
        .await;

        db::insert(
            "shipment_items",
            &json!({
                "shipment_id": shipment_id,
                "order_item_id": item["id"].clone(),
                "product_id": item["product_id"].clone(),
                "product_type": item["product_type"].clone(),
                "name": item["name"].clone(),
                "sku": item["sku"].clone(),
                "qty": qty,
                "weight": (item["weight"].as_f64().unwrap_or(0.0) * qty as f64) as i64,
                "price": item["price"].clone(),
                "base_price": item["base_price"].clone(),
                "total": round2(item["base_price"].as_f64().unwrap_or(0.0) * qty as f64),
                "base_total": round2(item["base_price"].as_f64().unwrap_or(0.0) * qty as f64),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await?;

        db::update(
            "order_items",
            item["id"].as_i64().unwrap_or(0),
            &json!({
                "qty_shipped": item["qty_shipped"].as_i64().unwrap_or(0) + qty,
                "updated_at": now(),
            })
            ,
        )
        .await?;

        total_qty += qty;
    }

    db::update(
        "shipments",
        shipment_id,
        &json!({"total_qty": total_qty, "status": "shipped", "updated_at": now()})
        ,
    )
    .await?;

    update_order_status(order_id).await?;
    db::find_by_id("shipments", shipment_id)
        .await
        .map(|shipment| shipment.unwrap_or_else(db::empty_row))
}

/// Creates a refund: per-item `qty_to_refund = qty_invoiced − qty_refunded`,
/// prorated totals (`campo / qty_ordered * qty`), stock returned to the
/// inventory source and `product_ordered_inventories` released.
pub async fn create_refund(
    order_id: i64,
    requested: &Map<String, Value>,
    adjustment_refund: f64,
    adjustment_fee: f64,
) -> ApiResult<RowMap> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = db::select(
        "SELECT * FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await?;

    let refund_id = db::insert(
        "refunds",
        &json!({
            "order_id": order_id,
            "state": "refunded",
            "base_currency_code": order["base_currency_code"].clone(),
            "channel_currency_code": order["channel_currency_code"].clone(),
            "order_currency_code": order["order_currency_code"].clone(),
            "adjustment_refund": round2(adjustment_refund),
            "base_adjustment_refund": round2(adjustment_refund),
            "adjustment_fee": round2(adjustment_fee),
            "base_adjustment_fee": round2(adjustment_fee),
            "created_at": now(),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    let mut sub_total = 0.0;
    let mut tax_total = 0.0;
    let mut discount_total = 0.0;
    let mut total_qty = 0.0;

    for item in &items {
        let item_key = item["id"].as_i64().unwrap_or(0).to_string();
        let qty_requested = requested
            .get(&item_key)
            .or_else(|| requested.get("all"))
            .and_then(Value::as_i64);
        let qty_to_refund = item["qty_invoiced"].as_i64().unwrap_or(0)
            - item["qty_refunded"].as_i64().unwrap_or(0);
        let qty = qty_requested.unwrap_or(qty_to_refund).min(qty_to_refund);
        if qty <= 0 {
            continue;
        }
        let qty_ordered = item["qty_ordered"].as_i64().unwrap_or(1) as f64;
        let price = item["base_price"].as_f64().unwrap_or(0.0);
        let tax = item["base_tax_amount"].as_f64().unwrap_or(0.0) / qty_ordered * qty as f64;
        let discount = item["base_discount_amount"].as_f64().unwrap_or(0.0) / qty_ordered * qty as f64;
        let total = price * qty as f64;

        db::insert(
            "refund_items",
            &json!({
                "refund_id": refund_id,
                "order_item_id": item["id"].clone(),
                "product_id": item["product_id"].clone(),
                "product_type": item["product_type"].clone(),
                "name": item["name"].clone(),
                "sku": item["sku"].clone(),
                "qty": qty,
                "price": price,
                "base_price": price,
                "total": round2(total),
                "base_total": round2(total),
                "tax_amount": round2(tax),
                "base_tax_amount": round2(tax),
                "discount_amount": round2(discount),
                "base_discount_amount": round2(discount),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await?;

        // Return stock to the source and release the reservation.
        let _ = db::execute(
            "UPDATE `product_inventories` SET `qty` = `qty` + ?, `updated_at` = ? \
             WHERE `product_id` = ? AND `inventory_source_id` = 1",
            &[
                Value::from(qty),
                Value::String(now()),
                item["product_id"].clone(),
            ],
        )
        .await;
        let _ = db::execute(
            "UPDATE `product_ordered_inventories` SET `qty` = MAX(`qty` - ?, 0), `updated_at` = ? \
             WHERE `product_id` = ?",
            &[
                Value::from(qty),
                Value::String(now()),
                item["product_id"].clone(),
            ],
        )
        .await;

        db::update(
            "order_items",
            item["id"].as_i64().unwrap_or(0),
            &json!({
                "qty_refunded": item["qty_refunded"].as_i64().unwrap_or(0) + qty,
                "amount_refunded": round2(item["amount_refunded"].as_f64().unwrap_or(0.0) + total),
                "base_amount_refunded": round2(item["base_amount_refunded"].as_f64().unwrap_or(0.0) + total),
                "tax_amount_refunded": round2(item["tax_amount_refunded"].as_f64().unwrap_or(0.0) + tax),
                "base_tax_amount_refunded": round2(item["base_tax_amount_refunded"].as_f64().unwrap_or(0.0) + tax),
                "discount_refunded": round2(item["discount_refunded"].as_f64().unwrap_or(0.0) + discount),
                "base_discount_refunded": round2(item["base_discount_refunded"].as_f64().unwrap_or(0.0) + discount),
                "updated_at": now(),
            })
            ,
        )
        .await?;

        sub_total += total;
        tax_total += tax;
        discount_total += discount;
        total_qty += qty as f64;
    }

    let shipping_amount = order["base_shipping_amount"].as_f64().unwrap_or(0.0)
        - order["base_shipping_refunded"].as_f64().unwrap_or(0.0);
    let grand_total = sub_total + tax_total + shipping_amount + adjustment_refund - adjustment_fee
        - discount_total;

    db::update(
        "refunds",
        refund_id,
        &json!({
            "total_qty": round2(total_qty),
            "sub_total": round2(sub_total),
            "base_sub_total": round2(sub_total),
            "grand_total": round2(grand_total),
            "base_grand_total": round2(grand_total),
            "shipping_amount": round2(shipping_amount),
            "base_shipping_amount": round2(shipping_amount),
            "tax_amount": round2(tax_total),
            "base_tax_amount": round2(tax_total),
            "discount_amount": round2(discount_total),
            "base_discount_amount": round2(discount_total),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    db::update(
        "orders",
        order_id,
        &json!({
            "grand_total_refunded": round2(order["grand_total_refunded"].as_f64().unwrap_or(0.0) + grand_total),
            "base_grand_total_refunded": round2(order["base_grand_total_refunded"].as_f64().unwrap_or(0.0) + grand_total),
            "sub_total_refunded": round2(order["sub_total_refunded"].as_f64().unwrap_or(0.0) + sub_total),
            "base_sub_total_refunded": round2(order["base_sub_total_refunded"].as_f64().unwrap_or(0.0) + sub_total),
            "tax_amount_refunded": round2(order["tax_amount_refunded"].as_f64().unwrap_or(0.0) + tax_total),
            "base_tax_amount_refunded": round2(order["base_tax_amount_refunded"].as_f64().unwrap_or(0.0) + tax_total),
            "discount_refunded": round2(order["discount_refunded"].as_f64().unwrap_or(0.0) + discount_total),
            "base_discount_refunded": round2(order["base_discount_refunded"].as_f64().unwrap_or(0.0) + discount_total),
            "shipping_refunded": round2(order["shipping_refunded"].as_f64().unwrap_or(0.0) + shipping_amount),
            "base_shipping_refunded": round2(order["base_shipping_refunded"].as_f64().unwrap_or(0.0) + shipping_amount),
            "updated_at": now(),
        })
        ,
    )
    .await?;

    update_order_status(order_id).await?;
    db::find_by_id("refunds", refund_id)
        .await
        .map(|refund| refund.unwrap_or_else(db::empty_row))
}
