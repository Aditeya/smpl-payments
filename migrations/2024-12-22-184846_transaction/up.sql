-- Your SQL goes here
CREATE TABLE transaction(
    id SERIAL PRIMARY KEY,
	from_wallet INT NOT NULL,
	to_wallet INT NOT NULL,
	amount DECIMAL(10, 2) NOT NULL CHECK (amount >= 0),
	created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (from_wallet) REFERENCES wallet(id),
    FOREIGN KEY (to_wallet) REFERENCES wallet(id)
);
