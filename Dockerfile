# syntax=docker/dockerfile:1
# run in the root of project next commands:
#
# docker build -t nnn-rust-image .
# docker run -d --name <container_name> -p 8080:8080 \
#     -e PG_DATABASE_URL=postgres://[USER]:[PASS]@[HOST]:[PORT]/[DB_NAME] \
#     -e REDIS_DATABASE_URI=redis://[HOST]:[PORT] \
#     -e ADDRESS=0.0.0.0:8080 (Port same as in -p variable)
#     nnn-rust-image

FROM rust:latest as build

ARG APP_NAME=nnn-rust-back

WORKDIR /app
COPY . .

RUN cargo build --locked --release
RUN cp ./target/release/$APP_NAME /bin/server

FROM ubuntu:latest AS final

RUN apt update && apt install -y libpq5

COPY --from=build /bin/server /bin/

EXPOSE 8080

CMD ["/bin/server"]