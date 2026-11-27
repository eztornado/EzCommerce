import { Component, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
  retried: boolean;
}

/**
 * Frontera de error para las rutas lazy (#39): si un chunk no llega (red
 * flaky), sin esto React desmonta el árbol entero y el usuario ve una
 * página en blanco en lugar de un mensaje con reintento.
 */
export default class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, retried: false };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  render() {
    const { error, retried } = this.state;
    if (!error) return this.props.children;

    return (
      <div className="route-error" role="alert">
        <h2>No se pudo cargar esta sección</h2>
        <p>Comprueba tu conexión y vuelve a intentarlo.</p>
        <button
          type="button"
          className="shop-primary-button"
          onClick={() => {
            // El fallo típico es la descarga de un chunk: un reload reintenta
            // la descarga de forma fiable. Solo una vez para no crear bucles.
            if (!retried) {
              this.setState({ retried: true });
              window.location.reload();
              return;
            }
            this.setState({ error: null });
          }}
        >
          Reintentar
        </button>
      </div>
    );
  }
}
