import { Fragment, useEffect, useRef } from 'react';
import { useShop } from '../store';
import { setLocale, useLocale } from './index';

/**
 * Aplica el idioma configurado en el backend (`ui_locale`) a la interfaz.
 * Solo reacciona cuando cambia el valor configurado, para que un cambio de
 * moneda o de locale de contenido no revierta la elección del visitante.
 *
 * El árbol se remonta al cambiar de idioma (`key`) para que los componentes ya
 * montados vuelvan a leer `t()` y traduzcan también los textos dinámicos
 * (menús, secciones de la portada, configuración...).
 */
export default function I18nBridge({ children }: { children: React.ReactNode }) {
  const context = useShop((state) => state.context);
  const locale = useLocale();
  const applied = useRef<string | null>(null);
  const configured = context?.ui_locale ?? context?.channel?.default_locale_code ?? null;

  useEffect(() => {
    if (!configured || applied.current === configured) return;
    applied.current = configured;
    setLocale(configured);
  }, [configured]);

  return <Fragment key={locale}>{children}</Fragment>;
}
