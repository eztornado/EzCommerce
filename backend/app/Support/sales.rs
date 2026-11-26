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

/// Decodifica una columna JSON (sqlx::Any la entrega como texto) para que el
/// panel reciba objetos y no cadenas escapadas.
fn parse_json_field(row: &mut RowMap, key: &str) {
    let Some(raw) = row.get(key).and_then(Value::as_str) else {
        return;
    };
    if let Ok(parsed) = larust_support::serde_json::from_str::<Value>(raw) {
        row.insert(key.to_string(), parsed);
    }
}

/// Creates the order from a validated cart. Inventory is reserved
/// (`product_ordered_inventories`) inside the same sequential flow; the
/// order starts `pending` (invoice moves it to `processing`).
pub async fn create_order_from_cart(cart_id: i64) -> ApiResult<Value> {
    let cart = db::find_by_id("cart", cart_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Cart not found"))?;
    cart::collect_totals(cart_id).await?;
    // Frescura de reglas automáticas/cupón en el momento de crear el pedido.
    let _ = super::cart_rules::apply_cart_rules(cart_id).await;
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

    // Guest checkouts have no cart customer: fall back to the billing email so
    // the order still carries a contact (Bagisto shows it in the grid).
    let customer_email = if cart["customer_email"].is_null() {
        cart_addresses
            .iter()
            .find(|address| address["address_type"].as_str() == Some("cart_billing"))
            .and_then(|address| address["email"].clone().as_str().map(str::to_string))
            .map(Value::String)
            .unwrap_or(Value::Null)
    } else {
        cart["customer_email"].clone()
    };

    let order_id = db::insert(
        "orders",
        &json!({
            "increment_id": increment_id,
            "status": "pending",
            "payment_status": "pending",
            "channel_name": channel_name,
            "channel_id": cart["channel_id"].clone(),
            "is_guest": cart["is_guest"].clone(),
            "customer_id": cart["customer_id"].clone(),
            "customer_email": customer_email,
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

        // Bookings: registrar la reserva del slot elegido (producto tipo
        // booking; el slot viene en el additional del cart item).
        if item["type"].as_str() == Some("booking") {
            // El additional del cart item envuelve los extras en "options".
            let options = item["additional"]
                .get("options")
                .cloned()
                .unwrap_or_else(|| item["additional"].clone());
            let booking_from = options.get("booking_from").and_then(Value::as_i64).unwrap_or(0);
            let booking_to = options.get("booking_to").and_then(Value::as_i64).unwrap_or(0);
            let _ = db::insert(
                "bookings",
                &json!({
                    "qty": item["quantity"].clone(),
                    "from": booking_from,
                    "to": booking_to,
                    "product_id": item["product_id"].clone(),
                    "order_item_id": item_id,
                    "order_id": order_id,
                    "created_at": now(),
                    "updated_at": now(),
                }),
            )
            .await?;
        }

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
    let transactions = db::select(
        "SELECT * FROM `order_transactions` WHERE `order_id` = ? ORDER BY `id` DESC",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();
    let refunds = db::select(
        "SELECT * FROM `refunds` WHERE `order_id` = ? ORDER BY `id` DESC",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();

    let mut payment = payment;
    if let Some(row) = payment.as_mut() {
        parse_json_field(row, "additional");
    }

    let payment_method = payment
        .as_ref()
        .and_then(|row| row["method"].as_str())
        .map(str::to_string);
    // Un pedido Stripe sólo se puede enviar/reembolsar cuando el cobro está
    // confirmado (total o parcialmente reembolsado).
    let payment_blocked = payment_method.as_deref() == Some("stripe")
        && !matches!(
            order["payment_status"].as_str().unwrap_or(""),
            "paid" | "partially_refunded"
        );

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
    payload["transactions"] = Value::Array(
        transactions
            .into_iter()
            .map(|mut row| {
                parse_json_field(&mut row, "data");
                Value::Object(row)
            })
            .collect(),
    );
    payload["comments"] = Value::Array(comments.into_iter().map(Value::Object).collect());
    payload["invoices"] = Value::Array(invoices.into_iter().map(Value::Object).collect());
    payload["shipments"] = Value::Array(shipments.into_iter().map(Value::Object).collect());
    payload["refunds"] = Value::Array(refunds.into_iter().map(Value::Object).collect());

    // Derived action permissions (`canShip/canInvoice/canCancel/canRefund`).
    let totals = db::select_one(
        "SELECT CAST(SUM(`qty_ordered`) AS SIGNED) AS ordered, CAST(SUM(`qty_invoiced`) AS SIGNED) AS invoiced, \
                CAST(SUM(`qty_shipped`) AS SIGNED) AS shipped, CAST(SUM(`qty_canceled`) AS SIGNED) AS canceled, CAST(SUM(`qty_refunded`) AS SIGNED) AS refunded \
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
    let paid_gateway = payment_method.as_deref() == Some("stripe")
        && matches!(
            order["payment_status"].as_str().unwrap_or(""),
            "paid" | "partially_refunded"
        );
    let can_capture = payment_method.as_deref() == Some("stripe")
        && order["payment_status"].as_str() == Some("authorized");
    payload["can_capture"] = json!(actionable && can_capture);
    payload["can_ship"] = json!(actionable && !payment_blocked && ordered > shipped + canceled);
    // Cancelar un cobro Stripe ya pagado exige reembolsarlo primero.
    payload["can_cancel"] = json!(actionable && !paid_gateway && ordered != canceled);
    payload["can_refund"] =
        json!(actionable && !payment_blocked && ordered != refunded + canceled);

    Ok(payload)
}

/// Recomputes the order status from item quantities
/// (`OrderRepository::updateOrderStatus`).
pub async fn update_order_status(order_id: i64) -> ApiResult<()> {
    let totals = db::select_one(
        "SELECT CAST(SUM(`qty_ordered`) AS SIGNED) AS ordered, CAST(SUM(`qty_invoiced`) AS SIGNED) AS invoiced, \
                CAST(SUM(`qty_shipped`) AS SIGNED) AS shipped, CAST(SUM(`qty_canceled`) AS SIGNED) AS canceled, CAST(SUM(`qty_refunded`) AS SIGNED) AS refunded \
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

    // El estado cancelado/cerrado debe ganar al de completado: si todo se
    // canceló, `ordered == invoiced + canceled` también es cierto.
    let status = if ordered == canceled {
        "canceled"
    } else if ordered == refunded + canceled {
        "closed"
    } else if ordered == invoiced + canceled && ordered == shipped + canceled + refunded {
        "completed"
    } else {
        "processing"
    }
    .to_string();

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

    // Stripe se confirma por webhook: una factura manual no debe marcar el
    // pedido como pagado. Los métodos offline conservan el comportamiento
    // histórico (facturar = cobro confirmado por el administrador).
    let payment_method = db::select_one(
        "SELECT `method` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["method"].as_str().map(str::to_string))
    .unwrap_or_default();
    let gateway_payment = payment_method == "stripe";
    let invoice_paid = !gateway_payment || order["payment_status"].as_str() == Some("paid");
    let transaction_id = db::select_one(
        "SELECT `transaction_id` FROM `order_transactions` \
         WHERE `order_id` = ? AND `type` IN ('payment', 'capture') \
           AND `status` IN ('succeeded', 'captured', 'paid', 'completed') \
         ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["transaction_id"].as_str().map(str::to_string));

    let invoice_increment = format!("INV-{:06}", document_seq("invoices").await);
    let invoice_id = db::insert(
        "invoices",
        &json!({
            "order_id": order_id,
            "state": if invoice_paid { "paid" } else { "pending_payment" },
            "transaction_id": transaction_id,
            "increment_id": invoice_increment,
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
            // Stripe mantiene su estado hasta que el webhook confirme el cobro.
            "payment_status": if gateway_payment {
                order["payment_status"].clone()
            } else {
                json!("paid")
            },
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

    // Un cobro Stripe ya capturado debe devolverse con un reembolso; cancelar
    // el pedido dejaría el dinero retenido sin contrapartida.
    let payment_method = db::select_one(
        "SELECT `method` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["method"].as_str().map(str::to_string))
    .unwrap_or_default();
    if payment_method == "stripe"
        && matches!(
            order["payment_status"].as_str().unwrap_or(""),
            "paid" | "partially_refunded"
        )
    {
        return Err(ApiError::unprocessable(
            "Paid Stripe orders must be refunded instead of canceled.",
        ));
    }

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
                "UPDATE `product_ordered_inventories` SET `qty` = CASE WHEN `qty` - ? < 0 THEN 0 ELSE `qty` - ? END, `updated_at` = ? \
                 WHERE `product_id` = ? AND `channel_id` = ?",
                &[
                    Value::from(qty_to_cancel),
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

    let payment_status = match order["payment_status"].as_str().unwrap_or("") {
        "" | "pending" | "authorized" => json!("failed"),
        _ => order["payment_status"].clone(),
    };
    db::update(
        "orders",
        order_id,
        &json!({"payment_status": payment_status, "updated_at": now()})
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
            "UPDATE `product_inventories` SET `qty` = CASE WHEN `qty` - ? < 0 THEN 0 ELSE `qty` - ? END, `updated_at` = ? \
             WHERE `product_id` = ? AND `inventory_source_id` = ?",
            &[
                Value::from(qty),
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

/// Datos necesarios para cobrar/reembolsar la pasarela antes de escribir el
/// reembolso local.
pub struct RefundPreview {
    pub amount: f64,
    pub currency: String,
    pub method: String,
}

/// Calcula el importe que se devolvería al cliente sin tocar la base de datos.
pub async fn refund_preview(
    order_id: i64,
    requested: &Map<String, Value>,
    adjustment_refund: f64,
    adjustment_fee: f64,
) -> ApiResult<RefundPreview> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = db::select(
        "SELECT * FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await?;

    let mut sub_total = 0.0;
    let mut tax_total = 0.0;
    let mut discount_total = 0.0;
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
        let discount =
            item["base_discount_amount"].as_f64().unwrap_or(0.0) / qty_ordered * qty as f64;
        sub_total += price * qty as f64;
        tax_total += tax;
        discount_total += discount;
    }

    let shipping_amount = order["base_shipping_amount"]
        .as_f64()
        .unwrap_or(0.0)
        - order["base_shipping_refunded"].as_f64().unwrap_or(0.0);
    let base_grand_total =
        sub_total + tax_total + shipping_amount + adjustment_refund - adjustment_fee
            - discount_total;
    let rate = order["exchange_rate"]
        .as_f64()
        .filter(|rate| *rate > 0.0)
        .unwrap_or(1.0);
    let currency = order["order_currency_code"]
        .as_str()
        .or_else(|| order["base_currency_code"].as_str())
        .unwrap_or("EUR")
        .to_string();
    let method = db::select_one(
        "SELECT `method` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["method"].as_str().map(str::to_string))
    .unwrap_or_default();

    Ok(RefundPreview {
        amount: round2(base_grand_total * rate),
        currency,
        method,
    })
}

/// Creates a refund: per-item `qty_to_refund = qty_invoiced − qty_refunded`,
/// prorated totals (`campo / qty_ordered * qty`), stock returned to the
/// inventory source and `product_ordered_inventories` released.
pub async fn create_refund(
    order_id: i64,
    requested: &Map<String, Value>,
    adjustment_refund: f64,
    adjustment_fee: f64,
    transaction_id: Option<&str>,
    state: &str,
) -> ApiResult<RowMap> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let items = db::select(
        "SELECT * FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await?;

    let refund_increment = format!("REF-{:06}", document_seq("refunds").await);
    let refund_id = db::insert(
        "refunds",
        &json!({
            "order_id": order_id,
            "state": state,
            "transaction_id": transaction_id,
            "increment_id": refund_increment,
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
                Value::from(qty),
                Value::String(now()),
                item["product_id"].clone(),
            ],
        )
        .await;
        let _ = db::execute(
            "UPDATE `product_ordered_inventories` SET `qty` = CASE WHEN `qty` - ? < 0 THEN 0 ELSE `qty` - ? END, `updated_at` = ? \
             WHERE `product_id` = ?",
            &[
                Value::from(qty),
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

    let refunded_total =
        round2(order["base_grand_total_refunded"].as_f64().unwrap_or(0.0) + grand_total);
    let paid_total = order["base_grand_total"].as_f64().unwrap_or(0.0);
    let payment_status = if state == "refunded" {
        match order["payment_status"].as_str().unwrap_or("") {
            "paid" | "partially_refunded" | "refunded" => {
                if refunded_total >= paid_total - 0.005 {
                    json!("refunded")
                } else {
                    json!("partially_refunded")
                }
            }
            _ => order["payment_status"].clone(),
        }
    } else {
        order["payment_status"].clone()
    };

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
            "payment_status": payment_status,
            "updated_at": now(),
        })
        ,
    )
    .await?;

    if let Some(transaction_id) = transaction_id {
        let rate = order["exchange_rate"]
            .as_f64()
            .filter(|rate| *rate > 0.0)
            .unwrap_or(1.0);
        let currency = order["order_currency_code"]
            .as_str()
            .or_else(|| order["base_currency_code"].as_str())
            .unwrap_or("EUR");
        let status = if state == "refunded" {
            "succeeded"
        } else {
            state
        };
        let _ = upsert_order_transaction(
            order_id,
            transaction_id,
            "refund",
            status,
            round2(grand_total * rate),
            currency,
            json!({
                "refund_id": refund_id,
                "increment_id": refund_increment,
            }),
        )
        .await;
    }

    update_order_status(order_id).await?;
    db::find_by_id("refunds", refund_id)
        .await
        .map(|refund| refund.unwrap_or_else(db::empty_row))
}

/// Mezcla datos en `order_payment.additional` (JSON) sin perder las claves
/// que ya hubiera guardado el checkout.
pub async fn merge_payment_additional(order_id: i64, extra: Value) -> ApiResult<()> {
    let row = db::select_one(
        "SELECT `id`, `additional` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await?;
    let Some(row) = row else {
        return Ok(());
    };
    let mut data = row["additional"]
        .as_str()
        .and_then(|raw| larust_support::serde_json::from_str::<Value>(raw).ok())
        .unwrap_or_else(|| json!({}));
    if !data.is_object() {
        data = json!({});
    }
    if let (Some(target), Some(source)) = (data.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    db::update(
        "order_payment",
        row["id"].as_i64().unwrap_or(0),
        &json!({"additional": data, "updated_at": now()}),
    )
    .await?;
    Ok(())
}

/// Confirma el cobro de Stripe: registra la transacción, marca el pago como
/// `paid`, guarda los datos de la tarjeta y emite la factura pendiente.
/// Es idempotente por `transaction_id`.
pub async fn confirm_payment(
    order_id: i64,
    transaction_id: &str,
    amount: f64,
    currency: &str,
    data: Value,
) -> ApiResult<()> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let already_paid = order["payment_status"].as_str() == Some("paid");

    let existing = db::select_one(
        "SELECT `id` FROM `order_transactions` WHERE `transaction_id` = ? AND `type` = 'payment' LIMIT 1",
        &[Value::String(transaction_id.to_string())],
    )
    .await?;
    if existing.is_none() {
        let rate = order["exchange_rate"]
            .as_f64()
            .filter(|rate| *rate > 0.0)
            .unwrap_or(1.0);
        db::insert(
            "order_transactions",
            &json!({
                "transaction_id": transaction_id,
                "status": "succeeded",
                "type": "payment",
                "amount": round2(amount),
                "base_amount": round2(amount / rate),
                "payment_method": "stripe",
                "currency_code": currency.to_ascii_uppercase(),
                "data": data.clone(),
                "order_id": order_id,
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await?;
    }

    merge_payment_additional(order_id, data).await?;
    if !already_paid {
        db::update(
            "orders",
            order_id,
            &json!({"payment_status": "paid", "updated_at": now()}),
        )
        .await?;
    }

    let invoiceable = db::scalar_i64(
        "SELECT COUNT(*) FROM `order_items` \
         WHERE `order_id` = ? AND `qty_ordered` > `qty_invoiced` + `qty_canceled`",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or(0);
    if invoiceable > 0 {
        create_invoice(order_id).await?;
    } else {
        update_order_status(order_id).await?;
    }
    Ok(())
}

async fn upsert_order_transaction(
    order_id: i64,
    transaction_id: &str,
    kind: &str,
    status: &str,
    amount: f64,
    currency: &str,
    data: Value,
) -> ApiResult<()> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let rate = order["exchange_rate"]
        .as_f64()
        .filter(|rate| *rate > 0.0)
        .unwrap_or(1.0);
    let existing = db::select_one(
        "SELECT `id` FROM `order_transactions` WHERE `transaction_id` = ? AND `type` = ? LIMIT 1",
        &[
            Value::String(transaction_id.to_string()),
            Value::String(kind.to_string()),
        ],
    )
    .await?;
    let payload = json!({
        "status": status,
        "amount": round2(amount),
        "base_amount": round2(amount / rate),
        "payment_method": "stripe",
        "currency_code": currency.to_ascii_uppercase(),
        "data": data,
        "order_id": order_id,
        "updated_at": now(),
    });
    match existing {
        Some(row) => {
            db::update(
                "order_transactions",
                row["id"].as_i64().unwrap_or(0),
                &payload,
            )
            .await?;
        }
        None => {
            let mut row = payload;
            if let Some(object) = row.as_object_mut() {
                object.insert("transaction_id".into(), json!(transaction_id));
                object.insert("type".into(), json!(kind));
                object.insert("created_at".into(), json!(now()));
            }
            db::insert("order_transactions", &row).await?;
        }
    }
    Ok(())
}

/// Pago autorizado con captura manual pendiente (`requires_capture`).
pub async fn mark_payment_authorized(
    order_id: i64,
    transaction_id: &str,
    amount: f64,
    currency: &str,
    data: Value,
) -> ApiResult<()> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    if order["payment_status"].as_str() == Some("paid") {
        return Ok(());
    }
    upsert_order_transaction(
        order_id,
        transaction_id,
        "authorization",
        "authorized",
        amount,
        currency,
        data.clone(),
    )
    .await?;
    merge_payment_additional(order_id, data).await?;
    db::update(
        "orders",
        order_id,
        &json!({"payment_status": "authorized", "updated_at": now()}),
    )
    .await?;
    Ok(())
}

/// Marca el pago como fallido sin degradar un cobro ya confirmado.
pub async fn mark_payment_failed(order_id: i64, reason: &str) -> ApiResult<()> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    if order["payment_status"].as_str() == Some("paid") {
        return Ok(());
    }
    db::update(
        "orders",
        order_id,
        &json!({"payment_status": "failed", "updated_at": now()}),
    )
    .await?;
    merge_payment_additional(
        order_id,
        json!({"error": reason, "failed_at": now()}),
    )
    .await?;
    Ok(())
}

/// Registra una disputa/chargeback abierta.
pub async fn mark_payment_disputed(
    order_id: i64,
    dispute_id: &str,
    status: &str,
    reason: &str,
    amount: f64,
    currency: &str,
) -> ApiResult<()> {
    db::update(
        "orders",
        order_id,
        &json!({"payment_status": "disputed", "updated_at": now()}),
    )
    .await?;
    let data = json!({
        "dispute_id": dispute_id,
        "dispute_status": status,
        "dispute_reason": reason,
        "disputed_at": now(),
    });
    merge_payment_additional(order_id, data.clone()).await?;
    upsert_order_transaction(
        order_id,
        dispute_id,
        "dispute",
        status,
        amount,
        currency,
        data,
    )
    .await?;
    Ok(())
}

/// Sincroniza el estado de un reembolso de Stripe (`pending` -> `refunded`).
pub async fn update_refund_state(transaction_id: &str, state: &str) -> ApiResult<()> {
    let Some(refund) = db::select_one(
        "SELECT `id`, `order_id` FROM `refunds` WHERE `transaction_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::String(transaction_id.to_string())],
    )
    .await?
    else {
        return Ok(());
    };
    let mapped = match state {
        "succeeded" => "refunded",
        "failed" | "canceled" => "failed",
        _ => "pending",
    };
    db::update(
        "refunds",
        refund["id"].as_i64().unwrap_or(0),
        &json!({"state": mapped, "updated_at": now()}),
    )
    .await?;
    let transaction_status = if mapped == "refunded" {
        "succeeded"
    } else {
        mapped
    };
    let _ = db::execute(
        "UPDATE `order_transactions` SET `status` = ?, `updated_at` = ? \
         WHERE `transaction_id` = ? AND `type` = 'refund'",
        &[
            Value::String(transaction_status.to_string()),
            Value::String(now()),
            Value::String(transaction_id.to_string()),
        ],
    )
    .await;
    if mapped == "refunded" {
        let order_id = refund["order_id"].as_i64().unwrap_or(0);
        if let Some(order) = db::find_by_id("orders", order_id).await? {
            let refunded_total = order["base_grand_total_refunded"].as_f64().unwrap_or(0.0);
            let paid_total = order["base_grand_total"].as_f64().unwrap_or(0.0);
            let payment_status = if refunded_total >= paid_total - 0.005 {
                "refunded"
            } else {
                "partially_refunded"
            };
            db::update(
                "orders",
                order_id,
                &json!({"payment_status": payment_status, "updated_at": now()}),
            )
            .await?;
        }
    }
    Ok(())
}

/// Cierra una disputa: si se gana vuelve a `paid`; si se pierde queda
/// `refunded` (los fondos se han retirado).
pub async fn mark_dispute_closed(
    order_id: i64,
    dispute_id: &str,
    status: &str,
    reason: &str,
    amount: f64,
    currency: &str,
) -> ApiResult<()> {
    let data = json!({
        "dispute_id": dispute_id,
        "dispute_status": status,
        "dispute_reason": reason,
        "dispute_closed_at": now(),
    });
    merge_payment_additional(order_id, data.clone()).await?;
    upsert_order_transaction(
        order_id,
        dispute_id,
        "dispute",
        status,
        amount,
        currency,
        data,
    )
    .await?;
    let payment_status = match status {
        "won" => "paid",
        "lost" | "charge_refunded" => "refunded",
        _ => "disputed",
    };
    db::update(
        "orders",
        order_id,
        &json!({"payment_status": payment_status, "updated_at": now()}),
    )
    .await?;
    Ok(())
}

/// Siguiente número correlativo de un documento de ventas (facturas y
/// reembolsos usan increment_id con prefijo propio).
async fn document_seq(table: &str) -> i64 {
    db::scalar_i64(&format!("SELECT COUNT(*) FROM `{table}`"), &[]).await.unwrap_or(0) + 1
}
