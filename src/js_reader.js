const shader_map = {
  "fragment": "HI THERE 2",
}

export const fetch_shader = (path) => {
  return "HI THERE";
}

export const set_shader = (path, res) => {
  shader_map[path] = res;
}