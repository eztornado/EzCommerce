-- Clientes, grupos y elementos asociados (Webkul Customer).
CREATE TABLE IF NOT EXISTS `customer_groups` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `name` VARCHAR(255) NOT NULL,
    `is_user_defined` BOOLEAN NOT NULL DEFAULT 1,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `customers` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `first_name` VARCHAR(255) NULL,
    `last_name` VARCHAR(255) NULL,
    `gender` VARCHAR(50) NULL,
    `date_of_birth` DATE NULL,
    `email` VARCHAR(255) NULL UNIQUE,
    `phone` VARCHAR(255) NULL UNIQUE,
    `image` VARCHAR(255) NULL,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `password` VARCHAR(255) NULL,
    `api_token` VARCHAR(80) NULL UNIQUE,
    `customer_group_id` BIGINT NULL,
    `subscribed_to_news_letter` BOOLEAN NOT NULL DEFAULT 0,
    `is_verified` BOOLEAN NOT NULL DEFAULT 0,
    `is_suspended` BOOLEAN NOT NULL DEFAULT 0,
    `token` VARCHAR(255) NULL,
    `customer_id` BIGINT NULL,
    `channel_id` BIGINT NULL,
    `remember_token` VARCHAR(100) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`customer_group_id`) REFERENCES `customer_groups`(`id`) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS `customer_notes` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `customer_id` BIGINT NULL,
    `note` LONGTEXT NULL,
    `customer_notified` BOOLEAN NOT NULL DEFAULT 0,
    `user_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `customer_password_resets` (
    `email` VARCHAR(255) NULL,
    `token` VARCHAR(255) NULL,
    `created_at` DATETIME NULL
);
CREATE INDEX IF NOT EXISTS `customer_password_resets_email_index` ON `customer_password_resets` (`email`);

CREATE TABLE IF NOT EXISTS `customer_social_accounts` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `customer_id` BIGINT NULL,
    `provider_name` VARCHAR(255) NULL,
    `provider_id` VARCHAR(255) NULL UNIQUE,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `wishlist` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `channel_id` BIGINT NULL,
    `product_id` BIGINT NULL,
    `customer_id` BIGINT NULL,
    `item_options` JSON NULL,
    `moved_to_cart` DATE NULL,
    `shared` BOOLEAN NOT NULL DEFAULT 0,
    `time_of_moving` DATE NULL,
    `additional` JSON NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `wishlist_items` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `channel_id` BIGINT NULL,
    `product_id` BIGINT NULL,
    `customer_id` BIGINT NULL,
    `additional` JSON NULL,
    `moved_to_cart` DATE NULL,
    `shared` BOOLEAN NOT NULL DEFAULT 0,
    `time_of_moving` DATE NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `compare_items` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `product_id` BIGINT NULL,
    `customer_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);
