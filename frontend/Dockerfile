# frontend-build: build frontend with yarn
FROM node:14-alpine as frontend-build
WORKDIR /app
COPY . /app
RUN yarn && yarn run build

