-- Páginas CMS (Webkul CMS).
CREATE TABLE IF NOT EXISTS `cms_pages` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `layout` VARCHAR(255) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `cms_page_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `cms_page_id` BIGINT NULL,
    `locale` VARCHAR(255) NULL,
    `page_title` VARCHAR(255) NULL,
    `url_key` VARCHAR(255) NULL,
    `html_content` LONGTEXT NULL,
    `meta_title` VARCHAR(255) NULL,
    `meta_description` LONGTEXT NULL,
    `meta_keywords` LONGTEXT NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`cms_page_id`) REFERENCES `cms_pages`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `cms_page_channels` (
    `cms_page_id` BIGINT NOT NULL,
    `channel_id` BIGINT NOT NULL,
    PRIMARY KEY (`cms_page_id`, `channel_id`),
    FOREIGN KEY (`cms_page_id`) REFERENCES `cms_pages`(`id`) ON DELETE CASCADE,
    FOREIGN KEY (`channel_id`) REFERENCES `channels`(`id`) ON DELETE CASCADE
);
