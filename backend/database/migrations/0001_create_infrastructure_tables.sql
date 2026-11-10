-- Infraestructura propia del API (convención Larust): blacklist de JWT.
CREATE TABLE IF NOT EXISTS `jwt_blacklist` (
    `id` BIGINT PRIMARY KEY AUTOINCREMENT,
    `jti` VARCHAR(64) NOT NULL UNIQUE,
    `expires_at` DATETIME NULL,
    `created_at` DATETIME NULL,
    `updated_at` DATETIME NULL
);
