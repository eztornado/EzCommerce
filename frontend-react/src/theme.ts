import { createTheme, type MantineColorsTuple } from '@mantine/core';

/**
 * Paleta y tipografía tomadas de Bagisto:
 *  - azul marino de la tienda (#060C3B)
 *  - naranja claro del pie (#F6F2EB)
 *  - Poppins para el cuerpo y DM Serif Display para los títulos.
 */
const navy: MantineColorsTuple = [
  '#eef0f8',
  '#d5d9ea',
  '#aeb6d4',
  '#848fbc',
  '#6673a9',
  '#535f9e',
  '#495699',
  '#3c477f',
  '#303c6a',
  '#060C3B',
];

export const bagistoTheme = createTheme({
  primaryColor: 'navy',
  primaryShade: 9,
  fontFamily: 'Poppins, ui-sans-serif, system-ui, sans-serif',
  headings: {
    fontFamily: 'Poppins, ui-sans-serif, system-ui, sans-serif',
    fontWeight: '600',
  },
  colors: { navy },
  defaultRadius: 'md',
  components: {
    Button: { defaultProps: { radius: 'md' } },
    TextInput: { defaultProps: { radius: 'md' } },
    Select: { defaultProps: { radius: 'md' } },
    NumberInput: { defaultProps: { radius: 'md' } },
    Paper: { defaultProps: { radius: 'md' } },
    Card: { defaultProps: { radius: 'md' } },
    Modal: { defaultProps: { radius: 'md' } },
    Badge: { defaultProps: { radius: 'sm' } },
  },
});

export const NAVY = '#060C3B';
export const SHOP_BG = '#ffffff';
export const LIGHT_ORANGE = '#F6F2EB';
export const ADMIN_BLUE = '#2563eb';
