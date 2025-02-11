run:
	@echo "did you really mean do build this?"
go:
	cargo run --example gen --release
plugintest:
	cargo run --example plugintest --release

mac:
	cargo build
mac.release:
	cargo build --release

android:
	cargo ndk -t arm64-v8a build
android.release:
	cargo ndk -t arm64-v8a build --release

clean:
	rm -rf ./output && \
		rm -rf ./examples/plugintest/include && \
		rm -rf ./examples/plugintest/rs && \
		rm -rf ./examples/plugintest/src
