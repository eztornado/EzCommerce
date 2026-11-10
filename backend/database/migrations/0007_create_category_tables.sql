-- Categorías con nested set (_lft/_rgt) y atributos filtrables (Webkul Category).
CREATE TABLE IF NOT EXISTS `categories` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `position` INTEGER NOT NULL DEFAULT 0,
    `image` VARCHAR(255) NULL,
    `category_banner` VARCHAR(255) NULL,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `_lft` INTEGER NOT NULL DEFAULT 0,
    `_rgt` INTEGER NOT NULL DEFAULT 0,
    `parent_id` BIGINT NULL,
    `display_mode` VARCHAR(255) NOT NULL DEFAULT 'products_and_description',
    `additional` JSON NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`parent_id`) REFERENCES `categories`(`id`) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS `categories_lft_rgt_index` ON `categories` (`_lft`, `_rgt`);

CREATE TABLE IF NOT EXISTS `category_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `category_id` BIGINT NULL,
    `locale_id` BIGINT NULL,
    `locale` VARCHAR(255) NULL,
    `name` VARCHAR(255) NULL,
    `slug` VARCHAR(255) NULL,
    `url_path` VARCHAR(2048) NULL,
    `description` LONGTEXT NULL,
    `meta_title` VARCHAR(255) NULL,
    `meta_description` LONGTEXT NULL,
    `meta_keywords` LONGTEXT NULL,
    `alt_image` VARCHAR(255) NULL,
    `alt_banner` VARCHAR(255) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`category_id`) REFERENCES `categories`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `category_filterable_attributes` (
    `category_id` BIGINT NOT NULL,
    `attribute_id` BIGINT NOT NULL,
    PRIMARY KEY (`category_id`, `attribute_id`),
    FOREIGN KEY (`category_id`) REFERENCES `categories`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`attribute_id`) REFERENCES `attributes`(`id`) ON DELETE CASCADE
);
