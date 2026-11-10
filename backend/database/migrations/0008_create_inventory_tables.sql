-- Almacenes / fuentes de inventario (Webkul Inventory).
CREATE TABLE IF NOT EXISTS `inventory_sources` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NOT NULL UNIQUE,
    `name` VARCHAR(255) NULL,
    `description` LONGTEXT NULL,
    `contact_name` VARCHAR(255) NULL,
    `contact_email` VARCHAR(255) NULL,
    `contact_number` VARCHAR(255) NULL,
    `contact_fax` VARCHAR(255) NULL,
    `country` VARCHAR(255) NULL,
    `state` VARCHAR(255) NULL,
    `city` VARCHAR(255) NULL,
    `street` VARCHAR(255) NULL,
    `postcode` VARCHAR(255) NULL,
    `priority` INTEGER NOT NULL DEFAULT 0,
    `latitude` DECIMAL(10,5) NULL,
    `longitude` DECIMAL(10,5) NULL,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);
