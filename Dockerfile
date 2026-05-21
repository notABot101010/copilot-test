FROM node:22-alpine AS frontend-build
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM golang:1.25-alpine AS backend-build
WORKDIR /app/backend
RUN apk add --no-cache git
COPY backend/go.mod backend/go.sum ./
RUN go mod download
COPY backend/ ./
RUN go build -o /out/server ./cmd/server

FROM alpine:3.22
RUN apk add --no-cache git ca-certificates
WORKDIR /app
COPY --from=backend-build /out/server /app/server
COPY --from=frontend-build /app/frontend/dist /app/frontend/dist
RUN mkdir -p /app/data/repos
ENV HTTP_PORT=8080
ENV SSH_PORT=2222
ENV REPOS_ROOT=/app/data/repos
ENV STATIC_ROOT=/app/frontend/dist
EXPOSE 8080 2222
CMD ["/app/server"]
