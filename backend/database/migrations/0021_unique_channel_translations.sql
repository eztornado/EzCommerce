-- Deduplicar `channel_translations` y dar unicidad (channel_id, locale):
-- los seeds repetidos y el upsert de Settings no deben multiplicar filas.
DELETE FROM `channel_translations` WHERE `id` NOT IN (
    SELECT MIN(`id`) FROM `channel_translations` GROUP BY `channel_id`, `locale`
);
CREATE UNIQUE INDEX IF NOT EXISTS `channel_translations_channel_locale_unique`
    ON `channel_translations` (`channel_id`, `locale`);
