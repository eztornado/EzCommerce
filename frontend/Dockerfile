# Build Flutter web + nginx con proxy /api hacia el backend.
FROM ghcr.io/cirruslabs/flutter:latest AS build
WORKDIR /app
COPY pubspec.yaml ./
RUN flutter pub get
COPY . .
# La URL del API se resuelve en el mismo origen (/api) vía el proxy de nginx;
# el cliente usa ApiClient.baseUrl configurado en tiempo de build.
# SAME_ORIGIN deja baseUrl vacío: el SPA llama a /api en el mismo origen
# (el nginx del contenedor hace proxy hacia el servicio api).
ARG API_BASE_URL=SAME_ORIGIN
RUN if [ "$API_BASE_URL" = "SAME_ORIGIN" ]; then \
      sed -i "s|static const String productionUrl = '.*';|static const String productionUrl = '';|" \
        lib/core/api/api_client.dart; \
    else \
      sed -i "s|static const String productionUrl = '.*';|static const String productionUrl = '$API_BASE_URL';|" \
        lib/core/api/api_client.dart; \
    fi \
    && flutter build web --release

FROM nginx:1.27-alpine AS runtime
COPY --from=build /app/build/web /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
