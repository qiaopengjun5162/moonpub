import esbuild from "esbuild";

const prod = process.argv.includes("production");

await esbuild.build({
  entryPoints: ["main.ts"],
  bundle: true,
  platform: "node",
  target: "ES2022",
  format: "cjs",
  outfile: "main.js",
  external: ["obsidian", "electron"],
  sourcemap: prod ? false : "inline",
  minify: prod,
  logLevel: "info",
});
