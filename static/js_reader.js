const RESOURCES = {
};

let CHANGES = [];

export const fetch_shader = (path) => {
    console.log("getting shader: " + path);
    return RESOURCES[path];
};

export const get_js_changes = () => {
    console.log("getting changes");

    const res = [...CHANGES];

    CHANGES = [];

    return res;

}

export const push_shader = (path, data) => {

    RESOURCES[path] = data;

    if (!CHANGES.includes(path)) {

        CHANGES.push(path);
    }
};
