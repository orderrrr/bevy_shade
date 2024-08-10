import { create_renderer } from '$lib/wgputoy/wgputoy';

export const getDimensions = (parentWidth, parentHeight) => {
    const baseIncrementX = Math.max(Math.floor(parentWidth / 32) - 1, 1);
    const baseIncrementY = Math.max(Math.floor(parentHeight / 32) - 1, 1);
    return { x: baseIncrementX * 32, y: baseIncrementY * 18 };
};

export const createWgpuToyRenderer = async (width, height, canvas_id) => {
    const dim = getDimensions(width * window.devicePixelRatio, height * window.devicePixelRatio);
    return create_renderer(dim.x, dim.y, canvas_id);
}
