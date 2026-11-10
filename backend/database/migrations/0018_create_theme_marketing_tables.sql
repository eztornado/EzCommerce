-- Secciones del tema (home por secciones) y marketing/SEO (Webkul Theme/Marketing).
CREATE TABLE IF NOT EXISTS `theme_sections` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NOT NULL,
    `name` VARCHAR(255) NULL,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `channel_id` BIGINT NULL,
    `theme_code` VARCHAR(255) NOT NULL DEFAULT 'default',
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `theme_section_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `theme_section_id` BIGINT NULL,
    `locale` VARCHAR(255) NULL,
    `options` JSON NULL,
    `draft_options` JSON NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`theme_section_id`) REFERENCES `theme_sections`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `marketing_events` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `date` DATE NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `marketing_templates` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `status` BOOLEAN NOT NULL DEFAULT 0,
    `content` LONGTEXT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `marketing_campaigns` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `subject` VARCHAR(255) NULL,
    `status` BOOLEAN NOT NULL DEFAULT 0,
    `spooling` BOOLEAN NOT NULL DEFAULT 0,
    `channel_id` BIGINT NULL,
    `customer_group_id` BIGINT NULL,
    `marketing_template_id` BIGINT NULL,
    `marketing_event_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`customer_group_id`) REFERENCES `customer_groups`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`marketing_template_id`) REFERENCES `marketing_templates`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`marketing_event_id`) REFERENCES `marketing_events`(`id`) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS `url_rewrites` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `entity_type` VARCHAR(255) NULL,
    `request_path` VARCHAR(255) NULL,
    `target_path` VARCHAR(255) NULL,
    `redirect_type` INTEGER NULL,
    `locale` VARCHAR(255) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `search_terms` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `term` VARCHAR(255) NULL,
    `results` INTEGER NOT NULL DEFAULT 0,
    `uses` INTEGER NOT NULL DEFAULT 1,
    `redirect_url` VARCHAR(255) NULL,
    `display_in_suggested_terms` BOOLEAN NOT NULL DEFAULT 0,
    `locale` VARCHAR(255) NULL,
    `channel_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `search_synonyms` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `terms` LONGTEXT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `notifications` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `type` VARCHAR(255) NULL,
    `read` BOOLEAN NOT NULL DEFAULT 0,
    `order_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`order_id`) REFERENCES `orders`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `gdpr_data_request` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `customer_id` BIGINT NULL,
    `email` VARCHAR(255) NULL,
    `status` VARCHAR(30) NULL,
    `type` VARCHAR(30) NULL,
    `message` VARCHAR(500) NULL,
    `revoked_at` DATETIME NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `eu_withdrawals` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `uuid` VARCHAR(36) NOT NULL UNIQUE,
    `order_id` BIGINT NULL UNIQUE,
    `customer_id` BIGINT NULL,
    `channel_id` BIGINT NULL,
    `is_guest` BOOLEAN NOT NULL DEFAULT 0,
    `customer_email` VARCHAR(255) NULL,
    `locale` VARCHAR(10) NULL,
    `reason_text` LONGTEXT NULL,
    `received_at` DATETIME NULL,
    `confirmation_sent_at` DATETIME NULL,
    `final_confirmation_sent_at` DATETIME NULL,
    `confirmation_error` VARCHAR(500) NULL,
    `status` VARCHAR(30) NOT NULL DEFAULT 'received',
    `declined_at` DATETIME NULL,
    `declined_reason` VARCHAR(500) NULL,
    `declined_by_user_id` BIGINT NULL,
    `refunded_at` DATETIME NULL,
    `refunded_by_user_id` BIGINT NULL,
    `refund_note` VARCHAR(500) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`order_id`) REFERENCES `orders`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`),
    FOREIGN KEY (`declined_by_user_id`) REFERENCES `admins`(`id`) ON DELETE SET NULL,
    FOREIGN KEY (`refunded_by_user_id`) REFERENCES `admins`(`id`) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS `eu_withdrawals_customer_id_index` ON `eu_withdrawals` (`customer_id`);
CREATE INDEX IF NOT EXISTS `eu_withdrawals_channel_status_index` ON `eu_withdrawals` (`channel_id`, `status`);
