import { writable, derived } from 'svelte/store';

export function createMemoizedCallback(callback, dependencies) {
  const deps = writable(dependencies);
  
  return derived(deps, ($deps) => {
    return (...args) => {
      return callback(...args, ...$deps);
    };
  });
}
