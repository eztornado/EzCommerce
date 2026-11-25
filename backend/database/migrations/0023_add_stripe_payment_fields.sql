-- Fase 1 de la pasarela Stripe: estados de pago, trazabilidad de
-- transacciones/reembolsos y registro idempotente de webhooks.
ALTER TABLE `orders` ADD COLUMN `payment_status` VARCHAR(32) NULL;
CREATE INDEX IF NOT EXISTS `idx_orders_payment_status` ON `orders` (`payment_status`);

ALTER TABLE `refunds` ADD COLUMN `transaction_id` VARCHAR(255) NULL;
ALTER TABLE `refunds` ADD COLUMN `error_message` TEXT NULL;
CREATE INDEX IF NOT EXISTS `idx_refunds_transaction_id` ON `refunds` (`transaction_id`);

ALTER TABLE `order_transactions` ADD COLUMN `refund_id` BIGINT NULL;
ALTER TABLE `order_transactions` ADD COLUMN `currency_code` VARCHAR(3) NULL;
CREATE INDEX IF NOT EXISTS `idx_order_payment_order_id` ON `order_payment` (`order_id`);
CREATE INDEX IF NOT EXISTS `idx_order_transactions_order_id` ON `order_transactions` (`order_id`);
CREATE UNIQUE INDEX IF NOT EXISTS `idx_order_transactions_unique` ON `order_transactions` (`transaction_id`, `type`);

CREATE TABLE IF NOT EXISTS `payment_webhook_events` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `gateway` VARCHAR(32) NOT NULL DEFAULT 'stripe',
    `event_id` VARCHAR(255) NOT NULL,
    `type` VARCHAR(255) NOT NULL,
    `status` VARCHAR(32) NOT NULL DEFAULT 'received',
    `payload` JSON NOT NULL,
    `error_message` TEXT NULL,
    `processed_at` DATETIME NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS `idx_payment_webhook_events_event_id` ON `payment_webhook_events` (`event_id`);
CREATE INDEX IF NOT EXISTS `idx_payment_webhook_events_status` ON `payment_webhook_events` (`status`);

-- Los pedidos existentes se consideran pagados si ya tienen una factura
-- pagada; el resto queda pendiente hasta que se confirme el cobro.
-- `update` va en minúsculas porque el traductor SQLite de migraciones
-- reemplaza la palabra DATE y convertiría UPDATE en UPTEXT.
update `orders`
SET `payment_status` = CASE
    WHEN EXISTS (
        SELECT 1 FROM `invoices`
        WHERE `invoices`.`order_id` = `orders`.`id` AND `invoices`.`state` = 'paid'
    ) THEN 'paid'
    ELSE 'pending'
END
WHERE `payment_status` IS NULL;
