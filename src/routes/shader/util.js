import * as brotliPromise from "brotli-wasm";

function bytesToBase64(bytes) {
    const binString = Array.from(bytes, (byte) =>
        String.fromCodePoint(byte),
    ).join("");
    return btoa(binString);
}

function base64ToBytes(base64) {
    const binString = atob(base64);
    return Uint8Array.from(binString, (m) => m.codePointAt(0));
}

export async function decompress(encoded) {
    //replace %2 with /
    encoded = encoded.replaceAll("%2F", "/");
    console.log(encoded);
    const textEncoder = new TextDecoder();
    const wasm = await brotliPromise.default;
    const compressed = base64ToBytes(encoded);

    const compressedBytes = wasm.decompress(compressed);
    // arraybuffer to string
    console.log("compressed", compressedBytes);
    const decompressed = textEncoder.decode(compressedBytes);
    console.log("decompressed string", decompressed);
    return decompressed;
}

export async function compress(value) {
    const textEncoder = new TextEncoder();

    const wasm = await brotliPromise.default;

    const uncompressedData = textEncoder.encode(value);

    const compressed = wasm.compress(uncompressedData);
    const encoded = bytesToBase64(compressed);
    console.log(encoded);
    return encoded;
}
