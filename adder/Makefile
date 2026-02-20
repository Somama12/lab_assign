test/%.s: test/%.snek src/main.rs
	cargo run -- $< $@

test/%.run: test/%.s runtime/start.rs
	nasm -f macho64 $< -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc --target=x86_64-apple-darwin -L ./runtime runtime/start.rs -o $@
