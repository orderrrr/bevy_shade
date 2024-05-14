const RESOURCES = {
};

let CHANGES = [];

export const fetch_shader = (path) => {
  return RESOURCES[path];
};

export const get_js_changes = () => {
  const res =  [...CHANGES];

  CHANGES = [];

  return res;

}

export const push_shader = (path, data) => {

  RESOURCES[path] = data;

  if (!CHANGES.includes(path)) {

    CHANGES.push(path);
  }
};
