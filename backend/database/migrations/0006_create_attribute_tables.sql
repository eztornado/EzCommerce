-- Atributos del catálogo: familias, grupos y EAV (Webkul Attribute).
CREATE TABLE IF NOT EXISTS `attribute_families` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `name` VARCHAR(255) NOT NULL,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `is_user_defined` BOOLEAN NOT NULL DEFAULT 1,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `attributes` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `admin_name` VARCHAR(255) NULL,
    `type` VARCHAR(255) NULL,
    `swatch_type` VARCHAR(255) NULL,
    `validation` VARCHAR(255) NULL,
    `position` INTEGER NOT NULL DEFAULT 0,
    `is_required` BOOLEAN NOT NULL DEFAULT 0,
    `is_unique` BOOLEAN NOT NULL DEFAULT 0,
    `is_filterable` BOOLEAN NOT NULL DEFAULT 0,
    `is_comparable` BOOLEAN NOT NULL DEFAULT 0,
    `is_configurable` BOOLEAN NOT NULL DEFAULT 0,
    `is_user_defined` BOOLEAN NOT NULL DEFAULT 1,
    `is_visible_on_front` BOOLEAN NOT NULL DEFAULT 0,
    `value_per_locale` BOOLEAN NOT NULL DEFAULT 0,
    `value_per_channel` BOOLEAN NOT NULL DEFAULT 0,
    `enable_wysiwyg` BOOLEAN NOT NULL DEFAULT 0,
    `default_value` LONGTEXT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `attribute_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `name` VARCHAR(255) NULL,
    `attribute_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`attribute_id`) REFERENCES `attributes`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `attribute_groups` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `attribute_family_id` BIGINT NULL,
    `name` VARCHAR(255) NULL,
    `position` INTEGER NOT NULL DEFAULT 0,
    `is_user_defined` BOOLEAN NOT NULL DEFAULT 1,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`attribute_family_id`) REFERENCES `attribute_families`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `attribute_group_mappings` (
    `attribute_id` BIGINT NOT NULL,
    `attribute_group_id` BIGINT NOT NULL,
    `position` INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (`attribute_id`, `attribute_group_id`),
    FOREIGN KEY (`attribute_id`) REFERENCES `attributes`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`attribute_group_id`) REFERENCES `attribute_groups`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `attribute_options` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `attribute_id` BIGINT NULL,
    `admin_name` VARCHAR(255) NULL,
    `sort_order` INTEGER NOT NULL DEFAULT 0,
    `swatch_value` LONGTEXT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`attribute_id`) REFERENCES `attributes`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `attribute_option_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `label` VARCHAR(255) NULL,
    `attribute_option_id` BIGINT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`attribute_option_id`) REFERENCES `attribute_options`(`id`) ON DELETE CASCADE
);
