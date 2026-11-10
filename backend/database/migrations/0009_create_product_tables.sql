-- Productos: EAV, product_flat, media, relaciones, inventarios e índices (Webkul Product).
CREATE TABLE IF NOT EXISTS `products` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `sku` VARCHAR(255) NULL UNIQUE,
    `type` VARCHAR(255) NOT NULL DEFAULT 'simple',
    `parent_id` BIGINT NULL,
    `attribute_family_id` BIGINT NULL,
    `additional` JSON NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`parent_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`attribute_family_id`) REFERENCES `attribute_families`(`id`)
);

CREATE TABLE IF NOT EXISTS `product_attribute_values` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `channel` VARCHAR(255) NULL,
    `text_value` LONGTEXT NULL,
    `boolean_value` BOOLEAN NULL,
    `integer_value` INTEGER NULL,
    `float_value` DECIMAL(12,4) NULL,
    `datetime_value` DATETIME NULL,
    `date_value` DATE NULL,
    `json_value` JSON NULL,
    `product_id` BIGINT NULL,
    `attribute_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`attribute_id`) REFERENCES `attributes`(`id`) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS `pav_product_attribute_index` ON `product_attribute_values` (`product_id`, `attribute_id`);

CREATE TABLE IF NOT EXISTS `product_flat` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `sku` VARCHAR(255) NULL,
    `type` VARCHAR(255) NULL,
    `product_number` VARCHAR(255) NULL,
    `name` LONGTEXT NULL,
    `short_description` LONGTEXT NULL,
    `description` LONGTEXT NULL,
    `url_key` VARCHAR(255) NULL,
    `new` BOOLEAN NOT NULL DEFAULT 1,
    `featured` BOOLEAN NOT NULL DEFAULT 0,
    `status` BOOLEAN NOT NULL DEFAULT 0,
    `visible_individually` BOOLEAN NOT NULL DEFAULT 1,
    `meta_title` VARCHAR(255) NULL,
    `meta_keywords` LONGTEXT NULL,
    `meta_description` LONGTEXT NULL,
    `price` DECIMAL(12,4) NULL,
    `special_price` DECIMAL(12,4) NULL,
    `special_price_from` DATE NULL,
    `special_price_to` DATE NULL,
    `weight` DECIMAL(12,4) NULL,
    `quantity` INTEGER NOT NULL DEFAULT 0,
    `images_count` INTEGER NOT NULL DEFAULT 0,
    `manage_stock` BOOLEAN NOT NULL DEFAULT 1,
    `base_image` LONGTEXT NULL,
    `category_name` LONGTEXT NULL,
    `attribute_family_name` VARCHAR(255) NULL,
    `locale` VARCHAR(255) NULL,
    `channel` VARCHAR(255) NULL,
    `attribute_family_id` BIGINT NULL,
    `product_id` BIGINT NULL,
    `parent_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`parent_id`) REFERENCES `product_flat`(`id`) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS `product_flat_product_id_index` ON `product_flat` (`product_id`);

CREATE TABLE IF NOT EXISTS `product_categories` (
    `product_id` BIGINT NOT NULL,
    `category_id` BIGINT NOT NULL,
    PRIMARY KEY (`product_id`, `category_id`),
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`category_id`) REFERENCES `categories`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_channels` (
    `product_id` BIGINT NOT NULL,
    `channel_id` BIGINT NOT NULL,
    PRIMARY KEY (`product_id`, `channel_id`),
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_inventories` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `qty` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `vendor_id` BIGINT NOT NULL DEFAULT 0,
    `inventory_source_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`inventory_source_id`) REFERENCES `inventory_sources`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_ordered_inventories` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `qty` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `channel_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_inventory_indices` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `qty` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `channel_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_price_indices` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `product_id` BIGINT NULL,
    `customer_group_id` BIGINT NULL,
    `min_price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `regular_min_price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `max_price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `regular_max_price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`customer_group_id`) REFERENCES `customer_groups`(`id`) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS `ppi_product_customer_group_index` ON `product_price_indices` (`product_id`, `customer_group_id`);

CREATE TABLE IF NOT EXISTS `product_customer_group_prices` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `qty` INTEGER NULL,
    `value_type` VARCHAR(255) NULL,
    `value` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `customer_group_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`customer_group_id`) REFERENCES `customer_groups`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_images` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NULL,
    `path` LONGTEXT NULL,
    `position` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_image_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `alt_text` VARCHAR(255) NULL,
    `product_image_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_image_id`) REFERENCES `product_images`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_videos` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NULL,
    `path` LONGTEXT NULL,
    `position` INTEGER NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_reviews` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `title` VARCHAR(255) NULL,
    `rating` INTEGER NOT NULL DEFAULT 0,
    `comment` LONGTEXT NULL,
    `status` VARCHAR(30) NOT NULL DEFAULT 'pending',
    `product_id` BIGINT NULL,
    `customer_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_review_attachments` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NULL,
    `mime_type` VARCHAR(255) NULL,
    `path` VARCHAR(255) NULL,
    `product_review_id` BIGINT NULL,
    `position` INTEGER NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_review_id`) REFERENCES `product_reviews`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_relations` (
    `parent_id` BIGINT NOT NULL,
    `child_id` BIGINT NOT NULL,
    PRIMARY KEY (`parent_id`, `child_id`),
    FOREIGN KEY (`parent_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`child_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_cross_sells` (
    `parent_id` BIGINT NOT NULL,
    `child_id` BIGINT NOT NULL,
    PRIMARY KEY (`parent_id`, `child_id`),
    FOREIGN KEY (`parent_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`child_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_up_sells` (
    `parent_id` BIGINT NOT NULL,
    `child_id` BIGINT NOT NULL,
    PRIMARY KEY (`parent_id`, `child_id`),
    FOREIGN KEY (`parent_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`child_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_super_attributes` (
    `product_id` BIGINT NOT NULL,
    `attribute_id` BIGINT NOT NULL,
    PRIMARY KEY (`product_id`, `attribute_id`),
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`attribute_id`) REFERENCES `attributes`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `product_grouped_products` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `qty` INTEGER NULL,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `associated_product_id` BIGINT NULL,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`associated_product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);
