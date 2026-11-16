# EzCommerce

Plataforma de comercio electrónico de código completo: una tienda online
responsive y un panel de administración, construidos con un **backend API en
Rust** (framework Larust) y un **frontend web en React con Mantine**.

Diseñado para desplegarse dockerizado (Docker Compose / Coolify) con
**MySQL/MariaDB** en producción y SQLite como base de datos de desarrollo.

## Características

### Tienda (React, responsive web)

- Página de inicio configurable por secciones (carruseles de imágenes,
  productos y categorías, banners de contenido)
- Catálogo con búsqueda, filtros por atributos y precio, ordenación
- Ficha de producto con galería, variantes configurables con swatches de
  color y productos agrupados
- Carrito para invitados (token de carrito firmado) y clientes registrados,
  cupones y estimación de envío
- Checkout de una página: dirección, método de envío, método de pago,
  revisión y confirmación
- Cuenta de cliente: perfil, pedidos (detalle, cancelación, reordenar),
  direcciones, wishlist, reseñas de productos
- Autenticación completa: registro, login, restablecimiento de contraseña
- Selector de idioma y moneda del canal en la cabecera

### Panel de administración (React)

- Dashboard con métricas de ventas, pedidos, clientes y gráfica de los
  últimos 30 días
- DataGrid genérico con búsqueda, filtros, ordenación, paginación, acciones
  masivas y exportación CSV
- Productos: creación por tipos (simple, virtual, configurable, agrupado,
  bundle, descargable), editor por pestañas con precios especiales por
  ventana de fechas, linked products, variantes, inventario por almacén,
  imágenes y copia de producto
- Categorías (árbol jerárquico), atributos con familias y grupos
- Pedidos con ciclo completo: facturas, envíos (multi-almacén), reembolsos y
  cancelación, con estados derivados de las cantidades y descuento de stock
- Clientes: ficha con pedidos, direcciones, reseñas y wishlist; grupos de
  clientes y moderación de reseñas
- Marketing: reglas de carrito con cupones (porcentaje, importe fijo, compra
  X consigue Y, reparto proporcional) y reglas de catálogo con reindexado de
  precios
- Communications: plantillas de email, eventos y campañas con envío a los
  suscriptores
- Search SEO: URL rewrites, términos y sinónimos de búsqueda
- CMS, localización (canales, monedas, idiomas), usuarios y roles con ACL
- Reporting por rangos de fecha con export CSV

### Backend API (Rust)

- API JSON para tienda y administración con JWT (claims estilo Laravel,
  blacklist de tokens y refresco)
- Motor de carrito con cálculo de totales, impuestos por zona, tarifas de
  envío planificadas y gratuitas, y reserva de inventario
- Precios materializados (`product_flat`, índices de precio e inventario) al
  estilo de Bagisto
- Localización por idioma y conversión de moneda con tasas de cambio
- Reservas (bookings) por slots para productos de tipo booking
- Migraciones SQL con doble dialecto (SQLite/MySQL) y seeders idempotentes
- Salud del servicio en `/up` y almacenamiento de imágenes con servir estático

## Estructura del repositorio

```
backend/         API en Rust (Larust): controladores, motor de carrito/ventas,
                 migraciones SQL y seeders
frontend-react/  App web React (Mantine): tienda y panel de administración
```

## Requisitos

- Rust estable (backend)
- Node 20+ (frontend)
- MariaDB/MySQL 10.4+ en producción; SQLite para desarrollo local

## Datos de ejemplo

Con `SEED_DEMO_DATA=true` el seeder genera un paquete completo para probar
la plataforma: 11 productos de catálogo (incluido un configurable con tres
variantes de color), secciones de home, cupones (10% automático y
`WELCOME10`), regla de catálogo, clientes, pedidos con facturas y envíos
(estados derivados reales: pendiente, en proceso, completado), reseñas
aprobadas y wishlist. El seeder es idempotente: re-ejecutarlo no duplica
datos.

