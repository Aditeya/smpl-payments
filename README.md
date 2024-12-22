# Smpl Payments

## Running
```sh
docker compose up -d
cargo install diesel
diesel migrations run
cargo run
```

## Endpoints

The following endpoints can be used with [bruno](https://www.usebruno.com/) in `SmplPaymentsBrunoCollection` folder:

- `POST /sign_up`: Sign up to service
- `POST /sign_in`: Authenticate and get JWT
- `GET /profile`: Get profile
- `PUT /profile`: Update profile
- `GET /wallet`: Get wallet
- `PUT /wallet`: Deposit/Withdraw wallet
- `GET /transactions`: List transaction
- `GET /transactions/:id`: Get transaction
- `POST /transactions`: Create transaction
