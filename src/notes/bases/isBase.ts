export const BASE_EXTENSION = /\.base$/i;

export function isBase(path: string): boolean {
  return BASE_EXTENSION.test(path);
}
