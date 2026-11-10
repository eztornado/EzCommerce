-- Canales, pivotes canal↔locale/moneda/almacén y configuración global (Webkul Core).
CREATE TABLE IF NOT EXISTS `channels` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NULL UNIQUE,
    `timezone` VARCHAR(255) NULL,
    `theme` VARCHAR(255) NULL,
    `hostname` VARCHAR(255) NULL,
    `logo` VARCHAR(255) NULL,
    `favicon` VARCHAR(255) NULL,
    `home_seo` JSON NULL,
    `is_maintenance_on` BOOLEAN NOT NULL DEFAULT 0,
    `allowed_ips` LONGTEXT NULL,
    `root_category_id` BIGINT NULL,
    `default_locale_id` BIGINT NULL,
    `base_currency_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`root_category_id`) REFERENCES `categories`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`default_locale_id`) REFERENCES `locales`(`id`),
    FOREIGN KEY (`base_currency_id`) REFERENCES `currencies`(`id`)
);
CREATE INDEX IF NOT EXISTS `channels_hostname_index` ON `channels` (`hostname`);

CREATE TABLE IF NOT EXISTS `channel_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `channel_id` BIGINT NULL,
    `locale` VARCHAR(255) NULL,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `home_page_content` LONGTEXT NULL,
    `footer_content` LONGTEXT NULL,
    `maintenance_mode_text` LONGTEXT NULL,
    `home_seo` JSON NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `channel_locales` (
    `channel_id` BIGINT NOT NULL,
    `locale_id` BIGINT NOT NULL,
    PRIMARY KEY (`channel_id`, `locale_id`),
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`locale_id`) REFERENCES `locales`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `channel_currencies` (
    `channel_id` BIGINT NOT NULL,
    `currency_id` BIGINT NOT NULL,
    PRIMARY KEY (`channel_id`, `currency_id`),
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`currency_id`) REFERENCES `currencies`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `channel_inventory_sources` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `channel_id` BIGINT NULL,
    `inventory_source_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`inventory_source_id`) REFERENCES `inventory_sources`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `core_config` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL,
    `value` LONGTEXT NULL,
    `channel_code` VARCHAR(255) NULL,
    `locale_code` VARCHAR(255) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `subscribers_list` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `email` VARCHAR(255) NULL,
    `is_subscribed` BOOLEAN NOT NULL DEFAULT 1,
    `token` VARCHAR(255) NULL,
    `customer_id` BIGINT NULL,
    `channel_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);
