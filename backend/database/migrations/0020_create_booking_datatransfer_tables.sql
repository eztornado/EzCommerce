-- Productos de reserva (Webkul BookingProduct) e import/export (Webkul DataTransfer).
CREATE TABLE IF NOT EXISTS `booking_products` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NULL,
    `qty` INTEGER NULL,
    `location` LONGTEXT NULL,
    `show_location` BOOLEAN NOT NULL DEFAULT 0,
    `available_every_week` BOOLEAN NOT NULL DEFAULT 0,
    `available_from` DATETIME NULL,
    `available_to` DATETIME NULL,
    `product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `bookings` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `qty` INTEGER NOT NULL DEFAULT 1,
    `from` BIGINT NULL,
    `to` BIGINT NULL,
    `allow_cancellation` BOOLEAN NOT NULL DEFAULT 0,
    `product_id` BIGINT NULL,
    `order_item_id` BIGINT NULL,
    `order_id` BIGINT NULL,
    `booking_product_event_ticket_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`product_id`) REFERENCES `products`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`order_item_id`) REFERENCES `order_items`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`order_id`) REFERENCES `orders`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `booking_product_default_slots` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `booking_type` VARCHAR(255) NULL,
    `duration` DECIMAL(10,2) NULL,
    `break_time` DECIMAL(10,2) NULL,
    `slots` JSON NULL,
    `booking_product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`booking_product_id`) REFERENCES `booking_products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `booking_product_appointment_slots` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `duration` DECIMAL(10,2) NULL,
    `break_time` DECIMAL(10,2) NULL,
    `same_slot_all_days` BOOLEAN NOT NULL DEFAULT 0,
    `slots` JSON NULL,
    `booking_product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`booking_product_id`) REFERENCES `booking_products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `booking_product_event_tickets` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `price` DECIMAL(12,4) NOT NULL DEFAULT 0,
    `special_price` DECIMAL(12,4) NULL,
    `qty` INTEGER NULL,
    `special_price_from` DATETIME NULL,
    `special_price_to` DATETIME NULL,
    `booking_product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`booking_product_id`) REFERENCES `booking_products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `booking_product_event_ticket_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `locale` VARCHAR(255) NULL,
    `booking_product_event_ticket_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`booking_product_event_ticket_id`) REFERENCES `booking_product_event_tickets`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `booking_product_rental_slots` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `renting_type` VARCHAR(255) NULL,
    `daily_price` DECIMAL(12,4) NULL,
    `hourly_price` DECIMAL(12,4) NULL,
    `same_slot_all_days` BOOLEAN NOT NULL DEFAULT 0,
    `slots` JSON NULL,
    `booking_product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`booking_product_id`) REFERENCES `booking_products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `booking_product_table_slots` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `price_type` VARCHAR(255) NULL,
    `guest_limit` INTEGER NULL,
    `duration` DECIMAL(10,2) NULL,
    `break_time` DECIMAL(10,2) NULL,
    `prevent_scheduling_before` DECIMAL(10,2) NULL,
    `same_slot_all_days` BOOLEAN NOT NULL DEFAULT 0,
    `slots` JSON NULL,
    `booking_product_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`booking_product_id`) REFERENCES `booking_products`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `imports` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `state` VARCHAR(30) NOT NULL DEFAULT 'pending',
    `process_in_queue` BOOLEAN NOT NULL DEFAULT 0,
    `type` VARCHAR(255) NULL,
    `action` VARCHAR(255) NULL,
    `validation_strategy` VARCHAR(255) NULL,
    `allowed_errors` INTEGER NULL,
    `processed_rows_count` INTEGER NULL,
    `invalid_rows_count` INTEGER NULL,
    `errors_count` INTEGER NULL,
    `errors` JSON NULL,
    `field_separator` VARCHAR(10) NULL,
    `file_path` VARCHAR(255) NULL,
    `images_directory_path` VARCHAR(255) NULL,
    `error_file_path` VARCHAR(255) NULL,
    `summary` JSON NULL,
    `started_at` DATETIME NULL,
    `completed_at` DATETIME NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `import_batches` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `state` VARCHAR(30) NOT NULL DEFAULT 'pending',
    `data` JSON NULL,
    `summary` JSON NULL,
    `import_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`import_id`) REFERENCES `imports`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `sitemaps` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `file_name` VARCHAR(255) NULL,
    `path` VARCHAR(255) NULL,
    `generated_at` DATETIME NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `sitemap_channels` (
    `sitemap_id` BIGINT NOT NULL,
    `channel_id` BIGINT NOT NULL,
    PRIMARY KEY (`sitemap_id`, `channel_id`),
    FOREIGN KEY (`sitemap_id`) REFERENCES `sitemaps`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `datagrid_saved_filters` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `src` VARCHAR(255) NULL,
    `applied` JSON NULL,
    `user_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`user_id`) REFERENCES `admins`(`id`) ON DELETE CASCADE
);
