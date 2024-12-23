# Smpl Payments

[Video Demo](https://youtube.com/)

A Simple Payments server written in Rust using the Axum Web Framework and Diesel ORM. \
The Database used is PostgresDB, which is run in a docker container

## Running
Run the following:
```sh
$ docker compose up -d
$ cargo install diesel
$ diesel migrations run
$ cargo run
```

## Resetting DB
```sh
$ diesel migrations redo --all
```

## Endpoints

The following endpoints can be used with [bruno](https://www.usebruno.com/) in `SmplPaymentsBrunoCollection` folder:

- `POST /sign_up`: Sign up to service
- `POST /sign_in`: Authenticate and get JWT
- `GET /profile`: Get profile
- `PUT /profile`: Update profile
- `GET /wallet`: Get wallet
- `PUT /wallet`: Deposit/Withdraw wallet
- `POST /transactions`: Create transaction
- `GET /transactions`: List transaction
- `GET /transactions/:id`: Get transaction
