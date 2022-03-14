dist:
	wasm-pack build --target web --no-typescript --release . -- -Z build-std=panic_abort,std
	cp -r static/* pkg/

serve:
	cd pkg/ && python ../serve.py

clean:
	rm -rf pkg/