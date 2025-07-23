## Formatting for .js files
Please install the VSCode prettier extension and set as default formatter for js 

## Javascript & Typescript interop

Besides from compiling Rust to WebAssembly, we sometimes have the need to call some external
Javascript function. This is not bad per se, but it has a problem: Javascript is a
dynamic, weakly typed language, any type guarantees we get from Rust are erased once we start 
calling JS code. To mitigate that, we've started to incorporate Typescript where we once used 
Javascript. The setup consists of the following:
1. .ts files under the `web_store/ts` that get compiled to .js files under `web_store/js`.
   This is because to use extern functions, we still need to import raw .js files.
2. We use the [tsync](https://github.com/Wulf/tsync) utility to export Rust structs
   as Typescript types which we can then use and keep in-sync. You can
   see an example of this with the `AccountRecord` struct, which then gets
   exported as the Typescript interface with the same name under `web_store/ts/types.ts`

To unify and make this setup straightforward, the top-most makefile from this project has
some useful make targets, mainly:
1. `make rust-client-ts-build`, which takes the .ts files and compiles them down to .js files.
2. `make rust-client-type-gen`, will generate the types.ts file from the Rust structs
   under web_store, keep in mind, for a struct to be exported it needs to use the `#[tsync]`
   macro.

For these targets to work, you will have to first run `install-tools`, which will install the
`tsync` tool and the TypeScript compiler. Also, the 2 targets mentioned above run before
`build-wasm`, so that should help us keep the Typescript ⇔  Rust types in sync.