Credenciales de los datos demo:

| Rol | Identificador | Contraseña |
|---|---|---|
| Administrador | valor de `ADMIN_EMAIL` (por defecto `admin@example.com`) | valor de `ADMIN_PASSWORD` (por defecto `admin123`) |
| Cliente | `customer@example.com` | `customer123` |
| Cliente 2 | `marc@example.com` | `customer123` |
| Cliente 3 | `laura@example.com` | `customer123` |

## Puesta en marcha (desarrollo)

Backend:

```bash
cd backend
cp .env.example .env
cargo run -- migrate:fresh --seed   # esquema completo + datos iniciales
cargo run                           # API en http://127.0.0.1:34187
```

Frontend:

```bash
cd frontend-react
npm install
npm run dev   # Vite con proxy /api -> http://127.0.0.1:34187
```

La URL del API se configura con `VITE_API_BASE` en `frontend-react` (vacía
por defecto: rutas `/api` relativas al propio dominio, con proxy de Vite en
desarrollo). Las credenciales del administrador y el resto de parámetros se
definen en `backend/.env` (ver `.env.example`).

## Despliegue con Docker Compose

```bash
JWT_SECRET=<secreto> DB_PASSWORD=<contraseña> docker compose up -d
```

Servicios:

- `api` — binario Rust; ejecuta migraciones y seeders antes de servir
  (healthcheck en `/up`, volumen `api_storage` para las imágenes subidas)
- `db` — MariaDB 11 con volumen persistente
- `web` — nginx que sirve el SPA React compilado y hace proxy de `/api` y
  `/storage` hacia el API (mismo origen, sin CORS)

Variables principales (ver `docker-compose.yml`): `JWT_SECRET`, `DB_*`,
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, `SEED_DEMO_DATA`, `FRONTEND_URL`.

## Despliegue en Coolify

1. Crear el proyecto desde el repositorio Git eligiendo **Docker Compose**
   (`docker-compose.yml` en la raíz)
2. Definir las variables obligatorias (`JWT_SECRET`, `DB_PASSWORD`,
   `DB_ROOT_PASSWORD`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`) y las recomendadas
   (`APP_URL`, `FRONTEND_URL` para CORS, `SEED_DEMO_DATA=false`)
3. Asignar dominios en Coolify: servicio `web` → dominio público de la tienda
   (el nginx del servicio hace proxy de `/api` y `/storage` hacia el API, sin
   CORS); el servicio `api` no necesita dominio público (healthcheck `/up`)
4. Montar volúmenes persistentes: `db_data` (base de datos) y `api_storage`
   (imágenes de producto)
5. Desplegar: el API migra y siembra automáticamente en el primer arranque

Los servicios usan `expose` en lugar de `ports` publicados: Coolify gestiona
el acceso mediante sus dominios y proxy. Si se quiere acceso directo por
puerto (por ejemplo para pruebas sin dominio), añadir en la UI del servicio el
mapeo deseado o sustituir `expose` por `ports` en el compose
(`"8080:80"` para la tienda, `"34187:34187"` para el API).

El SPA se construye con `API_BASE_URL=SAME_ORIGIN` (llamadas `/api` relativas
al propio dominio). Si el API se publica en otro dominio, cambiar ese build-arg.

## CI

GitHub Actions (`.github/workflows/ci.yml`) valida en cada push:

- Backend: `cargo check` + migraciones y seeders sobre SQLite
- Frontend: `npm install` + `npm run build` (TypeScript + Vite)
- Construcción de las imágenes Docker (API y web)

## API

API JSON para tienda (`/api/...`) y administración (`/api/admin/...`) con
autenticación JWT por cabecera `Authorization: Bearer`. Los errores siguen el
formato Laravel (`{message}` y validación 422 con `{message, errors}`); las
colecciones paginadas usan la forma estándar `{data, links, meta}`.

## Licencia

Uso interno. Consulta con el propietario del repositorio antes de reutilizar.
