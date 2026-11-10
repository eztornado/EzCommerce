-- Países y estados/provincias (Webkul Core).
CREATE TABLE IF NOT EXISTS `countries` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `code` VARCHAR(255) NULL,
    `name` VARCHAR(255) NULL
);

CREATE TABLE IF NOT EXISTS `country_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `name` VARCHAR(255) NULL,
    `country_id` BIGINT NULL,
    FOREIGN KEY (`country_id`) REFERENCES `countries`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `country_states` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `country_code` VARCHAR(255) NULL,
    `code` VARCHAR(255) NULL,
    `default_name` VARCHAR(255) NULL,
    `country_id` BIGINT NULL,
    FOREIGN KEY (`country_id`) REFERENCES `countries`(`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `country_state_translations` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `locale` VARCHAR(255) NULL,
    `default_name` VARCHAR(255) NULL,
    `country_state_id` BIGINT NULL,
    FOREIGN KEY (`country_state_id`) REFERENCES `country_states`(`id`) ON DELETE CASCADE
);
