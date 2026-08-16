export const env = {
  print_i32: (val) => console.log(val),
  print_f32: (val) => console.log(val),
  print_bool: (val) => console.log(val !== 0),
  alert: (val) => alert(val)
};

export async function run(bytes) {
  const { instance } = await WebAssembly.instantiate(bytes, { env });
  return instance.exports.main?.();
}

const bytes = await fetch(new URL('./out.wasm', import.meta.url)).then((r) => r.arrayBuffer())
await run(bytes)