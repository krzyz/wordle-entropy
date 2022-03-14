dist:
	wasm-pack build --target web --no-typescript
	cp static/* pkg/

serve:
	cd pkg/ && python ../serve.py

clean:
	rm -rf pkg/