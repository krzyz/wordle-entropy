dev:
	wasm-pack build --target web --no-typescript --dev --out-dir pkg-dev . -- -Z build-std=panic_abort,std
	cp -r static/* pkg-dev/

dist:
	wasm-pack build --target web --no-typescript --release . -- -Z build-std=panic_abort,std
	cp -r static/* pkg/

serve:
	cd pkg-dev/ && python ../serve.py

serve-release:
	cd pkg/ && python ../serve.py

clean:
	rm -rf pkg/

publish:
	rsync -avh pkg/ nixcomplexity:/mnt/data1/single_page/wordle/ --delete
