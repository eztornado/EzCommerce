import React from 'react';
import ReactDOM from 'react-dom/client';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import './styles/bagisto.css';
import App from './App';
import { bagistoTheme } from './theme';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <MantineProvider theme={bagistoTheme} defaultColorScheme="light">
      <App />
    </MantineProvider>
  </React.StrictMode>,
);
