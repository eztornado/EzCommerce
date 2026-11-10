-- RMA (devoluciones): solicitudes, estados, motivos, reglas, mensajes (Webkul RMA).
CREATE TABLE IF NOT EXISTS `rma_statuses` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `title` VARCHAR(255) NULL,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `color` VARCHAR(255) NULL,
    `default` INTEGER NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `rma_reasons` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `title` VARCHAR(255) NULL,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `position` INTEGER NOT NULL DEFAULT 0,
    `is_admin` BOOLEAN NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `rma_reason_resolutions` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `resolution_type` VARCHAR(255) NULL,
    `rma_reason_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`rma_reason_id`) REFERENCES `rma_reasons`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `rma_rules` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `return_period` INTEGER NULL,
    `default` BOOLEAN NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `rma` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `order_id` BIGINT NULL,
    `rma_status_id` BIGINT NULL,
    `package_condition` VARCHAR(255) NULL,
    `information` LONGTEXT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`order_id`) REFERENCES `orders`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`rma_status_id`) REFERENCES `rma_statuses`(`id`) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS `rma_items` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `quantity` INTEGER NOT NULL DEFAULT 0,
    `resolution` VARCHAR(255) NULL,
    `rma_id` BIGINT NULL,
    `rma_reason_id` BIGINT NULL,
    `order_item_id` BIGINT NULL,
    `variant_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`rma_id`) REFERENCES `rma`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`rma_reason_id`) REFERENCES `rma_reasons`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`order_item_id`) REFERENCES `order_items`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`variant_id`) REFERENCES `products`(`id`)
);

CREATE TABLE IF NOT EXISTS `rma_images` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `path` VARCHAR(255) NULL,
    `rma_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`rma_id`) REFERENCES `rma`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `rma_messages` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `message` LONGTEXT NULL,
    `attachment_path` LONGTEXT NULL,
    `attachment` LONGTEXT NULL,
    `is_admin` BOOLEAN NOT NULL DEFAULT 0,
    `rma_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`rma_id`) REFERENCES `rma`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `rma_custom_fields` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `label` VARCHAR(255) NULL,
    `type` VARCHAR(255) NULL,
    `is_required` BOOLEAN NOT NULL DEFAULT 0,
    `position` INTEGER NOT NULL DEFAULT 0,
    `input_validation` VARCHAR(255) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `rma_custom_field_options` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `value` VARCHAR(255) NULL,
    `rma_custom_field_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`rma_custom_field_id`) REFERENCES `rma_custom_fields`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `rma_additional_fields` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `value` LONGTEXT NULL,
    `rma_id` BIGINT NULL,
    `rma_custom_field_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`rma_id`) REFERENCES `rma`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`rma_custom_field_id`) REFERENCES `rma_custom_fields`(`id`) ON DELETE CASCADE
);
