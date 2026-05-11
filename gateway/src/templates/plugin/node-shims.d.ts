declare module "node:fs" {
  export function readFileSync(path: string, encoding: "utf-8"): string
}

declare module "node:path" {
  export function join(...paths: string[]): string
}
