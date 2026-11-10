-- Categorías y tarifas de impuestos (Webkul Tax).
CREATE TABLE IF NOT EXISTS `tax_categories` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `tax_rates` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `identifier` VARCHAR(255) NOT NULL UNIQUE,
    `is_zip` BOOLEAN NOT NULL DEFAULT 0,
    `zip_code` VARCHAR(255) NULL,
    `zip_from` VARCHAR(255) NULL,
    `zip_to` VARCHAR(255) NULL,
    `state` VARCHAR(255) NULL,
    `country` VARCHAR(255) NULL,
    `tax_rate` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `tax_categories_tax_rates` (
    `tax_category_id` BIGINT NOT NULL,
    `tax_rate_id` BIGINT NOT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    PRIMARY KEY (`tax_category_id`, `tax_rate_id`),
    FOREIGN KEY (`tax_category_id`) REFERENCES `tax_categories`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`tax_rate_id`) REFERENCES `tax_rates`(`id`) ON DELETE CASCADE
);
