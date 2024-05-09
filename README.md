### TO RUN
`cargo build --release --target wasm32-unknown-unknown`
`wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "bevy_shade" ./target/wasm32-unknown-unknown/release/bevy_shade.wasm`


`cargo build --release --target wasm32-unknown-unknown && wasm-bindgen --no-typescript --target web --out-dir ./dist/ --out-name "bevy_shade" ./target/wasm32-unknown-unknown/release/bevy_shade.wasm`
