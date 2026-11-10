# Build Flutter web + nginx con proxy /api hacia el backend.
FROM ghcr.io/cirruslabs/flutter:latest AS build
WORKDIR /app
COPY pubspec.yaml ./
RUN flutter pub get
COPY . .
# La URL del API se resuelve en el mismo origen (/api) vía el proxy de nginx;
# el cliente usa ApiClient.baseUrl configurado en tiempo de build.
ARG API_BASE_URL=http://localhost:34187
RUN sed -i "s|static const String productionUrl = '.*';|static const String productionUrl = '$API_BASE_URL';|" \
    lib/core/api/api_client.dart \
    && flutter build web --release

FROM nginx:1.27-alpine AS runtime
COPY --from=build /app/build/web /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
