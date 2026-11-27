/** Skeleton de carga de ruta (#39): sustituye al div vacío que causaba el
 *  flash blanco mientras baja el chunk de una página lazy. */
export default function RouteSkeleton() {
  return (
    <div className="route-skeleton" data-suspense="true" aria-busy="true">
      <div className="route-skeleton-hero" />
      <div className="route-skeleton-line" />
      <div className="route-skeleton-line short" />
      <div className="route-skeleton-grid">
        <div />
        <div />
        <div />
      </div>
    </div>
  );
}
