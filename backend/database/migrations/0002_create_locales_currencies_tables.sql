-- Locales y monedas (Webkul Core).
CREATE TABLE IF NOT EXISTS `locales` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `name` VARCHAR(255) NULL,
    `direction` VARCHAR(255) NOT NULL DEFAULT 'ltr',
    `logo_path` VARCHAR(255) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `currencies` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NULL,
    `name` VARCHAR(255) NULL,
    `symbol` VARCHAR(255) NULL,
    `decimal` INTEGER NOT NULL DEFAULT 2,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `currency_exchange_rates` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `rate` DECIMAL(24,12) NOT NULL DEFAULT 1,
    `target_currency` BIGINT NULL UNIQUE,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL,
    FOREIGN KEY (`target_currency`) REFERENCES `currencies`(`id`) ON DELETE CASCADE
);
