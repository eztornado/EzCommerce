# SPA React Mantine: build con Node, serve con nginx (mismo patrón que el
# anterior). API_BASE_URL=SAME_ORIGIN deja las llamadas /api relativas al
# propio dominio (nginx hace proxy al servicio api).
FROM node:20-alpine AS build
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm install
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx/default.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
