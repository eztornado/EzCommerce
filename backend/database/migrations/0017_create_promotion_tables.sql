-- Reglas de carrito y de catálogo + cupones e índices (Webkul CartRule/CatalogRule).
-- action_type: by_percent | by_fixed | cart_fixed | buy_x_get_y (carrito);
--              to_percent | by_percent | to_fixed | by_fixed (catálogo).
-- condition_type: 1 = todas (AND), 2 = cualquiera (OR). coupon_type: 1 sin cupón, 2 con cupón.
CREATE TABLE IF NOT EXISTS `cart_rules` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `starts_from` DATETIME NULL,
    `ends_till` DATETIME NULL,
    `status` BOOLEAN NOT NULL DEFAULT 0,
    `coupon_type` INTEGER NOT NULL DEFAULT 1,
    `use_auto_generation` BOOLEAN NOT NULL DEFAULT 0,
    `usage_per_customer` INTEGER NULL,
    `uses_per_coupon` INTEGER NULL,
    `times_used` INTEGER NOT NULL DEFAULT 0,
    `condition_type` INTEGER NOT NULL DEFAULT 1,
    `conditions` JSON NULL,
    `end_other_rules` BOOLEAN NOT NULL DEFAULT 0,
    `uses_attribute_conditions` BOOLEAN NOT NULL DEFAULT 0,
    `action_type` VARCHAR(50) NULL,
    `discount_amount` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `discount_quantity` INTEGER NOT NULL DEFAULT 1,
    `discount_step` INTEGER NULL,
    `apply_to_shipping` BOOLEAN NOT NULL DEFAULT 0,
    `free_shipping` BOOLEAN NOT NULL DEFAULT 0,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `cart_rule_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `label` VARCHAR(255) NULL,
    `cart_rule_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`cart_rule_id`) REFERENCES `cart_rules`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `cart_rule_channels` (
    `cart_rule_id` BIGINT NOT NULL,
    `channel_id` BIGINT NOT NULL,
    PRIMARY KEY (`cart_rule_id`, `channel_id`),
    FOREIGN KEY (`cart_rule_id`) REFERENCES `cart_rules`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `cart_rule_customer_groups` (
    `cart_rule_id` BIGINT NOT NULL,
    `customer_group_id` BIGINT NOT NULL,
    PRIMARY KEY (`cart_rule_id`, `customer_group_id`),
    FOREIGN KEY (`cart_rule_id`) REFERENCES `cart_rules`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`customer_group_id`) REFERENCES `customer_groups`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `cart_rule_customers` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `times_used` BIGINT NOT NULL DEFAULT 0,
    `cart_rule_id` BIGINT NULL,
    `customer_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`cart_rule_id`) REFERENCES `cart_rules`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `cart_rule_coupons` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `usage_limit` INTEGER NULL,
    `usage_per_customer` INTEGER NULL,
    `times_used` INTEGER NOT NULL DEFAULT 0,
    `type` INTEGER NOT NULL DEFAULT 0,
    `is_primary` BOOLEAN NOT NULL DEFAULT 0,
    `expired_at` DATE NULL,
    `cart_rule_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`cart_rule_id`) REFERENCES `cart_rules`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `cart_rule_coupon_usage` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `times_used` BIGINT NOT NULL DEFAULT 0,
    `cart_rule_coupon_id` BIGINT NULL,
    `customer_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`cart_rule_coupon_id`) REFERENCES `cart_rule_coupons`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `catalog_rules` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `starts_from` DATE NULL,
    `ends_till` DATE NULL,
    `status` BOOLEAN NOT NULL DEFAULT 0,
    `condition_type` INTEGER NOT NULL DEFAULT 1,
    `conditions` JSON NULL,
    `end_other_rules` BOOLEAN NOT NULL DEFAULT 0,
    `action_type` VARCHAR(50) NULL,
    `discount_amount` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `catalog_rule_channels` (
    `catalog_rule_id` BIGINT NOT NULL,
    `channel_id` BIGINT NOT NULL,
    PRIMARY KEY (`catalog_rule_id`, `channel_id`),
    FOREIGN KEY (`catalog_rule_id`) REFERENCES `catalog_rules`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `catalog_rule_customer_groups` (
    `catalog_rule_id` BIGINT NOT NULL,
    `customer_group_id` BIGINT NOT NULL,
    PRIMARY KEY (`catalog_rule_id`, `customer_group_id`),
    FOREIGN KEY (`catalog_rule_id`) REFERENCES `catalog_rules`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`customer_group_id`) REFERENCES `customer_groups`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `catalog_rule_products` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `starts_from` DATE NULL,
    `ends_till` DATE NULL,
    `end_other_rules` BOOLEAN NOT NULL DEFAULT 0,
    `action_type` VARCHAR(50) NULL,
    `discount_amount` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `catalog_rule_id` BIGINT NULL,
    `product_id` BIGINT NULL,
    `channel_id` BIGINT NULL,
    `customer_group_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`catalog_rule_id`) REFERENCES `catalog_rules`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `catalog_rule_product_prices` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `rule_date` DATE NULL,
    `starts_from` DATE NULL,
    `ends_till` DATE NULL,
    `product_id` BIGINT NULL,
    `customer_group_id` BIGINT NULL,
    `catalog_rule_id` BIGINT NULL,
    `channel_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);
