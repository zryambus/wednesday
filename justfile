imagename := "ivolchenkov/wednesday:latest"

build:
	cross build --target x86_64-unknown-linux-gnu --release
	docker build -t {{imagename}} .
	docker push {{imagename}}
