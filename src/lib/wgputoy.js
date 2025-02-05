import { create_renderer } from '$lib/wgputoy/wgputoy';

export const getDimensions = (canvas_id) => {
    const canvas = document.getElementById("wgputoy-canvas");
    const parentWidth = canvas.clientWidth;
    const parentHeight = canvas.clientHeight;

    const baseIncrementX = Math.max(Math.floor(parentWidth) - 1, 1);
    const baseIncrementY = Math.max(Math.floor(parentHeight) - 1, 1);
    return { x: baseIncrementX, y: baseIncrementY };
};

export const createWgpuToyRenderer = async (canvas_id) => {
    const dim = getDimensions(canvas_id);
    console.log("dim", dim);
    return create_renderer(dim.x, dim.y, canvas_id);
}
