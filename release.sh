cargo +nightly run --release -- \
	--prefix ~/cmdemo/ \
	--je-port 25565 \
	--be-port 9999 \
	--bind-addr 127.0.0.1
