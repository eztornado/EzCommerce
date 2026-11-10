-- Tipos de producto: bundle, downloadable y opciones personalizables (Webkul Product).
CREATE TABLE IF NOT EXISTS `product_bundle_options` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NULL,
    `is_required` BOOLEAN NOT NULL DEFAULT 1,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_bundle_option_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `label` VARCHAR(255) NULL,
    `product_bundle_option_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_bundle_option_id`) REFERENCES `product_bundle_options`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_bundle_option_products` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `qty` INTEGER NULL,
    `is_user_defined` BOOLEAN NOT NULL DEFAULT 1,
    `is_default` BOOLEAN NOT NULL DEFAULT 0,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `product_bundle_option_id` BIGINT NULL,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_bundle_option_id`) REFERENCES `product_bundle_options`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_downloadable_links` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `url` LONGTEXT NULL,
    `file` LONGTEXT NULL,
    `file_name` LONGTEXT NULL,
    `type` VARCHAR(255) NULL,
    `price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `sample_url` LONGTEXT NULL,
    `sample_file` LONGTEXT NULL,
    `sample_file_name` LONGTEXT NULL,
    `sample_type` VARCHAR(255) NULL,
    `downloads` INTEGER NOT NULL DEFAULT 0,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_downloadable_link_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `title` VARCHAR(255) NULL,
    `product_downloadable_link_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_downloadable_link_id`) REFERENCES `product_downloadable_links`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_downloadable_samples` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `url` LONGTEXT NULL,
    `file` LONGTEXT NULL,
    `file_name` LONGTEXT NULL,
    `type` VARCHAR(255) NULL,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_downloadable_sample_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `title` VARCHAR(255) NULL,
    `product_downloadable_sample_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_downloadable_sample_id`) REFERENCES `product_downloadable_samples`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_customizable_options` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NULL,
    `is_required` BOOLEAN NOT NULL DEFAULT 1,
    `max_characters` INTEGER NULL,
    `supported_file_extensions` VARCHAR(255) NULL,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_customizable_option_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `label` VARCHAR(255) NULL,
    `product_customizable_option_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_customizable_option_id`) REFERENCES `product_customizable_options`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_customizable_option_prices` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `label` VARCHAR(255) NULL,
    `price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `product_customizable_option_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_customizable_option_id`) REFERENCES `product_customizable_options`(`id`) ON DELETE CASCADE
);
