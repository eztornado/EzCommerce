-- Administradores y roles con ACL (Webkul User).
CREATE TABLE IF NOT EXISTS `roles` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NULL,
    `description` VARCHAR(255) NULL,
    `permission_type` VARCHAR(255) NOT NULL DEFAULT 'all',
    `permissions` JSON NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `admins` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL UNIQUE,
    `password` VARCHAR(255) NOT NULL,
    `api_token` VARCHAR(80) NULL UNIQUE,
    `status` BOOLEAN NOT NULL DEFAULT 1,
    `role_id` BIGINT NULL,
    `image` VARCHAR(255) NULL,
    `two_factor_secret` LONGTEXT NULL,
    `two_factor_enabled` BOOLEAN NOT NULL DEFAULT 0,
    `two_factor_backup_codes` JSON NULL,
    `two_factor_verified_at` DATETIME NULL,
    `remember_token` VARCHAR(100) NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);

CREATE TABLE IF NOT EXISTS `admin_password_resets` (
    `email` VARCHAR(255) NULL,
    `token` VARCHAR(255) NULL,
    `created_at` DATETIME NULL
);
CREATE INDEX IF NOT EXISTS `admin_password_resets_email_index` ON `admin_password_resets` (`email`);
