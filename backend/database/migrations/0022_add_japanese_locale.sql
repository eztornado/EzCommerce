-- Idioma japonés para la interfaz y corrección de la dirección del español.
update `locales` SET `direction` = 'ltr' WHERE `code` = 'es';

INSERT INTO `locales` (`code`, `name`, `direction`, `created_at`, `updated_at`)
SELECT 'ja', 'Japanese', 'ltr', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM `locales` WHERE `code` = 'ja');

INSERT INTO `channel_locales` (`channel_id`, `locale_id`)
SELECT 1, `id` FROM `locales`
WHERE `code` = 'ja'
  AND NOT EXISTS (
    SELECT 1 FROM `channel_locales` WHERE `channel_id` = 1 AND `locale_id` = `locales`.`id`
  );
