# EzCommerce

Plataforma de comercio electrónico de código completo: una tienda online
responsive y un panel de administración, construidos con un **backend API en
Rust** (framework Larust) y un **frontend Flutter** que compila a web, Android
e iOS desde la misma base de código.

Diseñado para desplegarse dockerizado (Docker Compose / Coolify) con
**MySQL/MariaDB** en producción y SQLite como base de datos de desarrollo.

## Características

### Tienda (Flutter, responsive web + móvil)

- Página de inicio configurable por secciones (carruseles de imágenes,
  productos y categorías, banners de contenido, tira de servicios)
- Catálogo con búsqueda, filtros por atributos y precio, ordenación y carga
  progresiva
- Ficha de producto con galería, variantes configurables con swatches de
  color, productos agrupados, bundles y descargables
- Carrito para invitados (token de carrito firmado) y clientes registrados,
  cupones y estimación de envío
- Checkout de una página: dirección, método de envío, método de pago,
  revisión y confirmación
- Cuenta de cliente: perfil, pedidos (detalle, cancelación, reordenar),
  direcciones, wishlist, reseñas de productos
- Autenticación completa: registro, login, restablecimiento de contraseña
- Panel de administración accesible desde la misma app

### Panel de administración (Flutter)

- Dashboard con métricas de ventas, pedidos, clientes y stock bajo mínimos
- DataGrid genérico con búsqueda, filtros, ordenación, paginación, acciones
  masivas y exportación CSV
- Productos: creación por tipos (simple, virtual, configurable, agrupado,
  bundle, descargable), atributos EAV, inventario por almacén, imágenes,
  precios especiales por fechas y por grupo de cliente
- Categorías (árbol jerárquico) y atributos con familias y grupos
- Pedidos con ciclo completo: facturas, envíos (multi-almacén), reembolsos y
  cancelación, con estados derivados de las cantidades y descuento de stock
- Clientes: ficha 360º, direcciones, notas y pedidos asociados
- Marketing: reglas de carrito con cupones (porcentaje, importe fijo, compra
  X consigue Y, reparto proporcional) y reglas de catálogo con reindexado de
  precios
- CMS, reglas de SEO (URL rewrites, términos y sinónimos de búsqueda),
  localización (canales, monedas, idiomas), usuarios y roles con ACL
- RMA (devoluciones) y solicitudes GDPR con exportación y anonimización de
  datos

### Backend API (Rust)

- API JSON para tienda y administración con JWT (claims estilo Laravel,
  blacklist de tokens y refresco)
- Motor de carrito con cálculo de totales, impuestos por zona, tarifas de
  envío planificadas y gratuitas, y reserva de inventario
- Precios materializados (`product_flat`, índices de precio e inventario) al
  estilo de Bagisto
- Migraciones SQL con doble dialecto (SQLite/MySQL) y seeders idempotentes
- Salud del servicio en `/up` y almacenamiento de imágenes con servir estático

## Estructura del repositorio

```
backend/    API en Rust (Larust): controladores, motor de carrito/ventas,
            migraciones SQL y seeders
frontend/   App Flutter: tienda (features/shop) y panel de administración
            (features/admin) compartiendo núcleo (core/)
```

## Requisitos

- Rust estable (backend)
- Flutter estable (frontend)
- MariaDB/MySQL 10.4+ en producción; SQLite para desarrollo local

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
cd frontend
flutter pub get
flutter run -d chrome               # o flutter build web
```

La URL del API se configura en `frontend/lib/core/api/api_client.dart`
(`ApiClient.baseUrl`). Las credenciales del administrador y el resto de
parámetros se definen en `backend/.env` (ver `.env.example`).

## Despliegue con Docker Compose

```bash
JWT_SECRET=<secreto> DB_PASSWORD=<contraseña> docker compose up -d
```

Servicios:

- `api` — binario Rust; ejecuta migraciones y seeders antes de servir
  (healthcheck en `/up`, volumen `api_storage` para las imágenes subidas)
- `db` — MariaDB 11 con volumen persistente
- `web` — nginx que sirve el SPA Flutter compilado y hace proxy de `/api` y
  `/storage` hacia el API (mismo origen, sin CORS)

Variables principales (ver `docker-compose.yml`): `JWT_SECRET`, `DB_*`,
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, `SEED_DEMO_DATA`, `FRONTEND_URL`.

## Despliegue en Coolify

1. Crear el proyecto desde el repositorio Git eligiendo **Docker Compose**
   (`docker-compose.yml` en la raíz)
2. Definir las variables obligatorias (`JWT_SECRET`, `DB_PASSWORD`,
   `DB_ROOT_PASSWORD`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`) y las recomendadas
   (`APP_URL`, `FRONTEND_URL` para CORS, `SEED_DEMO_DATA=false`)
3. Asignar dominios: servicio `web` → dominio público de la tienda (proxy de
   `/api` y `/storage` al API, sin CORS); servicio `api` → dominio del API con
   healthcheck `/up`
4. Montar volúmenes persistentes: `db_data` (base de datos) y `api_storage`
   (imágenes de producto)
5. Desplegar: el API migra y siembra automáticamente en el primer arranque

El SPA se construye con `API_BASE_URL=SAME_ORIGIN` (llamadas `/api` relativas
al propio dominio). Si el API se publica en otro dominio, cambiar ese build-arg.

## CI

GitHub Actions (`.github/workflows/ci.yml`) valida en cada push:

- Backend: `cargo check` + migraciones y seeders sobre SQLite
- Frontend: `flutter analyze` + `flutter build web`
- Construcción de las imágenes Docker (API y web)

## API

API JSON para tienda (`/api/...`) y administración (`/api/admin/...`) con
autenticación JWT por cabecera `Authorization: Bearer`. Los errores siguen el
formato Laravel (`{message}` y validación 422 con `{message, errors}`); las
colecciones paginadas usan la forma estándar `{data, links, meta}`.

## Licencia

Uso interno. Consulta con el propietario del repositorio antes de reutilizar.
