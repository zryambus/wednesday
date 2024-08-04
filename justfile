set dotenv-load := true

imagename := "ivolchenkov/wednesday:latest"

prepare:
	cargo sqlx prepare

build:
	echo SQLX_OFFLINE=1 >> .env
	cross build --target x86_64-unknown-linux-gnu --release
	sed -i '/SQLX_OFFLINE=1/d' .env
	docker build -t {{imagename}} .
	docker push {{imagename}}
